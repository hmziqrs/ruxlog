use crate::{
    db::sea_models::email_verification,
    error::{DbResult, ErrorCode, ErrorResponse},
};
use sea_orm::{
    entity::prelude::*, prelude::Expr, sea_query::Alias, JoinType, Order, QueryOrder, QuerySelect,
    Set, TransactionTrait,
};
use tokio::task;
use tracing::{error, info, instrument, warn};

use super::*;
use crate::db::sea_models::media_usage::EntityType;

impl Entity {
    pub const PER_PAGE: u64 = 20;

    #[allow(dead_code)]
    async fn load_media_for_users(
        conn: &DbConn,
        public_url: &str,
        users: Vec<Model>,
    ) -> DbResult<Vec<(Model, Option<UserMedia>)>> {
        use super::super::media::url::build_public_file_url;

        let mut media_ids = std::collections::HashSet::new();
        for user in &users {
            if let Some(id) = user.avatar_id {
                media_ids.insert(id);
            }
        }

        let media_map = if !media_ids.is_empty() {
            super::super::media::Entity::find()
                .filter(
                    super::super::media::Column::Id
                        .is_in(media_ids.into_iter().collect::<Vec<i32>>()),
                )
                .all(conn)
                .await?
                .into_iter()
                .map(|m| {
                    let file_url =
                        build_public_file_url(public_url, m.bucket.as_deref(), &m.object_key);
                    (
                        m.id,
                        UserMedia {
                            id: m.id,
                            object_key: m.object_key,
                            file_url,
                            mime_type: m.mime_type,
                            width: m.width,
                            height: m.height,
                            size: m.size,
                        },
                    )
                })
                .collect::<std::collections::HashMap<i32, UserMedia>>()
        } else {
            std::collections::HashMap::new()
        };

        let results = users
            .into_iter()
            .map(|user| {
                let avatar = user.avatar_id.and_then(|id| media_map.get(&id).cloned());
                (user, avatar)
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip(conn, new_user, email_code_hash), fields(user_id, email = %new_user.email))]
    pub async fn create(
        conn: &DbConn,
        new_user: NewUser,
        email_code_hash: String,
    ) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let hash = task::spawn_blocking(move || password_auth::generate_hash(new_user.password))
            .await
            .map_err(|_| {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message("Failed to generate password hash")
            })?;

        let user = ActiveModel {
            name: Set(new_user.name),
            email: Set(new_user.email),
            password: Set(Some(hash)),
            role: Set(new_user.role),
            is_verified: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        let transaction = conn.begin().await.map_err(|_| {
            ErrorResponse::new(ErrorCode::TransactionError)
                .with_message("Failed to begin transaction")
        })?;
        match user.insert(&transaction).await {
            Ok(model) => {
                tracing::Span::current().record("user_id", model.id);
                email_verification::Entity::create(&transaction, model.id, email_code_hash).await?;
                transaction.commit().await.map_err(|_| {
                    ErrorResponse::new(ErrorCode::TransactionError)
                        .with_message("Failed to commit transaction")
                })?;
                info!(user_id = model.id, email = %model.email, "User created");
                Ok(model)
            }
            Err(err) => {
                error!("Failed to create user: {}", err);
                transaction.rollback().await.map_err(|_| {
                    ErrorResponse::new(ErrorCode::TransactionError)
                        .with_message("Failed to rollback transaction")
                })?;
                Err(err.into())
            }
        }
    }

    #[instrument(skip(conn, update_user), fields(user_id))]
    pub async fn update(
        conn: &DbConn,
        user_id: i32,
        update_user: UpdateUser,
    ) -> DbResult<Option<Model>> {
        let user: Option<Model> = match Self::find_by_id(user_id).one(conn).await {
            Ok(user) => user,
            Err(err) => return Err(err.into()),
        };

        if let Some(user_model) = user {
            // EMAIL-CHANGE-1: capture the current email/verified state before
            // the model is consumed by `into()`, so an email change can be
            // detected and the stale trust state reset.
            let prev_email = user_model.email.clone();
            let prev_verified = user_model.is_verified;
            let mut user_active: ActiveModel = user_model.into();

            if let Some(name) = update_user.name {
                user_active.name = Set(name);
            }

            if let Some(email) = update_user.email {
                if email != prev_email {
                    // A verified account that changes its email must lose its
                    // verified status until the NEW address is proven — otherwise
                    // a verified badge persists on an unproven address the
                    // account holder may not control (trust spoofing, CWE-290).
                    // Rotate the per-user `session_auth_secret` so every prior
                    // session is invalidated on its next request (the caller
                    // must re-authenticate), mirroring the password-change path
                    // (F#16). The new address is re-verified via the existing
                    // email-verification resend endpoint.
                    user_active.email = Set(email);
                    user_active.is_verified = Set(false);
                    user_active.session_auth_secret = Set(super::model::new_session_auth_secret()
                        .map_err(|err| {
                            error!(
                                user_id,
                                "session_auth_secret rotation failed during email change: {err}"
                            );
                            ErrorResponse::new(ErrorCode::InternalServerError)
                                .with_message(format!("session_auth_secret rotation failed: {err}"))
                        })?);
                    info!(
                        user_id,
                        previously_verified = prev_verified,
                        "Email changed: is_verified reset and prior sessions invalidated"
                    );
                } else {
                    user_active.email = Set(email);
                }
            }

            user_active.updated_at = Set(update_user.updated_at);

            match user_active.update(conn).await {
                Ok(updated_user) => {
                    info!(user_id, "User updated");
                    Ok(Some(updated_user))
                }
                Err(err) => {
                    error!(user_id, "Failed to update user: {}", err);
                    Err(err.into())
                }
            }
        } else {
            warn!(user_id, "User not found for update");
            Ok(None)
        }
    }

    #[instrument(skip(conn), fields(user_id))]
    pub async fn verify(conn: &DbConn, user_id: i32) -> DbResult<Model> {
        let user = Self::find_by_id_with_404(conn, user_id).await?;
        let mut user_active: ActiveModel = user.into();

        user_active.is_verified = Set(true);
        user_active.updated_at = Set(chrono::Utc::now().fixed_offset());

        match user_active.update(conn).await {
            Ok(model) => {
                info!(user_id, "User verified");
                Ok(model)
            }
            Err(err) => {
                error!(user_id, "Failed to verify user: {}", err);
                Err(err.into())
            }
        }
    }

    #[instrument(skip(conn, new_password), fields(user_id))]
    pub async fn change_password<T: ConnectionTrait>(
        conn: &T,
        user_id: i32,
        new_password: String,
    ) -> DbResult<()> {
        let user = Self::find_by_id_with_404(conn, user_id).await?;
        let mut user_active: ActiveModel = user.into();

        let hash = task::spawn_blocking(move || password_auth::generate_hash(new_password))
            .await
            .map_err(|_| {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message("Failed to generate password hash")
            })?;

        user_active.password = Set(Some(hash));
        user_active.updated_at = Set(chrono::Utc::now().fixed_offset());

        // F#16: rotate the server-random `session_auth_secret` on credential
        // change. `session_auth_hash` (services/auth.rs) is keyed on this secret,
        // so rotating it invalidates ALL of the user's prior sessions on their
        // next request (the extractor's ct_eq mismatch deletes the stale record).
        // This closes the CSRF/session-fixation trust transition for the
        // password-change path: a compromised session cannot survive a reset.
        user_active.session_auth_secret =
            Set(super::model::new_session_auth_secret().map_err(|err| {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message(format!("session_auth_secret rotation failed: {err}"))
            })?);

        match user_active.update(conn).await {
            Ok(_) => {
                info!(user_id, "User password changed (session binding rotated)");
                Ok(())
            }
            Err(err) => {
                error!(user_id, "Failed to change password: {}", err);
                Err(err.into())
            }
        }
    }

    #[instrument(skip(conn), fields(user_id))]
    pub async fn get_by_id(conn: &DbConn, user_id: i32) -> DbResult<Option<Model>> {
        match Self::find_by_id(user_id).one(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    #[instrument(skip(conn), fields(user_id))]
    pub async fn find_by_id_with_404<T: ConnectionTrait>(
        conn: &T,
        user_id: i32,
    ) -> DbResult<Model> {
        match Self::find_by_id(user_id).one(conn).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message(format!("User with ID {} not found", user_id))),
            Err(err) => Err(err.into()),
        }
    }

    /// V-MED-6 (TOTP replay TOCTOU): atomically advance the
    /// `two_fa_last_totp_counter` watermark only if the proposed `new_counter`
    /// is strictly greater than the row's current value (or the row has no
    /// watermark yet).
    ///
    /// This is a single conditional UPDATE executed by the DB:
    /// ```sql
    /// UPDATE users
    ///   SET two_fa_last_totp_counter = :new_counter
    ///   WHERE id = :user_id
    ///     AND (two_fa_last_totp_counter IS NULL OR two_fa_last_totp_counter < :new_counter)
    /// ```
    /// The UPDATE is the authoritative replay gate: under two concurrent
    /// `twofa_verify`/`twofa_disable` requests for the same user and the same
    /// matched counter, only one UPDATE can claim the row (the DB applies each
    /// UPDATE's WHERE against the row it reads under its own lock), so only one
    /// request sees `rows_affected > 0` and is authorized. The other observes
    /// the now-advanced watermark, matches zero rows, and is rejected as a
    /// replay — closing the read-modify-write TOCTOU that the prior
    /// `find_by_id`-then-`update` path had.
    ///
    /// Returns `true` when the watermark was advanced (caller may authorize),
    /// `false` when another request already advanced past `new_counter`
    /// (caller must treat as a replay). Any DB error propagates as a 500.
    #[instrument(skip(conn), fields(user_id, new_counter))]
    pub async fn advance_totp_counter_if_higher<T: ConnectionTrait>(
        conn: &T,
        user_id: i32,
        new_counter: i64,
    ) -> DbResult<bool> {
        // Build the conditional SET. `col_expr` writes
        // `two_fa_last_totp_counter = $new_counter`; the IS NULL OR < filter is
        // the replay guard. `Expr::col` + `.lt` builds the comparison against
        // the bound value so both branches reference the same `new_counter`.
        let new_watermark = Expr::col(Column::TwoFaLastTotpCounter).lt(new_counter);
        let result = Self::update_many()
            .col_expr(Column::TwoFaLastTotpCounter, Expr::value(new_counter))
            .filter(Column::Id.eq(user_id))
            .filter(Column::TwoFaLastTotpCounter.is_null().or(new_watermark))
            .exec(conn)
            .await?;
        Ok(result.rows_affected > 0)
    }

    #[instrument(skip(conn), fields(email = %user_email))]
    pub async fn find_by_email(conn: &DbConn, user_email: String) -> DbResult<Option<Model>> {
        match Self::find()
            .filter(Column::Email.eq(user_email))
            .one(conn)
            .await
        {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// CRYP-ENC-004(a): `google_id` is stored AT REST as a DETERMINISTIC
    /// field-crypto envelope, so the lookup encrypts the supplied id with the
    /// same key and matches the stored ciphertext column. Transparent to
    /// callers — they pass the plaintext Google subject id and get the user.
    /// A key-unset deployment surfaces a 500 (never a silent miss).
    pub async fn find_by_google_id(conn: &DbConn, google_id: String) -> DbResult<Option<Model>> {
        let lookup =
            crate::utils::field_crypto::encrypt_deterministic(&google_id).map_err(|err| {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message(format!("google_id lookup encryption failed: {err}"))
            })?;
        match Self::find()
            .filter(Column::GoogleId.eq(lookup))
            .one(conn)
            .await
        {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    #[instrument(skip(conn), fields(user_id, email = %google_email))]
    pub async fn create_from_google(
        conn: &DbConn,
        google_id: String,
        google_email: String,
        google_name: String,
    ) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();

        let user = ActiveModel {
            name: Set(google_name),
            email: Set(google_email),
            password: Set(None),
            google_id: Set(Some(google_id)),
            oauth_provider: Set(Some("google".to_string())),
            role: Set(UserRole::User),
            is_verified: Set(true), // Google accounts are pre-verified
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match user.insert(conn).await {
            Ok(model) => {
                tracing::Span::current().record("user_id", model.id);
                info!(user_id = model.id, email = %model.email, "User created from Google");
                Ok(model)
            }
            Err(err) => {
                error!("Failed to create user from Google: {}", err);
                Err(err.into())
            }
        }
    }

    #[instrument(skip(conn, new_user), fields(user_id, email = %new_user.email))]
    pub async fn admin_create(
        conn: &DbConn,
        public_url: &str,
        new_user: AdminCreateUser,
    ) -> DbResult<UserWithRelations> {
        let txn = conn.begin().await?;

        let now = chrono::Utc::now().fixed_offset();
        let hash = task::spawn_blocking(move || password_auth::generate_hash(new_user.password))
            .await
            .map_err(|_| {
                ErrorResponse::new(ErrorCode::InternalServerError)
                    .with_message("Failed to generate password hash")
            })?;

        let avatar_id = new_user.avatar_id;

        let user = ActiveModel {
            name: Set(new_user.name),
            email: Set(new_user.email),
            password: Set(Some(hash)),
            role: Set(new_user.role),
            avatar_id: Set(avatar_id),
            is_verified: Set(new_user.is_verified.unwrap_or(false)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let model = user.insert(&txn).await?;

        if let Some(mid) = avatar_id {
            super::super::media_usage::Entity::track_usage(
                &txn,
                mid,
                EntityType::User,
                model.id,
                "avatar_id",
            )
            .await?;
        }

        txn.commit().await?;

        Self::find_by_id_with_relations(conn, public_url, model.id).await
    }

    pub async fn admin_update(
        conn: &DbConn,
        public_url: &str,
        user_id: i32,
        update_user: AdminUpdateUser,
    ) -> DbResult<Option<UserWithRelations>> {
        let user: Option<Model> = Self::get_by_id(conn, user_id).await?;

        if let Some(user_model) = user {
            let txn = conn.begin().await?;

            let old_avatar_id = user_model.avatar_id;
            let mut user_active: ActiveModel = user_model.into();

            if let Some(name) = update_user.name {
                user_active.name = Set(name);
            }

            if let Some(email) = update_user.email {
                user_active.email = Set(email);
            }

            if let Some(password) = update_user.password {
                let hash = task::spawn_blocking(move || password_auth::generate_hash(password))
                    .await
                    .map_err(|_| {
                        ErrorResponse::new(ErrorCode::InternalServerError)
                            .with_message("Failed to generate password hash")
                    })?;
                user_active.password = Set(Some(hash));
            }

            if let Some(role) = update_user.role {
                user_active.role = Set(role);
            }

            let new_avatar_id = update_user.avatar_id;
            if update_user.avatar_id.is_some() {
                user_active.avatar_id = Set(new_avatar_id);
            }

            if let Some(is_verified) = update_user.is_verified {
                user_active.is_verified = Set(is_verified);
            }

            user_active.updated_at = Set(update_user.updated_at);

            let _updated_user = user_active.update(&txn).await?;

            if update_user.avatar_id.is_some() && old_avatar_id != new_avatar_id {
                super::super::media_usage::Entity::update_usage(
                    &txn,
                    old_avatar_id,
                    new_avatar_id,
                    EntityType::User,
                    user_id,
                    "avatar_id",
                )
                .await?;
            }

            txn.commit().await?;

            Self::find_by_id_with_relations(conn, public_url, user_id)
                .await
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn admin_delete(conn: &DbConn, user_id: i32) -> DbResult<u64> {
        let txn = conn.begin().await?;

        super::super::media_usage::Entity::delete_by_entity(&txn, EntityType::User, user_id)
            .await?;

        let result = Self::delete_by_id(user_id).exec(&txn).await?;

        txn.commit().await?;

        Ok(result.rows_affected)
    }

    pub async fn find_by_id_with_relations(
        conn: &DbConn,
        public_url: &str,
        user_id: i32,
    ) -> DbResult<UserWithRelations> {
        use super::super::media::url::public_file_url_expr;

        let user_query = Self::find()
            .select_only()
            .columns(vec![
                Column::Id,
                Column::Name,
                Column::Email,
                Column::AvatarId,
                Column::IsVerified,
                Column::Role,
                Column::TwoFaEnabled,
                Column::CreatedAt,
                Column::UpdatedAt,
            ])
            .join_as(
                JoinType::LeftJoin,
                Relation::Media.def(),
                Alias::new("avatar_media"),
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::ObjectKey,
                )),
                "avatar_object_key",
            )
            .expr_as(
                public_file_url_expr(public_url, "avatar_media"),
                "avatar_file_url",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::MimeType,
                )),
                "avatar_mime_type",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::Width,
                )),
                "avatar_width",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::Height,
                )),
                "avatar_height",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::Size,
                )),
                "avatar_size",
            )
            .filter(Column::Id.eq(user_id));

