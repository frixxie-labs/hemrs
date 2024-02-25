use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use log::info;
use sqlx::{query, query_as, Pool, Postgres};
use std::{error::Error, fmt};

use env_data::{EnvData, EnvDataEntry};

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

pub async fn store_env_data_entry(
    State(pool): State<Pool<Postgres>>,
    Json(env_data): Json<EnvDataEntry>,
) -> Result<String, HandlerError> {
    info!("POST /entry");
    query!(
        "INSERT INTO env_data VALUES ($1, $2, $3, $4)",
        env_data.ts,
        env_data.room,
        env_data.temperature,
        env_data.humidity
    )
    .execute(&pool)
    .await
    .map_err(|e| HandlerError::new(500, format!("Failed to store data in database: {}", e)))?;
    Ok("OK".to_string())
}

pub async fn store_env_data(
    State(pool): State<Pool<Postgres>>,
    Json(env_data): Json<EnvData>,
) -> Result<String, HandlerError> {
    info!("POST /");
    let env_data_entry: EnvDataEntry = env_data.into();
    query!(
        "INSERT INTO env_data VALUES ($1, $2, $3, $4)",
        env_data_entry.ts,
        env_data_entry.room,
        env_data_entry.temperature,
        env_data_entry.humidity
    )
    .execute(&pool)
    .await
    .map_err(|e| HandlerError::new(500, format!("Failed to store data in database: {}", e)))?;
    Ok("OK".to_string())
}

pub async fn fetch_all_data(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<EnvDataEntry>>, HandlerError> {
    info!("GET /");
    let rows = query_as!(EnvDataEntry, "SELECT * FROM env_data")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;

    Ok(Json(rows))
}
