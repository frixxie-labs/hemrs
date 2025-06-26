use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::sensors::{NewSensor, Sensor};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_sensors(State(pool): State<PgPool>) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}

#[instrument]
pub async fn fetch_sensor_by_sensor_id(
    State(pool): State<PgPool>,
    Path(sensor_id): Path<i32>,
) -> Result<Json<Sensor>, HandlerError> {
    let sensor = Sensor::read_by_id(&pool, sensor_id).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensor))
}

#[instrument]
pub async fn insert_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<NewSensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.insert(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn delete_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.delete(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn update_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.update(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn fetch_sensors_by_device_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read_by_device_id(&pool, device_id)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(sensors))
}
