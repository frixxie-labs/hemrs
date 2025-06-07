use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use metrics::counter;
use moka::future::Cache;
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::{
    devices::Devices,
    measurements::{Measurement, MeasurementStats, NewMeasurement, NewMeasurements},
    sensors::Sensors,
};

use super::error::HandlerError;

async fn insert_measurement(
    measurement: NewMeasurement,
    pool: &PgPool,
    cache: &Cache<(i32, i32), Measurement>,
) -> Result<(), HandlerError> {
    let device = Devices::read_by_id(pool, measurement.device)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch device from database: {}", e))
        })?;
    let sensor = Sensors::read_by_id(pool, measurement.sensor)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch sensor from database: {}", e))
        })?;
    let ts = measurement.timestamp.unwrap_or_else(|| chrono::Utc::now());
    let entry = Measurement {
        value: measurement.measurement.clone(),
        timestamp: ts,
        device_name: device.name,
        device_location: device.location,
        sensor_name: sensor.name,
        unit: sensor.unit,
    };
    cache.insert((device.id, sensor.id), entry.clone()).await;
    measurement.insert(pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to insert data into database: {}", e))
    })?;
    Ok(())
}

#[instrument]
pub async fn store_measurements(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
    Json(measurement): Json<NewMeasurements>,
) -> Result<Response, HandlerError>
where
    Response: IntoResponse,
{
    let (pool, cache) = app_state;
    match measurement {
        NewMeasurements::Measurement(new_measurement) => {
            insert_measurement(new_measurement, &pool, &cache).await?;
            counter!("new_measurements").increment(1);
        }
        NewMeasurements::Measurements(new_measurements) => {
            for measurement in new_measurements {
                insert_measurement(measurement, &pool, &cache).await?;
                counter!("new_measurements").increment(1);
            }
        }
    };
    let resp = Response::builder()
        .status(201)
        .body("Measurement(s) inserted successfully".into())
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to build response: {}", e))
        })?;
    Ok(resp)
}

#[instrument]
pub async fn fetch_latest_measurement(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
) -> Result<Json<Measurement>, HandlerError> {
    let (pool, _cache) = app_state;
    let entry = Measurement::read_latest(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(entry))
}

#[instrument]
pub async fn fetch_measurements_count(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
) -> Result<Json<usize>, HandlerError> {
    let (pool, _cache) = app_state;
    let count = Measurement::read_total_measurements(&pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(count as usize))
}

#[instrument]
pub async fn fetch_all_measurements(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let entries = Measurement::read_all(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entries))
}

#[instrument]
pub async fn fetch_measurement_by_device_id(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
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
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Measurement>, HandlerError> {
    let (pool, cache) = app_state;
    // Check cache first
    if let Some(measurement) = cache.get(&(device_id, sensor_id)).await {
        return Ok(Json(measurement));
    }
    let measurement =
        Measurement::read_latest_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
            .await
            .map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
            })?;
    // Insert into cache
    cache
        .insert((device_id, sensor_id), measurement.clone())
        .await;
    Ok(Json(measurement))
}

#[instrument]
pub async fn fetch_measurement_by_device_id_and_sensor_id(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(measurements))
}

#[instrument]
pub async fn fetch_all_latest_measurements(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_all_latest_measurements(&pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    // Insert all latest measurements into cache
    Ok(Json(measurements))
}

#[instrument]
pub async fn fetch_stats_by_device_id_and_sensor_id(
    State(app_state): State<(PgPool, Cache<(i32, i32), Measurement>)>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<MeasurementStats>, HandlerError> {
    let (pool, _cache) = app_state;
    let stats = Measurement::read_stats_by_device_id_and_sensor_id(&pool, device_id, sensor_id)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(stats))
}

#[cfg(test)]

mod tests {
    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    use super::*;

    #[sqlx::test]
    async fn should_store_single_measurement_without_ts(db: PgPool) {
        let cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(60))
            .build();
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurement = NewMeasurement::new(None, 1, 1, 1.0);
        let result = store_measurements(
            State((db, cache)),
            Json(NewMeasurements::Measurement(new_measurement)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }

    #[sqlx::test]
    async fn should_store_single_measurement_with_ts(db: PgPool) {
        let cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(60))
            .build();
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurement = NewMeasurement::new(Some(chrono::Utc::now()), 1, 1, 1.0);
        let result = store_measurements(
            State((db, cache)),
            Json(NewMeasurements::Measurement(new_measurement)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }

    #[sqlx::test]
    async fn should_store_multiple_measurements(db: PgPool) {
        let cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(60))
            .build();
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurements = vec![
            NewMeasurement::new(None, 1, 1, 1.0),
            NewMeasurement::new(None, 1, 1, 2.0),
        ];
        let result = store_measurements(
            State((db, cache)),
            Json(NewMeasurements::Measurements(new_measurements)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }

    #[sqlx::test]
    async fn should_store_multiple_measurements_with_and_without_ts(db: PgPool) {
        let cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(60))
            .build();
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurements = vec![
            NewMeasurement::new(None, 1, 1, 1.0),
            NewMeasurement::new(Some(chrono::Utc::now()), 1, 1, 2.0),
        ];
        let result = store_measurements(
            State((db, cache)),
            Json(NewMeasurements::Measurements(new_measurements)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }
}
