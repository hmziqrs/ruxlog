use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(),
    info(
        title = "Ruxlog API",
        version = "1.0.0",
        description = "Ruxlog blog platform API"
    )
)]
pub struct ApiDoc;
