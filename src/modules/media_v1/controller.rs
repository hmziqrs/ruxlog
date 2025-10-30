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
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::sea_models::media::{Entity as Media, NewMedia},
    error::{ErrorCode, ErrorResponse},
    extractors::{ValidatedJson, ValidatedMultipart},
    services::{auth::AuthSession, image_optimizer},
    AppState,
};
use tracing::warn;

use super::validator::{MediaUploadMetadata, V1MediaListQuery};

const MAX_UPLOAD_SIZE_BYTES: usize = 20 * 1024 * 1024; // 20MiB ceiling

#[debug_handler]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    mut multipart: ValidatedMultipart,
) -> Result<impl IntoResponse, ErrorResponse> {
    let uploader = auth.user.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Authentication required to upload media")
    })?;

    let mut metadata = MediaUploadMetadata::default();
    let mut file_bytes: Option<Bytes> = None;
    let mut original_name: Option<String> = None;
    let mut mime_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        ErrorResponse::new(ErrorCode::ValidationError).with_details(err.to_string())
    })? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            original_name = field.file_name().map(|name| name.to_string());
            mime_type = field.content_type().map(|ty| ty.to_string());
            let bytes = field.bytes().await.map_err(|err| {
                ErrorResponse::new(ErrorCode::FileUploadError)
                    .with_message("Failed to read uploaded file")
                    .with_details(err.to_string())
            })?;

            if bytes.len() > MAX_UPLOAD_SIZE_BYTES {
                return Err(ErrorResponse::new(ErrorCode::FileTooLarge)
                    .with_message("File size exceeds the 20MiB upload limit"));
            }

            file_bytes = Some(bytes);
        } else {
            let value = field.text().await.map_err(|err| {
                ErrorResponse::new(ErrorCode::InvalidFormat)
                    .with_message("Failed to read accompanying form field")
                    .with_details(err.to_string())
            })?;

            metadata
                .apply_field(&field_name, &value)
                .map_err(|msg| ErrorResponse::new(ErrorCode::InvalidValue).with_message(&msg))?;
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::MissingRequiredField).with_message("Missing file field")
    })?;

    // Derive useful metadata if it was not supplied
    if metadata.width.is_none() || metadata.height.is_none() {
        if let Ok(dimensions) = imagesize::blob_size(&file_bytes) {
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
    };

    let stored = Media::create(&state.sea_db, new_media).await?;

    Ok((StatusCode::CREATED, Json(json!(stored))))
}

/// List media with pagination and filtering
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
