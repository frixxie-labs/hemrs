use axum::{extract::State, Json};
use log::{info, warn};

use crate::{
    database::{
        create::Create, db_connection_pool::Postgres, delete::Delete, read::Read, update::Update,
    },
    sensors::Sensor,
};

use super::error::HandlerError;

pub async fn fetch_sensors(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    info!("GET api/sensors");
    let sensors = Vec::<Sensor>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}

pub async fn insert_sensor(
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    info!("POST api/sensors");
    sensor.create(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

pub async fn delete_sensor(
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    info!("DELETE api/sensors");
    sensor.delete(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

pub async fn update_sensor(
    State(pg_pool): State<Postgres>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    info!("DELETE api/sensors");
    sensor.update(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}
