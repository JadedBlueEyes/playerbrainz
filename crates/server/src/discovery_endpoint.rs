use axum::Json;

#[derive(serde::Serialize)]
pub struct DiscoveryResponse {
    graph_endpoint: String,
}

pub async fn discovery() -> Json<DiscoveryResponse> {
    let graph_endpoint = "/graphql".to_string();
    Json(DiscoveryResponse { graph_endpoint })
}
