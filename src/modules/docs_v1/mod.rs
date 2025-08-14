use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{error::ErrorResponse, AppState};

/// Documentation module (`/docs` + `/docs/openapi.json`)
/// Minimal OpenAPI exposure without external codegen deps (keeps "no new deps" rule).
/// Extend paths incrementally; keep spec lean for personal blog scope.
#[debug_handler]
pub async fn openapi_json(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // NOTE: Hand‑crafted minimal spec (expand as endpoints stabilize)
    let spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Ruxlog Blog API",
            "version": "1.0.0",
            "description": "Minimal OpenAPI document (manually curated)."
        },
        "paths": {
            "/post/v1/view/{id_or_slug}": {
                "post": {
                    "summary": "Fetch a single post by ID or slug",
                    "parameters": [
                        {
                            "name": "id_or_slug",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": { "description": "Post found" },
                        "404": { "description": "Post not found" }
                    }
                }
            },
            "/post/v1/list/published": {
                "post": {
                    "summary": "List published posts (paginated defaults server-side)",
                    "responses": {
                        "200": { "description": "List returned" }
                    }
                }
            },
            "/auth/v1/register": {
                "post": {
                    "summary": "Register new user",
                    "responses": {
                        "201": { "description": "User created" },
                        "400": { "description": "Validation error" }
                    }
                }
            },
            "/asset/v1/upload": {
                "post": {
                    "summary": "Upload an asset (multipart)",
                    "responses": {
                        "201": { "description": "Asset stored" },
                        "400": { "description": "Invalid file" }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "ErrorResponse": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "example": "DB_002" },
                        "status": { "type": "integer", "example": 404 },
                        "message": { "type": "string", "example": "The requested record was not found" },
                        "details": { "type": "string" },
                        "context": { "type": "object" },
                        "requestId": { "type": "string" }
                    }
                }
            }
        }
    });

    Ok((StatusCode::OK, Json(json!(spec))))
}

#[debug_handler]
pub async fn swagger_ui(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Serve a lightweight Swagger UI HTML (CDN assets).
    const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<title>Ruxlog API Docs</title>
<link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"/>
<style>
body { margin:0; background:#0f1115; }
.topbar { display:none; }
</style>
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>
window.onload = () => {
  SwaggerUIBundle({
    url: '/docs/openapi.json',
    dom_id: '#swagger-ui',
    presets: [SwaggerUIBundle.presets.apis],
    layout: 'BaseLayout'
  });
};
</script>
</body>
</html>"#;

    Ok((StatusCode::OK, Html(HTML)))
}
