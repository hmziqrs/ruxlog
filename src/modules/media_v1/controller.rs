use std::collections::{BTreeSet, HashMap};

use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use bytes::Bytes;
use chrono::{Datelike, Utc};
use sea_orm::{prelude::DateTimeWithTimeZone, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    db::sea_models::{
        category::{self, Model as CategoryModel},
        media::{self, Entity as Media, NewMedia},
        media_usage,
        media_variant::{Entity as MediaVariant, NewMediaVariant},
        post::{self, Model as PostModel},
        user::{self, Model as UserModel},
    },
    error::{ErrorCode, ErrorResponse},
    extractors::{ValidatedJson, ValidatedMultipart},
    services::{auth::AuthSession, image_optimizer},
    AppState,
};
use tracing::{debug, error, info, instrument, warn};

use super::validator::{MediaUploadMetadata, V1MediaListQuery, V1MediaUsageQuery};

const MAX_UPLOAD_SIZE_BYTES: usize = 20 * 1024 * 1024; // 20MiB ceiling

#[derive(Debug, Serialize)]
struct PostUsage {
    usage_id: i32,
    field_name: String,
    created_at: DateTimeWithTimeZone,
    post: Option<PostSummary>,
}

#[derive(Debug, Serialize)]
struct CategoryUsage {
    usage_id: i32,
    field_name: String,
    created_at: DateTimeWithTimeZone,
    category: Option<CategorySummary>,
}

#[derive(Debug, Serialize)]
struct UserUsage {
    usage_id: i32,
    field_name: String,
    created_at: DateTimeWithTimeZone,
    user: Option<UserSummary>,
}

#[derive(Debug, Serialize)]
struct PostSummary {
    id: i32,
    title: String,
    slug: String,
    status: post::PostStatus,
    featured_image_id: Option<i32>,
}

impl From<&PostModel> for PostSummary {
    fn from(model: &PostModel) -> Self {
        Self {
            id: model.id,
            title: model.title.clone(),
            slug: model.slug.clone(),
            status: model.status,
            featured_image_id: model.featured_image_id,
        }
    }
}

#[derive(Debug, Serialize)]
struct CategorySummary {
    id: i32,
    name: String,
    slug: String,
    cover_id: Option<i32>,
    logo_id: Option<i32>,
}

impl From<&CategoryModel> for CategorySummary {
    fn from(model: &CategoryModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            slug: model.slug.clone(),
            cover_id: model.cover_id,
            logo_id: model.logo_id,
        }
    }
}

#[derive(Debug, Serialize)]
struct UserSummary {
    id: i32,
    name: String,
    email: String,
    role: user::UserRole,
    avatar_id: Option<i32>,
}

impl From<&UserModel> for UserSummary {
    fn from(model: &UserModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            email: model.email.clone(),
            role: model.role,
            avatar_id: model.avatar_id,
        }
    }
}

#[derive(Default)]
struct UsageAccumulator {
    posts: Vec<PostUsage>,
    categories: Vec<CategoryUsage>,
    users: Vec<UserUsage>,
}

#[derive(Debug, Serialize)]
struct MediaUsageGroup {
    media_id: i32,
    media: Option<media::Model>,
    posts: Vec<PostUsage>,
    categories: Vec<CategoryUsage>,
    users: Vec<UserUsage>,
}

