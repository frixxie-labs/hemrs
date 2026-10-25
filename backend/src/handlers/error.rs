use std::{error::Error, fmt};

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum HandlerError {
    InvalidInput(&'static str),
    NotFound,
    Unavailable(&'static str),
    Internal(anyhow::Error),
}

impl HandlerError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn public_message(&self) -> &'static str {
        match self {
            Self::InvalidInput(message) | Self::Unavailable(message) => message,
            Self::NotFound => "Resource not found",
            Self::Internal(_) => "Internal server error",
        }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.status(), self.public_message())
    }
}

impl Error for HandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Internal(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(error: anyhow::Error) -> Self {
        match error.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::RowNotFound) => Self::NotFound,
            _ => Self::Internal(error),
        }
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
        if let Self::Internal(error) = &self {
            tracing::error!(error = ?error, "Request failed");
        }
        (self.status(), self.public_message()).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn internal_errors_do_not_expose_database_details() {
        let response =
            HandlerError::from(anyhow::anyhow!("secret database credentials")).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"Internal server error");
    }

    #[test]
    fn missing_rows_are_not_found_even_with_context() {
        let error = anyhow::Error::new(sqlx::Error::RowNotFound).context("Read failed");
        assert_eq!(
            HandlerError::from(error).into_response().status(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn expected_errors_have_specific_http_statuses() {
        assert_eq!(
            HandlerError::InvalidInput("Invalid input")
                .into_response()
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            HandlerError::Unavailable("Ingestion unavailable")
                .into_response()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
