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

impl IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.status).unwrap(), self.message).into_response()
    }
}