#[debug_handler]
#[instrument(
    skip(state, auth, multipart),
    fields(
        user_id,
        file_size,
        content_hash,
        is_duplicate,
        is_optimized,
        variant_count,
        result
    )
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    mut multipart: ValidatedMultipart,
) -> Result<impl IntoResponse, ErrorResponse> {
    let uploader = auth.user.ok_or_else(|| {
        error!("Unauthorized upload attempt");
        ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Authentication required to upload media")
    })?;

    tracing::Span::current().record("user_id", uploader.id);
    info!(user_id = uploader.id, "Processing media upload");

    let mut metadata = MediaUploadMetadata::default();
    let mut file_bytes: Option<Bytes> = None;
    let mut original_name: Option<String> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        error!(error = %err, "Failed to read multipart field");
        ErrorResponse::new(ErrorCode::ValidationError).with_details(err.to_string())
    })? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            original_name = field.file_name().map(|name| name.to_string());
            mime_type = field.content_type().map(|ty| ty.to_string());
            let bytes = field.bytes().await.map_err(|err| {
                error!(error = %err, "Failed to read uploaded file bytes");
                ErrorResponse::new(ErrorCode::FileUploadError)
                    .with_message("Failed to read uploaded file")
                    .with_details(err.to_string())
            })?;

            debug!(file_size = bytes.len(), "File bytes received");

            if bytes.len() > MAX_UPLOAD_SIZE_BYTES {
                warn!(
                    file_size = bytes.len(),
                    max_size = MAX_UPLOAD_SIZE_BYTES,
                    "Upload exceeds size limit"
                );
                return Err(ErrorResponse::new(ErrorCode::FileTooLarge)
                    .with_message("File size exceeds the 20MiB upload limit"));
            }

            file_bytes = Some(bytes);
        } else {
            let value = field.text().await.map_err(|err| {
                error!(error = %err, field = %field_name, "Failed to read form field");
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Failed to read accompanying form field")
                    .with_details(err.to_string())
            })?;

            metadata.apply_field(&field_name, &value).map_err(|msg| {
                warn!(field = %field_name, error = %msg, "Invalid metadata field");
                ErrorResponse::new(ErrorCode::InvalidValue).with_message(&msg)
            })?;
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| {
        error!("No file field in multipart upload");
        ErrorResponse::new(ErrorCode::MissingRequiredField).with_message("Missing file field")
    })?;

    tracing::Span::current().record("file_size", file_bytes.len() as i64);

    let mut hasher = Sha256::new();
    hasher.update(&file_bytes);
    let content_hash = format!("{:x}", hasher.finalize());

    debug!(content_hash = %content_hash, "File hash calculated");
    tracing::Span::current().record("content_hash", &content_hash);

    if let Some(existing) = Media::find_by_hash(&state.sea_db, &content_hash).await? {
        info!(
            media_id = existing.id,
            content_hash = %content_hash,
            "Duplicate file detected, returning existing media"
        );
        tracing::Span::current().record("is_duplicate", true);
        tracing::Span::current().record("result", "duplicate");
        return Ok((StatusCode::OK, Json(json!(existing))));
    }

    tracing::Span::current().record("is_duplicate", false);

    // Derive useful metadata if it was not supplied
    if metadata.width.is_none() || metadata.height.is_none() {
        if let Ok(dimensions) = imagesize::blob_size(&file_bytes) {
            debug!(
                width = dimensions.width,
                height = dimensions.height,
                "Image dimensions detected"
            );
            metadata.width = metadata
                .width
                .or_else(|| i32::try_from(dimensions.width).ok());
            metadata.height = metadata
                .height
                .or_else(|| i32::try_from(dimensions.height).ok());
        }
    }

    let mut extension = infer_extension(original_name.as_deref(), mime_type.as_deref());
    let mut content_type = mime_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let mut final_bytes = file_bytes.clone();
    let mut variants_to_upload = Vec::new();
    let mut is_optimized = false;
    let mut optimized_at = None;
    struct PreparedVariant {
        object_key: String,
        mime_type: String,
        width: Option<i32>,
        height: Option<i32>,
        size: i64,
        extension: Option<String>,
        quality: Option<i32>,
        variant_type: String,
    }
    let mut prepared_variants: Vec<PreparedVariant> = Vec::new();

    if content_type.starts_with("image/") {
        let optimization_request = image_optimizer::OptimizationRequest {
            bytes: &file_bytes,
            metadata: &metadata,
            reference: metadata.reference_type,
            original_mime: mime_type.as_deref(),
            original_extension: extension.as_deref(),
        };

        let optimization_outcome =
            match image_optimizer::optimize(&state.optimizer, optimization_request) {
                Ok(outcome) => outcome,
                Err(err) => {
                    warn!("image optimizer error: {}", err);
                    image_optimizer::OptimizationOutcome::Skipped(
                        image_optimizer::SkipReason::DecodeFailed,
                    )
                }
            };

        if let image_optimizer::OptimizationOutcome::Optimized(result) = optimization_outcome {
            final_bytes = result.original.bytes.clone();
            content_type = result.original.mime_type.clone();
            extension = Some(result.original.extension.clone());

            if let Ok(width) = i32::try_from(result.original.width) {
                metadata.width = Some(width);
            }
            if let Ok(height) = i32::try_from(result.original.height) {
                metadata.height = Some(height);
            }

            variants_to_upload = result.variants;
            is_optimized = true;
            optimized_at = Some(Utc::now().fixed_offset());
        }
    }

    let size_bytes = i64::try_from(final_bytes.len()).map_err(|_| {
        ErrorResponse::new(ErrorCode::InvalidValue)
            .with_message("File size exceeds supported range")
    })?;

    let object_key = build_object_key(extension.as_deref());
    let base_object_key = object_key
        .rsplit_once('.')
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_else(|| object_key.clone());

    let byte_stream = ByteStream::from(final_bytes.clone().to_vec());

    state
        .s3_client
        .put_object()
        .bucket(&state.r2.bucket)
        .key(&object_key)
        .body(byte_stream)
        .content_type(&content_type)
        .send()
        .await
        .map_err(|err| {
            ErrorResponse::new(ErrorCode::StorageError)
                .with_message("Failed to persist media to storage")
                .with_details(err.to_string())
        })?;

    for variant in variants_to_upload {
        let suffix = match variant.label {
            image_optimizer::VariantLabel::Width(width) => format!("@{}w", width),
            image_optimizer::VariantLabel::Lqip => "@lqip".to_string(),
            image_optimizer::VariantLabel::Original => continue,
        };

        let variant_key = format!(
            "{}{}{}",
            base_object_key,
            suffix,
            if variant.extension.is_empty() {
                String::new()
            } else {
                format!(".{}", variant.extension)
            }
        );

        let size_bytes = i64::try_from(variant.bytes.len()).map_err(|_| {
            ErrorResponse::new(ErrorCode::InvalidValue)
                .with_message("Variant size exceeds supported range")
        })?;

        prepared_variants.push(PreparedVariant {
            object_key: variant_key.clone(),
            mime_type: variant.mime_type.clone(),
            width: i32::try_from(variant.width).ok(),
            height: i32::try_from(variant.height).ok(),
            size: size_bytes,
            extension: if variant.extension.is_empty() {
                None
            } else {
                Some(variant.extension.clone())
            },
            quality: variant.quality.map(|q| i32::from(q)),
            variant_type: label_to_variant_type(&variant.label),
        });

        if let Err(err) = state
            .s3_client
            .put_object()
            .bucket(&state.r2.bucket)
            .key(&variant_key)
            .body(ByteStream::from(variant.bytes.to_vec()))
            .content_type(&variant.mime_type)
            .send()
            .await
        {
            warn!(
                "failed to upload optimized variant {}: {}",
                variant_key, err
            );
        }
    }

    let public_url = format!(
        "{}/{}",
        state.r2.public_url.trim_end_matches('/'),
        &object_key
    );

    let new_media = NewMedia {
        object_key,
        file_url: public_url,
        mime_type: content_type,
        width: metadata.width,
        height: metadata.height,
        size: size_bytes,
        extension,
        uploader_id: Some(uploader.id),
        reference_type: metadata.reference_type,
        content_hash: Some(content_hash),
        is_optimized,
        optimized_at,
    };

    let stored = Media::create(&state.sea_db, new_media).await?;

    if !prepared_variants.is_empty() {
        let records = prepared_variants
            .into_iter()
            .map(|variant| NewMediaVariant {
                media_id: stored.id,
                object_key: variant.object_key,
                mime_type: variant.mime_type,
                width: variant.width,
                height: variant.height,
                size: variant.size,
                extension: variant.extension,
                quality: variant.quality,
                variant_type: variant.variant_type,
            })
            .collect();

        MediaVariant::create_many(&state.sea_db, records).await?;
    }

    Ok((StatusCode::CREATED, Json(json!(stored))))
}

