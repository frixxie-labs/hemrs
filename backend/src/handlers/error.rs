use std::{error::Error, fmt};

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone)]
pub struct HandlerError {
    pub status: u16,
    pub message: String,
}

impl HandlerError {
    pub fn new(status: u16, message: String) -> Self {
        Self { status, message }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.status, self.message)
    }
}

impl Error for HandlerError {}

impl From<anyhow::Error> for HandlerError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            status: 500,
            message: format!("{err:#}"),
        }
    }
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.status).unwrap(), self.message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::HandlerError;

    #[quickcheck_macros::quickcheck]
    fn from_anyhow_always_produces_500(msg: String) -> bool {
        let err = anyhow::anyhow!("{}", msg);
        let handler_err = HandlerError::from(err);
        handler_err.status == 500
    }

    #[quickcheck_macros::quickcheck]
    fn display_contains_status_and_message(status: u16, message: String) -> bool {
        // Only use valid HTTP status codes to avoid panics in IntoResponse
        let status = (status % 500) + 100; // clamp to 100–599
        let err = HandlerError::new(status, message.clone());
        let displayed = format!("{err}");
        displayed.contains(&status.to_string()) && displayed.contains(&message)
    }

    #[quickcheck_macros::quickcheck]
    fn new_stores_fields_correctly(status: u16, message: String) -> bool {
        let status = (status % 500) + 100;
        let err = HandlerError::new(status, message.clone());
        err.status == status && err.message == message
    }
}
