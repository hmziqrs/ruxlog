pub mod model {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "post_series_posts")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,

        pub series_id: i32,
        pub post_id: i32,

        pub sort_order: i32,

        pub created_at: DateTimeWithTimeZone,
        pub updated_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::super::post_series::Entity",
            from = "Column::SeriesId",
            to = "super::super::post_series::Column::Id",
            on_update = "Cascade",
            on_delete = "Cascade"
        )]
        Series,
        #[sea_orm(
            belongs_to = "super::super::post::Entity",
            from = "Column::PostId",
            to = "super::super::post::Column::Id",
            on_update = "Cascade",
            on_delete = "Cascade"
        )]
        Post,
    }

    impl Related<super::super::post_series::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Series.def()
        }
    }

    impl Related<super::super::post::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Post.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod slice {
    use serde::{Deserialize, Serialize};

    /// Payload to add a post into a series
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct NewPostSeriesPost {
        pub series_id: i32,
        pub post_id: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sort_order: Option<i32>,
    }

    /// Payload to remove a post from a series
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct RemovePostSeriesPost {
        pub series_id: i32,
        pub post_id: i32,
    }

    /// Payload to reorder a post within a series
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ReorderSeriesPost {
        pub series_id: i32,
        pub post_id: i32,
        pub new_sort_order: i32,
    }

    /// Query parameters for listing mappings within a series
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct SeriesPostsListQuery {
        pub series_id: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub page: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub per_page: Option<u64>,
    }
}

pub mod actions {
    use crate::error::DbResult;
    use sea_orm::{
        entity::prelude::*, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
        TransactionTrait,
    };

    use super::model::*;
    use super::slice::*;

    impl Entity {
        pub const PER_PAGE: u64 = 10;

