use axum::{extract::State, Json};
use tracing::{instrument, warn};

use crate::{
    database::{
        db_connection_pool::Postgres, delete::Delete, insert::Insert, read::Read, update::Update,
    },
    sensors::{Sensor, Sensors},
};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_sensors(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Sensors>>, HandlerError> {
    let sensors = Vec::<Sensors>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}

#[instrument]
pub async fn insert_sensor(
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.insert(pg_pool).await.map_err(|e| {
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
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.delete(pg_pool).await.map_err(|e| {
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
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.update(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}
