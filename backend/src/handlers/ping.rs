use axum::Json;
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;

use super::error::HandlerError;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_return_ping_response() {
        let result = ping().await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.status, "ok");
        assert!(!response.timestamp.is_empty());

        // Verify timestamp is in RFC3339 format (basic check)
        assert!(response.timestamp.contains('T'));
        assert!(response.timestamp.contains('Z') || response.timestamp.contains('+'));
    }
}

#[derive(Serialize, ToSchema)]
pub struct PingResponse {
    pub status: String,
    pub timestamp: String,
}

#[utoipa::path(
    get,
    path = "/status/ping",
    responses(
        (status = 200, description = "Health check successful", body = PingResponse),
    )
)]
#[instrument]
pub async fn ping() -> Result<Json<PingResponse>, HandlerError> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let response = PingResponse {
        status: "ok".to_string(),
        timestamp,
    };
    Ok(Json(response))
}
