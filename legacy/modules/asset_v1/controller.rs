use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;

use serde_json::json;

use uuid::Uuid;

use crate::{
    db::sea_models::asset::{AssetContext, Entity as Asset, NewAsset},
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::auth::AuthSession,
    AppState,
};

use super::validator::{V1AssetQueryParams, V1UpdateAssetPayload};

#[debug_handler]
pub async fn upload(
    State(state): State<AppState>,
    auth: AuthSession,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ErrorResponse> {
    let owner_id = match auth.user {
        Some(user) => user.id,
        None => {
            return Err(ErrorResponse::new(ErrorCode::Unauthorized)
                .with_message("Authentication required for file upload"))
        }
    };
    let mut payload = NewAsset {
        owner_id: Some(owner_id),
        file_url: String::new(),
        file_name: None,
        mime_type: None,
        size: None,
        context: None,
    };

    let mut file_data = None;

    // Process the multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ErrorResponse::new(ErrorCode::ValidationError)
            .with_message(&format!("Failed to process form: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                payload.file_name = field.file_name().map(|s| s.to_string());
                payload.mime_type = field.content_type().map(|s| s.to_string());
                let data = field.bytes().await.map_err(|e| {
                    ErrorResponse::new(ErrorCode::ValidationError)
                        .with_message(&format!("Failed to read file data: {}", e))
                })?;

                payload.size = Some(data.len() as i32);
                file_data = Some(data);
            }
            "context" => {
                let text = field.text().await.map_err(|e| {
                    ErrorResponse::new(ErrorCode::ValidationError)
                        .with_message(&format!("Failed to read context: {}", e))
                })?;
                let ctx = AssetContext::from_str(&text).map_err(|_| {
                    ErrorResponse::new(ErrorCode::ValidationError)
                        .with_message("Invalid asset context")
                })?;
                payload.context = Some(ctx);
            }
            _ => {}
        }
    }

    let file_data = file_data.ok_or_else(|| {
        ErrorResponse::new(ErrorCode::MissingRequiredField).with_message("No file provided")
    })?;

    let file_name = payload.file_name.as_ref().ok_or_else(|| {
        ErrorResponse::new(ErrorCode::MissingRequiredField).with_message("No filename provided")
    })?;

    if file_data.len() > 10 * 1024 * 1024 {
        return Err(ErrorResponse::new(ErrorCode::FileTooLarge)
            .with_message("File size exceeds the maximum allowed size of 10MB"));
    }

    if let Some(mime_type) = &payload.mime_type {
        let allowed_types = [
            "image/jpeg",
            "image/png",
            "image/gif",
            "image/webp",
            "application/pdf",
            "text/plain",
            "application/zip",
        ];

        if !allowed_types.contains(&mime_type.as_str()) {
            return Err(ErrorResponse::new(ErrorCode::InvalidFileType).with_message(
                "Unsupported file type. Allowed types: JPEG, PNG, GIF, WEBP, PDF, TXT, ZIP",
            ));
        }
    }

    let extension = match file_name.split('.').last() {
        Some(ext) => format!(".{}", ext),
        None => String::new(),
    };

    let unique_filename = format!("{}{}", Uuid::new_v4(), extension);

    println!("Unique filename: {}", unique_filename);

    let byte_stream = ByteStream::from(file_data);

    let content_type = payload
        .mime_type
        .as_deref()
        .unwrap_or("application/octet-stream");

    match state
        .s3_client
        .put_object()
        .bucket(&state.r2.bucket)
        .key(&unique_filename)
        .body(byte_stream)
        .content_type(content_type)
        .send()
        .await
    {
        Ok(_) => {
            let file_url = format!("{}/{}", state.r2.public_url, unique_filename);

            payload.file_url = file_url.clone();

            match Asset::create(&state.sea_db, payload).await {
                Ok(result) => Ok((StatusCode::CREATED, Json(json!(result)))),
                Err(err) => Err(ErrorResponse::new(ErrorCode::AssetMetadataError)
                    .with_message(&format!("Failed to save asset metadata: {}", err))),
            }
        }
        Err(e) => {
            println!("Error uploading to R2: {:?}", e);
            println!("Error uploading to R2: {:?}", e.raw_response());
            Err(ErrorResponse::new(ErrorCode::StorageError)
                .with_message(&format!("Failed to upload file to R2: {}", e)))
        }
    }
}

