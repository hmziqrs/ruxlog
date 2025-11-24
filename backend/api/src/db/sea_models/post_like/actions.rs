use crate::error::DbResult;
use sea_orm::{entity::prelude::*, Set, TransactionTrait};
use tracing::{info, instrument, warn};

use super::*;

impl Entity {
    /// Check if a user has liked a specific post
    #[instrument(skip(conn), fields(post_id, user_id))]
    pub async fn has_liked(conn: &DbConn, post_id: i32, user_id: i32) -> DbResult<bool> {
        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .filter(Column::UserId.eq(user_id))
            .count(conn)
            .await?;

        Ok(count > 0)
    }

    /// Like a post (with transaction to update likes_count atomically)
    /// Returns (success, new_likes_count)
    #[instrument(skip(conn), fields(post_id, user_id))]
    pub async fn like_post(conn: &DbConn, post_id: i32, user_id: i32) -> DbResult<(bool, i32)> {
        let transaction = conn.begin().await?;

        // Check if already liked
        let existing = Self::find()
            .filter(Column::PostId.eq(post_id))
            .filter(Column::UserId.eq(user_id))
            .one(&transaction)
            .await?;

        if existing.is_some() {
            // Already liked, just return current count
            let post = super::super::post::Entity::find_by_id(post_id)
                .one(&transaction)
                .await?;
            let likes_count = post.map(|p| p.likes_count).unwrap_or(0);
            transaction.rollback().await?;
            warn!(post_id, user_id, "User already liked this post");
            return Ok((false, likes_count));
        }

        // Create the like record
        let now = chrono::Utc::now().fixed_offset();
        let like = ActiveModel {
            post_id: Set(post_id),
            user_id: Set(user_id),
            created_at: Set(now),
            ..Default::default()
        };

        match like.insert(&transaction).await {
            Ok(_) => {}
            Err(err) => {
                transaction.rollback().await?;
                return Err(err.into());
            }
        }

        // Increment likes_count on the post
        let post = super::super::post::Entity::find_by_id(post_id)
            .one(&transaction)
            .await?;

        let new_likes_count = if let Some(post_model) = post {
            let new_count = post_model.likes_count + 1;
            let mut post_active: super::super::post::ActiveModel = post_model.into();
            post_active.likes_count = Set(new_count);
            match post_active.update(&transaction).await {
                Ok(_) => new_count,
                Err(err) => {
                    transaction.rollback().await?;
                    return Err(err.into());
                }
            }
        } else {
            transaction.rollback().await?;
            return Err(DbErr::RecordNotFound("Post not found".to_string()).into());
        };

        transaction.commit().await?;
        info!(post_id, user_id, likes_count = new_likes_count, "Post liked");
        Ok((true, new_likes_count))
    }

    /// Unlike a post (with transaction to update likes_count atomically)
    /// Returns (success, new_likes_count)
    #[instrument(skip(conn), fields(post_id, user_id))]
    pub async fn unlike_post(conn: &DbConn, post_id: i32, user_id: i32) -> DbResult<(bool, i32)> {
        let transaction = conn.begin().await?;

        // Check if the like exists
        let existing = Self::find()
            .filter(Column::PostId.eq(post_id))
            .filter(Column::UserId.eq(user_id))
            .one(&transaction)
            .await?;

        let like_record = match existing {
            Some(record) => record,
            None => {
                // Not liked, just return current count
                let post = super::super::post::Entity::find_by_id(post_id)
                    .one(&transaction)
                    .await?;
                let likes_count = post.map(|p| p.likes_count).unwrap_or(0);
                transaction.rollback().await?;
                warn!(post_id, user_id, "User hasn't liked this post");
                return Ok((false, likes_count));
            }
        };

        // Delete the like record
        match like_record.delete(&transaction).await {
            Ok(_) => {}
            Err(err) => {
                transaction.rollback().await?;
                return Err(err.into());
            }
        }

        // Decrement likes_count on the post (but never below 0)
        let post = super::super::post::Entity::find_by_id(post_id)
            .one(&transaction)
            .await?;

        let new_likes_count = if let Some(post_model) = post {
            // Ensure we never go below 0
            let new_count = (post_model.likes_count - 1).max(0);
            let mut post_active: super::super::post::ActiveModel = post_model.into();
            post_active.likes_count = Set(new_count);
            match post_active.update(&transaction).await {
                Ok(_) => new_count,
                Err(err) => {
                    transaction.rollback().await?;
                    return Err(err.into());
                }
            }
        } else {
            transaction.rollback().await?;
            return Err(DbErr::RecordNotFound("Post not found".to_string()).into());
        };

        transaction.commit().await?;
        info!(
            post_id,
            user_id,
            likes_count = new_likes_count,
            "Post unliked"
        );
        Ok((true, new_likes_count))
    }

    /// Get like status for a user on a specific post
    #[instrument(skip(conn), fields(post_id, user_id))]
    pub async fn get_like_status(
        conn: &DbConn,
        post_id: i32,
        user_id: i32,
    ) -> DbResult<LikeStatus> {
        let is_liked = Self::has_liked(conn, post_id, user_id).await?;
        let post = super::super::post::Entity::find_by_id(post_id)
            .one(conn)
            .await?;
        let likes_count = post.map(|p| p.likes_count).unwrap_or(0);

        Ok(LikeStatus {
            post_id,
            is_liked,
            likes_count,
        })
    }

    /// Get like status for a user on multiple posts
    #[instrument(skip(conn), fields(user_id, post_count = post_ids.len()))]
    pub async fn get_like_status_batch(
        conn: &DbConn,
        post_ids: &[i32],
        user_id: i32,
    ) -> DbResult<Vec<LikeStatus>> {
        // Get all likes for this user on these posts
        let likes = Self::find()
            .filter(Column::PostId.is_in(post_ids.to_vec()))
            .filter(Column::UserId.eq(user_id))
            .all(conn)
            .await?;

        let liked_post_ids: std::collections::HashSet<i32> =
            likes.iter().map(|l| l.post_id).collect();

        // Get all posts to get their likes_count
        let posts = super::super::post::Entity::find()
            .filter(super::super::post::Column::Id.is_in(post_ids.to_vec()))
            .all(conn)
            .await?;

        let statuses = posts
            .into_iter()
            .map(|p| LikeStatus {
                post_id: p.id,
                is_liked: liked_post_ids.contains(&p.id),
                likes_count: p.likes_count,
            })
            .collect();

        Ok(statuses)
    }

    /// Count total likes for a post
    #[instrument(skip(conn), fields(post_id))]
    pub async fn count_by_post(conn: &DbConn, post_id: i32) -> DbResult<u64> {
        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .count(conn)
            .await?;

        Ok(count)
    }
}
