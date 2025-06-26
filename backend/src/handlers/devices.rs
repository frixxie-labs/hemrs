use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::devices::{Device, NewDevice};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_devices(State(pool): State<PgPool>) -> Result<Json<Vec<Device>>, HandlerError> {
    let devices = Device::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(devices))
}

#[instrument]
pub async fn fetch_devices_by_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Device>, HandlerError> {
    let device = Device::read_by_id(&pool, device_id).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(device))
}

#[instrument]
pub async fn insert_device(
    State(pool): State<PgPool>,
    Json(device): Json<NewDevice>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.insert(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn delete_device(
    State(pool): State<PgPool>,
    Json(device): Json<Device>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.delete(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn update_device(
    State(pool): State<PgPool>,
    Json(device): Json<Device>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.update(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}
