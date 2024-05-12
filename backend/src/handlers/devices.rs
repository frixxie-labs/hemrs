use axum::{extract::State, Json};
use log::{info, warn};

use crate::{
    database::{
        insert::Insert, db_connection_pool::Postgres, delete::Delete, read::Read, update::Update,
    },
    devices::{Device, Devices},
};

use super::error::HandlerError;

pub async fn fetch_devices(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Devices>>, HandlerError> {
    info!("GET api/devices");
    let devices = Vec::<Devices>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(devices))
}

pub async fn insert_device(
    State(pg_pool): State<Postgres>,
    Json(device): Json<Device>,
) -> Result<String, HandlerError> {
    info!("POST api/devices");
    device.insert(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

pub async fn delete_device(
    State(pg_pool): State<Postgres>,
    Json(device): Json<Devices>,
) -> Result<String, HandlerError> {
    info!("DELETE api/devices");
    device.delete(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

pub async fn update_device(
    State(pg_pool): State<Postgres>,
    Json(device): Json<Devices>,
) -> Result<String, HandlerError> {
    info!("DELETE api/devices");
    device.update(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}
