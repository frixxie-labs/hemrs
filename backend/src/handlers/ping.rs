use axum::Json;
use serde::Serialize;
use tracing::instrument;

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

/// Response structure for the ping endpoint.
#[derive(Serialize)]
pub struct PingResponse {
    pub status: String,
    pub timestamp: String,
}

/// HTTP handler for the ping/status endpoint.
///
/// This endpoint provides a simple health check for the service.
/// It returns a JSON response indicating the service is running and includes a timestamp.
///
/// # Returns
///
/// JSON response with status "ok" and current timestamp.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON with status and timestamp
#[instrument]
pub async fn ping() -> Result<Json<PingResponse>, HandlerError> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let response = PingResponse {
        status: "ok".to_string(),
        timestamp,
    };
    Ok(Json(response))
}