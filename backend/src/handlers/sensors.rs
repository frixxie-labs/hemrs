use axum::{extract::State, Json};
use tracing::{instrument, warn};

use crate::{
    database::db_connection_pool::{DbConnectionPool, Postgres},
    sensors::{NewSensor, Sensors},
};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_sensors(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Sensors>>, HandlerError> {
    let pool = pg_pool.get_connection().await;
    let sensors = Sensors::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}

#[instrument]
pub async fn insert_sensor(
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<NewSensor>,
) -> Result<String, HandlerError> {
    let pool = pg_pool.get_connection().await;
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
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<Sensors>,
) -> Result<String, HandlerError> {
    let pool = pg_pool.get_connection().await;
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
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<Sensors>,
) -> Result<String, HandlerError> {
    let pool = pg_pool.get_connection().await;
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.update(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}