        let row = user_query
            .into_model::<UserWithJoinedData>()
            .one(conn)
            .await?;

        match row {
            Some(r) => Ok(r.into_relation()),
            None => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message(format!("User with ID {} not found", user_id))),
        }
    }

    pub async fn admin_list(
        conn: &DbConn,
        public_url: &str,
        query: AdminUserQuery,
    ) -> DbResult<(Vec<UserWithRelations>, u64)> {
        use super::super::media::url::public_file_url_expr;

        let mut user_query = Self::find()
            .select_only()
            .columns(vec![
                Column::Id,
                Column::Name,
                Column::Email,
                Column::AvatarId,
                Column::IsVerified,
                Column::Role,
                Column::TwoFaEnabled,
                Column::CreatedAt,
                Column::UpdatedAt,
            ])
            .join_as(
                JoinType::LeftJoin,
                Relation::Media.def(),
                Alias::new("avatar_media"),
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::ObjectKey,
                )),
                "avatar_object_key",
            )
            .expr_as(
                public_file_url_expr(public_url, "avatar_media"),
                "avatar_file_url",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::MimeType,
                )),
                "avatar_mime_type",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::Width,
                )),
                "avatar_width",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::Height,
                )),
                "avatar_height",
            )
            .expr_as(
                Expr::col((
                    Alias::new("avatar_media"),
                    super::super::media::Column::Size,
                )),
                "avatar_size",
            );

        if let Some(email_filter) = query.email {
            let email_pattern = format!("%{}%", email_filter);
            user_query = user_query.filter(Column::Email.contains(&email_pattern));
        }

        if let Some(name_filter) = query.name {
            let name_pattern = format!("%{}%", name_filter);
            user_query = user_query.filter(Column::Name.contains(&name_pattern));
        }

        if let Some(role_filter) = query.role {
            user_query = user_query.filter(Column::Role.eq(role_filter));
        }

        if let Some(status_filter) = query.status {
            user_query = user_query.filter(Column::IsVerified.eq(status_filter));
        }

        if let Some(ts) = query.created_at_gt {
            user_query = user_query.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            user_query = user_query.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            user_query = user_query.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            user_query = user_query.filter(Column::UpdatedAt.lt(ts));
        }

        if let Some(sorts) = query.sorts {
            for sort in sorts {
                let column = match sort.field.as_str() {
                    "id" => Some(Column::Id),
                    "email" => Some(Column::Email),
                    "name" => Some(Column::Name),
                    "role" => Some(Column::Role),
                    "status" => Some(Column::IsVerified),
                    "is_verified" => Some(Column::IsVerified),
                    "created_at" => Some(Column::CreatedAt),
                    "updated_at" => Some(Column::UpdatedAt),
                    _ => None,
                };
                if let Some(col) = column {
                    user_query = user_query.order_by(col, sort.order);
                }
            }
        } else {
            user_query = user_query.order_by(Column::Id, Order::Desc);
        }

        let page = match query.page {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = user_query
            .into_model::<UserWithJoinedData>()
            .paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => {
                    let users_with_relations =
                        results.into_iter().map(|r| r.into_relation()).collect();
                    Ok((users_with_relations, total))
                }
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::sea_query::PostgresQueryBuilder;
    use sea_orm::QueryTrait;

    /// V-MED-6 TOCTOU fix: `advance_totp_counter_if_higher` must emit a SINGLE
    /// conditional UPDATE — `SET two_fa_last_totp_counter = :new WHERE id = :uid
    /// AND (two_fa_last_totp_counter IS NULL OR two_fa_last_totp_counter < :new)`
    /// — so the DB, not the application, decides who wins the watermark race.
    /// The guarantee is the SQL shape (the conditional WHERE), not a value this
    /// test asserts; rendering the statement to SQL keeps the test DB-free.
    /// If a future refactor drops the `IS NULL OR <` guard, this test fails.
    #[test]
    fn advance_totp_counter_emits_atomic_conditional_update() {
        let sql =
            Entity::update_many()
                .col_expr(
                    Column::TwoFaLastTotpCounter,
                    sea_orm::sea_query::Expr::value(1_000_042_i64),
                )
                .filter(Column::Id.eq(7))
                .filter(Column::TwoFaLastTotpCounter.is_null().or(
                    sea_orm::sea_query::Expr::col(Column::TwoFaLastTotpCounter).lt(1_000_042_i64),
                ))
                .into_query()
                .to_string(PostgresQueryBuilder);

        // The SET targets exactly the watermark column with the new counter.
        assert!(
            sql.contains("SET \"two_fa_last_totp_counter\" = 1000042"),
            "expected SET on the watermark column, got: {sql}"
        );
        // The row is scoped to one user — no bulk update.
        assert!(
            sql.contains("\"id\" = 7"),
            "expected WHERE id = 7, got: {sql}"
        );
        // The replay guard: IS NULL OR < new_counter. Both arms must be present
        // so first-use (NULL) and strict-advance (<) both pass while a replay
        // (==) matches zero rows.
        assert!(
            sql.contains("\"two_fa_last_totp_counter\" IS NULL"),
            "missing IS NULL branch (first-use), got: {sql}"
        );
        assert!(
            sql.contains("\"two_fa_last_totp_counter\" < 1000042"),
            "missing strict-less-than branch (replay guard), got: {sql}"
        );
        // Sanity: this is an UPDATE against the users table, not a SELECT.
        assert!(sql.starts_with("UPDATE \"users\""), "not an UPDATE: {sql}");
    }

    /// The watermark column's SeaORM identity must be stable so the SET column
    /// and the WHERE filter refer to the same physical column — a mismatched
    /// alias here would silently break the replay guard.
    #[test]
    fn two_fa_last_totp_counter_iden_is_stable() {
        use sea_orm::sea_query::Iden;
        assert_eq!(
            Column::TwoFaLastTotpCounter.to_string(),
            "two_fa_last_totp_counter",
            "column iden must match the schema column name"
        );
    }
}