        /// Add a post to a series. If mapping already exists, returns existing model.
        /// If `sort_order` is not provided, appends to the end.
        pub async fn add(conn: &DbConn, payload: NewPostSeriesPost) -> DbResult<Model> {
            let now = chrono::Utc::now().fixed_offset();

            // Idempotency: return existing mapping if present
            if let Some(existing) = Entity::find()
                .filter(Column::SeriesId.eq(payload.series_id))
                .filter(Column::PostId.eq(payload.post_id))
                .one(conn)
                .await?
            {
                return Ok(existing);
            }

            let txn = conn.begin().await?;

            let next_order = match payload.sort_order {
                Some(order) if order > 0 => order,
                _ => {
                    let max = Self::max_sort_order_for_series(&txn, payload.series_id).await?;
                    max.unwrap_or(0) + 1
                }
            };

            let active = ActiveModel {
                series_id: Set(payload.series_id),
                post_id: Set(payload.post_id),
                sort_order: Set(next_order),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            let created = active.insert(&txn).await?;

            // Normalize ordering to ensure uniqueness and contiguous sequence
            Self::normalize_sort_orders(&txn, payload.series_id).await?;

            txn.commit().await?;
            Ok(created)
        }

        /// Remove a post from a series. Returns rows affected.
        pub async fn remove(conn: &DbConn, payload: RemovePostSeriesPost) -> DbResult<u64> {
            let txn = conn.begin().await?;

            let res = Entity::delete_many()
                .filter(Column::SeriesId.eq(payload.series_id))
                .filter(Column::PostId.eq(payload.post_id))
                .exec(&txn)
                .await?;

            // Normalize remaining after removal
            Self::normalize_sort_orders(&txn, payload.series_id).await?;

            txn.commit().await?;
            Ok(res.rows_affected)
        }

        /// Reorder a post within a series (1-based position). Returns rows updated.
        pub async fn reorder(conn: &DbConn, payload: ReorderSeriesPost) -> DbResult<u64> {
            let txn = conn.begin().await?;

            let mut items = Entity::find()
                .filter(Column::SeriesId.eq(payload.series_id))
                .order_by_asc(Column::SortOrder)
                .order_by_asc(Column::Id)
                .all(&txn)
                .await?;

            // Find index of the target post
            if let Some((idx, _)) = items
                .iter()
                .enumerate()
                .find(|(_, m)| m.post_id == payload.post_id)
            {
                let item = items.remove(idx);

                // Clamp new position
                let len = items.len() as i32;
                let mut target_pos = payload.new_sort_order;
                if target_pos < 1 {
                    target_pos = 1;
                }
                if target_pos > len + 1 {
                    target_pos = len + 1;
                }

                // Insert at new position (convert to 0-based)
                items.insert((target_pos - 1) as usize, item);

                // Persist new order
                let now = chrono::Utc::now().fixed_offset();
                let mut updated_count = 0u64;
                for (i, m) in items.into_iter().enumerate() {
                    let mut active: ActiveModel = m.into();
                    let desired = (i as i32) + 1;
                    active.sort_order = Set(desired);
                    active.updated_at = Set(now);
                    active.update(&txn).await?;
                    updated_count += 1;
                }

                txn.commit().await?;
                Ok(updated_count)
            } else {
                // Nothing to do if mapping not found
                Ok(0)
            }
        }

        /// List mappings for a series ordered by sort_order asc, id asc.
        /// Returns (items, total_count)
        pub async fn list_by_series(
            conn: &DbConn,
            query: SeriesPostsListQuery,
        ) -> DbResult<(Vec<Model>, u64)> {
            let per_page = query.per_page.unwrap_or(Self::PER_PAGE);
            let page = match query.page {
                Some(p) if p > 0 => p,
                _ => 1,
            };

            let finder = Entity::find()
                .filter(Column::SeriesId.eq(query.series_id))
                .order_by_asc(Column::SortOrder)
                .order_by_asc(Column::Id);

            let paginator = finder.paginate(conn, per_page);
            let total = paginator.num_items().await?;
            let items = paginator.fetch_page(page - 1).await?;
            Ok((items, total))
        }

        /// Count mappings for a given series.
        pub async fn count_by_series(conn: &DbConn, series_id: i32) -> DbResult<u64> {
            let c = Entity::find()
                .filter(Column::SeriesId.eq(series_id))
                .count(conn)
                .await?;
            Ok(c)
        }

        /// Remove all mappings for a series. Returns rows affected.
        pub async fn clear_series(conn: &DbConn, series_id: i32) -> DbResult<u64> {
            let res = Entity::delete_many()
                .filter(Column::SeriesId.eq(series_id))
                .exec(conn)
                .await?;
            Ok(res.rows_affected)
        }

        /// Get the maximum sort_order for a given series.
        async fn max_sort_order_for_series<C>(conn: &C, series_id: i32) -> DbResult<Option<i32>>
        where
            C: ConnectionTrait,
        {
            let item = Entity::find()
                .filter(Column::SeriesId.eq(series_id))
                .order_by_desc(Column::SortOrder)
                .order_by_desc(Column::Id)
                .one(conn)
                .await?;
            Ok(item.map(|m| m.sort_order))
        }

        /// Normalize sort orders to be contiguous starting from 1.
        async fn normalize_sort_orders<C>(conn: &C, series_id: i32) -> DbResult<()>
        where
            C: ConnectionTrait,
        {
            let mut items = Entity::find()
                .filter(Column::SeriesId.eq(series_id))
                .order_by_asc(Column::SortOrder)
                .order_by_asc(Column::Id)
                .all(conn)
                .await?;

            let now = chrono::Utc::now().fixed_offset();
            for (idx, model) in items.drain(..).enumerate() {
                let desired = (idx as i32) + 1;
                if model.sort_order != desired {
                    let mut active: ActiveModel = model.into();
                    active.sort_order = Set(desired);
                    active.updated_at = Set(now);
                    active.update(conn).await?;
                }
            }
            Ok(())
        }
    }
}

// Re-exports for convenience and consistency with other modules
pub use model::{ActiveModel, Column, Entity, Model, Relation};
pub use slice::*;
