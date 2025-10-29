use crate::error::DbResult;
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, Set};

use super::slice::*;
use super::*;

impl Entity {
    pub const PER_PAGE: u64 = 20;

    pub async fn create(conn: &DbConn, new_asset: NewAsset) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let asset = ActiveModel {
            file_url: Set(new_asset.file_url),
            file_name: Set(new_asset.file_name),
            mime_type: Set(new_asset.mime_type),
            size: Set(new_asset.size),
            uploaded_at: Set(now),
            owner_id: Set(new_asset.owner_id),
            context: Set(new_asset.context),
            ..Default::default()
        };

        match asset.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn update(
        conn: &DbConn,
        asset_id: i32,
        update_asset: UpdateAsset,
    ) -> DbResult<Option<Model>> {
        let asset: Option<Model> = match Self::find_by_id(asset_id).one(conn).await {
            Ok(asset) => asset,
            Err(err) => return Err(err.into()),
        };

        if let Some(asset_model) = asset {
            let mut asset_active: ActiveModel = asset_model.into();

            if let Some(file_url) = update_asset.file_url {
                asset_active.file_url = Set(file_url);
            }

            asset_active.file_name = Set(update_asset.file_name);
            asset_active.mime_type = Set(update_asset.mime_type);
            asset_active.size = Set(update_asset.size);
            asset_active.owner_id = Set(update_asset.owner_id);
            asset_active.context = Set(update_asset.context);

            match asset_active.update(conn).await {
                Ok(updated_asset) => Ok(Some(updated_asset)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn delete(conn: &DbConn, asset_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(asset_id).exec(conn).await {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_by_id_or_filename(
        conn: &DbConn,
        asset_id: Option<i32>,
        file_name: Option<String>,
    ) -> DbResult<Option<Model>> {
        if asset_id.is_none() && file_name.is_none() {
            return Err(
                DbErr::Custom("Either asset_id or file_name must be provided".to_string()).into(),
            );
        }
        let mut asset_query = Self::find();
        if let Some(id) = asset_id {
            asset_query = asset_query.filter(Column::Id.eq(id));
        } else if let Some(name) = file_name {
            asset_query = asset_query.filter(Column::FileName.eq(name));
        }
        match asset_query.one(conn).await {
            Ok(asset) => Ok(asset),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_with_query(conn: &DbConn, query: AssetQuery) -> DbResult<(Vec<Model>, u64)> {
        let mut asset_query = Self::find();

        if let Some(search_term) = query.search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            asset_query = asset_query.filter(
                Condition::any()
                    .add(Column::FileName.contains(&search_pattern))
                    .add(Column::MimeType.contains(&search_pattern)),
            );
        }

        if let Some(owner_id_filter) = query.owner_id {
            asset_query = asset_query.filter(Column::OwnerId.eq(owner_id_filter));
        }

        if let Some(context_filter) = query.context {
            asset_query = asset_query.filter(Column::Context.eq(context_filter));
        }

        // Date range filters
        if let Some(ts) = query.uploaded_at_gt {
            asset_query = asset_query.filter(Column::UploadedAt.gt(ts));
        }
        if let Some(ts) = query.uploaded_at_lt {
            asset_query = asset_query.filter(Column::UploadedAt.lt(ts));
        }

        // Multi-field sorting with per-field order
        if let Some(sorts) = query.sorts {
            for sort in sorts {
                let column = match sort.field.as_str() {
                    "file_name" => Some(Column::FileName),
                    "mime_type" => Some(Column::MimeType),
                    "size" => Some(Column::Size),
                    "uploaded_at" => Some(Column::UploadedAt),
                    "owner_id" => Some(Column::OwnerId),
                    "context" => Some(Column::Context),
                    _ => None,
                };
                if let Some(col) = column {
                    asset_query = asset_query.order_by(col, sort.order);
                }
            }
        } else {
            asset_query = asset_query.order_by(Column::UploadedAt, Order::Desc);
        }

        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        let paginator = asset_query.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
