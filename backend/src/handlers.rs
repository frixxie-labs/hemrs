use anyhow::Result;
use axum::{extract::State, Json};
use log::info;

use sensors::Dht11;

use crate::create::Create;
use crate::db_connection_pool::Postgres;

use crate::error::HandlerError;
use crate::measurements::Measurement;
use crate::read::Read;

pub async fn store_env_data(
    State(pg_pool): State<Postgres>,
    Json(dht11_data): Json<Dht11>,
) -> Result<String, HandlerError> {
    info!("POST /");

    dht11_data
        .create(pg_pool)
        .await
        .map_err(|e| HandlerError::new(500, format!("Failed to store data in database: {}", e)))?;

    Ok("OK".to_string())
}

pub async fn fetch_latest_data(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Measurement>, HandlerError> {
    info!("GET api/latest");
    let entry = Measurement::read(pg_pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry))
}

pub async fn fetch_all_data(
    State(pool): State<Postgres>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    info!("GET api/all");
    let entry = Vec::<Measurement>::read(pool).await.map_err(|e| {
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry))
}
