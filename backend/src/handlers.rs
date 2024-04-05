use anyhow::Result;
use axum::{extract::State, Json};
use log::{info, warn};

use sensors::Dht11;

use crate::create::Create;
use crate::db_connection_pool::Postgres;

use crate::devices::Device;
use crate::error::HandlerError;
use crate::measurements::Measurement;
use crate::read::Read;
use crate::sensors::Sensor;

pub async fn fetch_devices(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Device>>, HandlerError> {
    info!("GET api/devices");
    let devices = Vec::<Device>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(devices))
}

pub async fn fetch_sensors(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    info!("GET api/devices");
    let sensors = Vec::<Sensor>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}

pub async fn store_measurements(
    State(pg_pool): State<Postgres>,
    Json(dht11_data): Json<Dht11>,
) -> Result<String, HandlerError> {
    info!("POST /");

    dht11_data.create(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;

    Ok("OK".to_string())
}

pub async fn fetch_latest_measurement(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Measurement>, HandlerError> {
    info!("GET api/measurements/latest");
    let entry = Measurement::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry))
}

pub async fn fetch_all_measurements(
    State(pool): State<Postgres>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    info!("GET api/measurements/all");
    let entry = Vec::<Measurement>::read(pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry))
}
