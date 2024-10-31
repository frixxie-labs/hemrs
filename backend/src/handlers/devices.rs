use axum::{extract::State, Json};
use tracing::{instrument, warn};

use crate::{
    database::{
        db_connection_pool::Postgres, delete::Delete, insert::Insert, read::Read, update::Update,
    },
    devices::{Device, Devices},
};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_devices(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Devices>>, HandlerError> {
    let devices = Vec::<Devices>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(devices))
}

#[instrument]
pub async fn insert_device(
    State(pg_pool): State<Postgres>,
    Json(device): Json<Device>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.insert(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn delete_device(
    State(pg_pool): State<Postgres>,
    Json(device): Json<Devices>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.delete(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn update_device(
    State(pg_pool): State<Postgres>,
    Json(device): Json<Devices>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.update(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}