#[debug_handler]
pub async fn view(
    State(state): State<AppState>,
    Path(media_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match Media::find_by_id_with_usage(&state.sea_db, media_id).await? {
        Some(media) => Ok((StatusCode::OK, Json(json!(media)))),
        None => Err(ErrorResponse::new(ErrorCode::FileNotFound).with_message("Media not found")),
    }
}

#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1MediaListQuery>,
) -> impl IntoResponse {
    let query = payload.0.into_query();
    let page = query.page.unwrap_or(1);

    match Media::find_with_query(&state.sea_db, query).await {
        Ok((items, total)) => (
            StatusCode::OK,
            Json(json!({
                "data": items,
                "total": total,
                "per_page": Media::PER_PAGE,
                "page": page,
            })),
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

#[debug_handler]
pub async fn list_usage_details(
    State(state): State<AppState>,
    payload: ValidatedJson<V1MediaUsageQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let payload = payload.0;
    let media_ids = payload.media_ids;

    let usage_records = media_usage::Entity::find()
        .filter(media_usage::Column::MediaId.is_in(media_ids.clone()))
        .order_by_asc(media_usage::Column::MediaId)
        .order_by_asc(media_usage::Column::CreatedAt)
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?;

    let media_records = Media::find()
        .filter(media::Column::Id.is_in(media_ids.clone()))
        .all(&state.sea_db)
        .await
        .map_err(ErrorResponse::from)?
        .into_iter()
        .map(|media| (media.id, media))
        .collect::<HashMap<_, _>>();

    let mut post_ids = BTreeSet::new();
    let mut category_ids = BTreeSet::new();
    let mut user_ids = BTreeSet::new();

    for usage in &usage_records {
        match usage.entity_type {
            media_usage::EntityType::Post => {
                post_ids.insert(usage.entity_id);
            }
            media_usage::EntityType::Category => {
                category_ids.insert(usage.entity_id);
            }
            media_usage::EntityType::User => {
                user_ids.insert(usage.entity_id);
            }
        }
    }

    let post_map = if post_ids.is_empty() {
        HashMap::new()
    } else {
        post::Entity::find()
            .filter(post::Column::Id.is_in(post_ids.iter().copied().collect::<Vec<_>>()))
            .all(&state.sea_db)
            .await
            .map_err(ErrorResponse::from)?
            .into_iter()
            .map(|post| (post.id, post))
            .collect::<HashMap<_, _>>()
    };

    let category_map = if category_ids.is_empty() {
        HashMap::new()
    } else {
        category::Entity::find()
            .filter(category::Column::Id.is_in(category_ids.iter().copied().collect::<Vec<_>>()))
            .all(&state.sea_db)
            .await
            .map_err(ErrorResponse::from)?
            .into_iter()
            .map(|category| (category.id, category))
            .collect::<HashMap<_, _>>()
    };

    let user_map = if user_ids.is_empty() {
        HashMap::new()
    } else {
        user::Entity::find()
            .filter(user::Column::Id.is_in(user_ids.iter().copied().collect::<Vec<_>>()))
            .all(&state.sea_db)
            .await
            .map_err(ErrorResponse::from)?
            .into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>()
    };

    let mut usage_map: HashMap<i32, UsageAccumulator> = HashMap::new();

    for usage in usage_records {
        let media_usage::Model {
            id: usage_id,
            media_id,
            entity_type,
            entity_id,
            field_name,
            created_at,
        } = usage;

        let entry = usage_map.entry(media_id).or_default();

        match entity_type {
            media_usage::EntityType::Post => {
                let summary = post_map.get(&entity_id).map(PostSummary::from);
                entry.posts.push(PostUsage {
                    usage_id,
                    field_name,
                    created_at,
                    post: summary,
                });
            }
            media_usage::EntityType::Category => {
                let summary = category_map.get(&entity_id).map(CategorySummary::from);
                entry.categories.push(CategoryUsage {
                    usage_id,
                    field_name,
                    created_at,
                    category: summary,
                });
            }
            media_usage::EntityType::User => {
                let summary = user_map.get(&entity_id).map(UserSummary::from);
                entry.users.push(UserUsage {
                    usage_id,
                    field_name,
                    created_at,
                    user: summary,
                });
            }
        }
    }

    let mut response = Vec::new();
    for media_id in media_ids {
        let mut accumulator = usage_map.remove(&media_id).unwrap_or_default();
        accumulator.posts.sort_by_key(|item| item.usage_id);
        accumulator.categories.sort_by_key(|item| item.usage_id);
        accumulator.users.sort_by_key(|item| item.usage_id);

        response.push(MediaUsageGroup {
            media_id,
            media: media_records.get(&media_id).cloned(),
            posts: accumulator.posts,
            categories: accumulator.categories,
            users: accumulator.users,
        });
    }

    Ok((StatusCode::OK, Json(json!({ "data": response }))))
}

#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(media_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let uploader = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Authentication required to delete media")
    })?;

    let media = Media::find_by_id(&state.sea_db, media_id)
        .await?
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::FileNotFound).with_message("Media record not found")
        })?;

    if let Some(owner_id) = media.uploader_id {
        if owner_id != uploader.id {
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("You can only delete media you uploaded"));
        }
    }

    state
        .s3_client
        .delete_object()
        .bucket(&state.r2.bucket)
        .key(&media.object_key)
        .send()
        .await
        .map_err(|err| {
            ErrorResponse::new(ErrorCode::FileDeletionError)
                .with_message("Failed to delete media from storage")
                .with_details(err.to_string())
        })?;

    Media::delete_by_id(&state.sea_db, media_id).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Media deleted",
            "media_id": media.id,
        })),
    ))
}

fn infer_extension(filename: Option<&str>, mime_type: Option<&str>) -> Option<String> {
    if let Some(name) = filename {
        if let Some((_, ext)) = name.rsplit_once('.') {
            let ext = ext.trim().trim_matches('.');
            if !ext.is_empty() {
                return Some(ext.to_ascii_lowercase());
            }
        }
    }

    mime_type
        .and_then(|mt| mt.rsplit_once('/'))
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn build_object_key(extension: Option<&str>) -> String {
    let now = Utc::now();
    let prefix = format!("media/{}/{:02}", now.year(), now.month());
    let base = format!("{}/{}", prefix, Uuid::new_v4());

    match extension {
        Some(ext) => format!("{}.{}", base, ext),
        None => base,
    }
}

fn label_to_variant_type(label: &image_optimizer::VariantLabel) -> String {
    match label {
        image_optimizer::VariantLabel::Width(width) => format!("{}w", width),
        image_optimizer::VariantLabel::Lqip => "lqip".to_string(),
        image_optimizer::VariantLabel::Original => "original".to_string(),
    }
}
