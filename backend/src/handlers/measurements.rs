use axum::{
    extract::{Path, State},
    Json,
};
use metrics::counter;
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::measurements::{Measurement, NewMeasurements};

use super::error::HandlerError;

#[instrument]
pub async fn store_measurements(
    State(pool): State<PgPool>,
    Json(measurement): Json<NewMeasurements>,
) -> Result<String, HandlerError> {
    match measurement {
        NewMeasurements::Temperature(new_temperature) => {
            new_temperature.insert(&pool).await.map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to insert data into database: {}", e))
            })?;
        }
        NewMeasurements::Dht11(dht11) => {
            dht11.insert(&pool).await.map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to insert data into database: {}", e))
            })?;
        }
        NewMeasurements::Measurement(new_measurement) => {
            new_measurement.insert(&pool).await.map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to insert data into database: {}", e))
            })?;
        }
    };

    counter!("new_measurements").increment(1);

    Ok("OK".to_string())
}

#[instrument]
pub async fn fetch_latest_measurement(
    State(pool): State<PgPool>,
) -> Result<Json<Measurement>, HandlerError> {
    let entry = Measurement::read_latest(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(entry))
}

#[instrument]
pub async fn fetch_measurements_count(
    State(pool): State<PgPool>,
) -> Result<Json<usize>, HandlerError> {
    let entry = Measurement::read_all(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry.len()))
}

#[instrument]
pub async fn fetch_all_measurements(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let entries = Measurement::read_all(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entries))
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
pub async fn fetch_latest_measurement_by_device_id_and_sensor_id(
    State(pool): State<PgPool>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Measurement>, HandlerError> {
    let measurement =
        Measurement::read_latest_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
            .await
            .map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
            })?;
    Ok(Json(measurement))
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
