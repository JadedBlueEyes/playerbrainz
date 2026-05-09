use crate::error::AppError;
use axum::Json;

#[derive(serde::Serialize)]
pub struct DiscoveryResponse {
    graph_endpoint: String,
}

pub async fn discovery() -> Result<Json<DiscoveryResponse>, AppError> {
    let graph_endpoint = "/graphql".to_string();
    Ok(Json(DiscoveryResponse { graph_endpoint }))
}