/// Update an existing asset
#[debug_handler]
pub async fn update(
    State(state): State<AppState>,
    // auth: AuthSession,
    Path(asset_id): Path<i32>,
    payload: ValidatedJson<V1UpdateAssetPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let update_asset = payload.0.into_update_asset();

    match Asset::update(&state.sea_db, asset_id, update_asset).await {
        Ok(Some(asset)) => Ok((StatusCode::OK, Json(json!(asset)))),
        Ok(None) => {
            Err(ErrorResponse::new(ErrorCode::FileNotFound).with_message("Asset does not exist"))
        }
        Err(err) => Err(ErrorResponse::new(ErrorCode::AssetMetadataError)
            .with_message(&format!("Failed to update asset metadata: {}", err))),
    }
}

/// Delete an asset from R2 and the database
#[debug_handler]
pub async fn delete(
    State(state): State<AppState>,
    // auth: AuthSession,
    Path(asset_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let asset = match Asset::find_by_id_or_filename(&state.sea_db, Some(asset_id), None).await {
        Ok(Some(asset)) => asset,
        Ok(None) => {
            return Err(ErrorResponse::new(ErrorCode::FileNotFound).with_message("Asset not found"));
        }
        Err(err) => {
            return Err(ErrorResponse::new(ErrorCode::QueryError)
                .with_message(&format!("Database error: {}", err)))
        }
    };

    let file_name = asset.file_url.split('/').last().ok_or_else(|| {
        ErrorResponse::new(ErrorCode::InvalidValue).with_message("Invalid file URL")
    })?;

    state
        .s3_client
        .delete_object()
        .bucket(&state.r2.bucket)
        .key(file_name)
        .send()
        .await
        .map_err(|e| {
            ErrorResponse::new(ErrorCode::FileDeletionError)
                .with_message(&format!("Failed to delete file from storage: {}", e))
        })?;

    match Asset::delete(&state.sea_db, asset_id).await {
        Ok(1) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Asset deleted successfully" })),
        )),
        Ok(0) => {
            Err(ErrorResponse::new(ErrorCode::FileNotFound).with_message("Asset does not exist"))
        }
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({ "message": "Asset deleted successfully" })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::QueryError)
            .with_message(&format!("Failed to delete asset record: {}", err))),
    }
}

/// Find an asset by ID
#[debug_handler]
pub async fn find_by_id(
    State(state): State<AppState>,
    Path(asset_id): Path<i32>,
) -> Result<impl IntoResponse, ErrorResponse> {
    match Asset::find_by_id_or_filename(&state.sea_db, Some(asset_id), None).await {
        Ok(Some(asset)) => Ok((StatusCode::OK, Json(json!(asset)))),
        Ok(None) => {
            Err(ErrorResponse::new(ErrorCode::FileNotFound).with_message("Asset not found"))
        }
        Err(err) => Err(ErrorResponse::new(ErrorCode::QueryError)
            .with_message(&format!("Database error: {}", err))),
    }
}

/// Find assets with query parameters
#[debug_handler]
pub async fn find_with_query(
    State(state): State<AppState>,
    payload: ValidatedJson<V1AssetQueryParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let asset_query = payload.0.into_asset_query();
    let page = asset_query.page_no.unwrap_or(1);

    match Asset::find_with_query(&state.sea_db, asset_query).await {
        Ok((assets, total)) => Ok((
            StatusCode::OK,
            Json(json!({
                "data": assets,
                "total": total,
                "per_page": Asset::PER_PAGE,
                "page": page,
            })),
        )),
        Err(err) => Err(ErrorResponse::new(ErrorCode::QueryError)
            .with_message(&format!("Failed to query assets: {}", err))),
    }
}

#[debug_handler]
pub async fn contexts(State(_state): State<AppState>) -> Result<impl IntoResponse, ErrorResponse> {
    let contexts = vec![
        AssetContext::UserAvatar.as_str(),
        AssetContext::CategoryCover.as_str(),
        AssetContext::CategoryLogo.as_str(),
        AssetContext::PostFeatured.as_str(),
        AssetContext::PostInline.as_str(),
    ];
    Ok((StatusCode::OK, Json(json!(contexts))))
}
