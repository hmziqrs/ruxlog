use sea_orm::{entity::prelude::*, Order, QueryOrder, Set};
use crate::error::DbResult;

use super::*;

impl Entity {
    const PER_PAGE: u64 = 20;

    // Create a new post view
    pub async fn create(conn: &DbConn, new_post_view: NewPostView) -> DbResult<Model> {
        let now = chrono::Utc::now().naive_utc();
        let post_view = ActiveModel {
            post_id: Set(new_post_view.post_id),
            ip_address: Set(new_post_view.ip_address),
            user_agent: Set(new_post_view.user_agent),
            created_at: Set(now),
            ..Default::default()
        };

        match post_view.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Find post views by post_id
    pub async fn find_by_post_id(
        conn: &DbConn,
        post_id: i32,
        query: PostViewQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut post_view_query = Self::find().filter(Column::PostId.eq(post_id));

        // Apply filters
        if let Some(ip_address) = &query.ip_address {
            post_view_query = post_view_query.filter(Column::IpAddress.eq(ip_address));
        }

        if let Some(created_at) = query.created_at {
            post_view_query = post_view_query.filter(Column::CreatedAt.eq(created_at));
        }

        // Handle sort_by fields
        if let Some(sort_fields) = &query.sort_by {
            for field in sort_fields {
                let order = if query.sort_order.as_deref() == Some("asc") {
                    Order::Asc
                } else {
                    Order::Desc
                };
                
                post_view_query = match field.as_str() {
                    "created_at" => post_view_query.order_by(Column::CreatedAt, order),
                    _ => post_view_query,
                };
            }
        } else {
            // Default ordering
            post_view_query = post_view_query.order_by(Column::CreatedAt, Order::Desc);
        }

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        
        let paginator = post_view_query.paginate(conn, Self::PER_PAGE);
        
        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    // Check if IP address has viewed post in the last 24 hours
    pub async fn has_viewed_recently(
        conn: &DbConn,
        post_id: i32,
        ip_address: &str,
    ) -> DbResult<bool> {
        let one_day_ago = chrono::Utc::now()
            .naive_utc()
            .checked_sub_days(chrono::Days::new(1))
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .filter(Column::IpAddress.eq(ip_address))
            .filter(Column::CreatedAt.gte(one_day_ago))
            .count(conn)
            .await?;

        Ok(count > 0)
    }

    // Count views by post ID
    pub async fn count_by_post_id(conn: &DbConn, post_id: i32) -> DbResult<i64> {
        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .count(conn)
            .await?;
        
        Ok(count as i64)
    }
}