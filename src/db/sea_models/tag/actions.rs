use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, Set};
use crate::error::{DbResult, ErrorCode, ErrorResponse};

use super::*;

impl Entity {
    const PER_PAGE: u64 = 20;

    // Create a new tag
    pub async fn create(conn: &DbConn, new_tag: NewTag) -> DbResult<Model> {
        let now = chrono::Utc::now().naive_utc();
        let tag = ActiveModel {
            name: Set(new_tag.name),
            slug: Set(new_tag.slug),
            description: Set(new_tag.description),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match tag.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Update an existing tag
    pub async fn update(
        conn: &DbConn,
        tag_id: i32,
        update_tag: UpdateTag,
    ) -> DbResult<Option<Model>> {
        // Get tag directly using SeaORM's Entity::find_by_id
        let tag: Option<Model> = match Self::find_by_id(tag_id).one(conn).await {
            Ok(tag) => tag,
            Err(err) => return Err(err.into()),
        };

        if let Some(tag_model) = tag {
            let mut tag_active: ActiveModel = tag_model.into();

            if let Some(name) = update_tag.name {
                tag_active.name = Set(name);
            }

            if let Some(slug) = update_tag.slug {
                tag_active.slug = Set(slug);
            }

            if let Some(description) = update_tag.description {
                tag_active.description = Set(Some(description));
            }

            tag_active.updated_at = Set(chrono::Utc::now().naive_utc());

            match tag_active.update(conn).await {
                Ok(updated_tag) => Ok(Some(updated_tag)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    // Delete a tag
    pub async fn delete(conn: &DbConn, tag_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(tag_id).exec(conn).await {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Find tag by ID
    pub async fn get_by_id(conn: &DbConn, tag_id: i32) -> DbResult<Option<Model>> {
        // Use SeaORM's Entity::find_by_id directly
        match Self::find_by_id(tag_id).one(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }
    
    // Find tag by ID with not found handling
    pub async fn find_by_id_with_404(conn: &DbConn, tag_id: i32) -> DbResult<Model> {
        // Use the basic get_by_id and transform the Option result
        match Self::find_by_id(tag_id).one(conn).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                          .with_message(&format!("Tag with ID {} not found", tag_id))),
            Err(err) => Err(err.into()),
        }
    }

    // Find all tags
    pub async fn find_all(conn: &DbConn) -> DbResult<Vec<Model>> {
        match Self::find()
            .order_by(Column::Name, Order::Desc)
            .all(conn)
            .await
        {
            Ok(models) => Ok(models),
            Err(err) => Err(err.into()),
        }
    }

    // Find tags with query parameters
    pub async fn find_with_query(
        conn: &DbConn,
        query: TagQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut tag_query = Self::find();

        // Handle search filter
        if let Some(search_term) = query.search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            tag_query = tag_query.filter(
                Condition::any()
                    .add(Column::Name.contains(&search_pattern))
                    .add(Column::Description.contains(&search_pattern)),
            );
        }

        // Handle ordering
        match query.sort_order.as_deref() {
            Some("asc") => {
                tag_query = tag_query.order_by(Column::Name, Order::Asc);
            }
            _ => {
                tag_query = tag_query.order_by(Column::Name, Order::Desc);
            }
        }

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        let paginator = tag_query.paginate(conn, Self::PER_PAGE);
        
        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => {
                match paginator.fetch_page(page - 1).await {
                    Ok(results) => Ok((results, total)),
                    Err(err) => Err(err.into()),
                }
            },
            Err(err) => Err(err.into()),
        }
    }
}
