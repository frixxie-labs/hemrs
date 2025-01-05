use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::{
    devices::{Devices, NewDevice},
    measurements::Measurement,
};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_devices(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Devices>>, HandlerError> {
    let devices = Devices::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(devices))
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
    Json(device): Json<Devices>,
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
    Json(device): Json<Devices>,
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

#[instrument]
pub async fn fetch_measurement_by_device_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let measurements = Measurement::read_by_device_id(device_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(measurements))
}

#[instrument]
pub async fn fetch_measurement_by_device_id_and_sensor_id(
    State(pool): State<PgPool>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let measurements = Measurement::read_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(measurements))
}
