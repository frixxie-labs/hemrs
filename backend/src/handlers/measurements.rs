use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use moka::future::Cache;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tracing::{instrument, warn};

use crate::measurements::{Measurement, MeasurementStats, NewMeasurement, NewMeasurements};

use super::error::HandlerError;

type ApplicationState = State<(PgPool, Cache<(i32, i32), Measurement>)>;

/// HTTP handler to store new measurements asynchronously.
///
/// This endpoint accepts either single measurements or batches of measurements
/// and queues them for asynchronous insertion via a background thread. This
/// approach prevents blocking the HTTP response while ensuring data persistence.
///
/// # Arguments
///
/// * `tx` - Channel sender for asynchronous measurement processing
/// * `measurement` - Single measurement or batch of measurements from JSON body
///
/// # Returns
///
/// HTTP 201 response on success, or error if channel communication fails.
///
/// # HTTP Response
///
/// - `201 Created` - Measurements queued successfully for insertion
/// - `500 Internal Server Error` - Channel communication error
///
/// # JSON Format
///
/// Single measurement:
/// ```json
/// {
///   "timestamp": "2023-01-01T12:00:00Z", // optional
///   "device": 1,
///   "sensor": 2,
///   "measurement": 23.5
/// }
/// ```
///
/// Multiple measurements:
/// ```json
/// [
///   {"device": 1, "sensor": 2, "measurement": 23.5},
///   {"device": 1, "sensor": 3, "measurement": 45.2}
/// ]
/// ```
#[instrument]
pub async fn store_measurements(
    State(tx): State<Sender<NewMeasurement>>,
    Json(measurement): Json<NewMeasurements>,
) -> Result<Response, HandlerError>
where
    Response: IntoResponse,
{
    match measurement {
        NewMeasurements::Measurement(new_measurement) => {
            tx.send(new_measurement).await.map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(
                    500,
                    format!("Failed to send measurement to background thread: {e}"),
                )
            })?;
        }
        NewMeasurements::Measurements(new_measurements) => {
            for measurement in new_measurements {
                tx.send(measurement).await.map_err(|e| {
                    warn!("Failed with error: {}", e);
                    HandlerError::new(
                        500,
                        format!("Failed to send measurement to background thread: {e}"),
                    )
                })?;
            }
        }
    };

    let resp = Response::builder()
        .status(201)
        .body("Measurement(s) inserted successfully".into())
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to build response: {e}"))
        })?;

    Ok(resp)
}

/// HTTP handler to retrieve the most recent measurement from any device.
///
/// This endpoint returns the latest single measurement across the entire system,
/// useful for system health monitoring or "last activity" displays.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
///
/// # Returns
///
/// JSON representation of the latest measurement, or error if none exist or database fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON representation of the latest measurement
/// - `500 Internal Server Error` - No measurements exist or database error
#[instrument]
pub async fn fetch_latest_measurement(
    State(app_state): ApplicationState,
) -> Result<Json<Measurement>, HandlerError> {
    let (pool, _cache) = app_state;

    let entry = Measurement::read_latest(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;

    Ok(Json(entry))
}

/// HTTP handler to get the total count of measurements in the system.
///
/// This endpoint returns the total number of measurement records across
/// all devices and sensors. Useful for system statistics and monitoring dashboards.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
///
/// # Returns
///
/// JSON number representing the total measurement count, or error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON number with total count
/// - `500 Internal Server Error` - Database error occurred
#[instrument]
pub async fn fetch_measurements_count(
    State(app_state): ApplicationState,
) -> Result<Json<usize>, HandlerError> {
    let (pool, _cache) = app_state;
    let count = Measurement::read_total_measurements(&pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
        })?;
    Ok(Json(count as usize))
}

/// HTTP handler to retrieve all measurements in the system.
///
/// This endpoint fetches every measurement record from all devices and sensors,
/// ordered chronologically. This can return large amounts of data and should be
/// used with caution in production environments.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
///
/// # Returns
///
/// JSON array of all measurements, or error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of all measurements
/// - `500 Internal Server Error` - Database error occurred
///
/// # Performance Warning
///
/// This endpoint can return large datasets. Consider using more specific
/// endpoints for better performance in production environments.
#[instrument]
pub async fn fetch_all_measurements(
    State(app_state): ApplicationState,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let entries = Measurement::read_all(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;

    Ok(Json(entries))
}

/// HTTP handler to retrieve all measurements from a specific device.
///
/// This endpoint fetches all measurements reported by the specified device
/// across all its sensors, ordered chronologically. Useful for device-level
/// monitoring and analysis.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
/// * `device_id` - Device ID extracted from URL path parameter
///
/// # Returns
///
/// JSON array of measurements from the device, or error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of measurements from the device
/// - `500 Internal Server Error` - Database error or device not found
#[instrument]
pub async fn fetch_measurement_by_device_id(
    State(app_state): ApplicationState,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_by_device_id(device_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
        })?;
    Ok(Json(measurements))
}

/// HTTP handler to retrieve the latest measurement for a specific device-sensor pair.
///
/// This endpoint returns the most recent measurement from the specified device-sensor
/// combination. Uses caching with 60s TTL to improve performance for frequently
/// accessed measurements. Cache misses will query the database and update the cache.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
/// * `device_id` - Device ID extracted from URL path parameter
/// * `sensor_id` - Sensor ID extracted from URL path parameter
///
/// # Returns
///
/// JSON representation of the latest measurement, or error if none exist or database fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON representation of the latest measurement
/// - `500 Internal Server Error` - No measurements exist for this combination or database error
///
/// # Caching
///
/// Results are cached for 60 seconds to improve performance. Cache key is (device_id, sensor_id).
#[instrument]
pub async fn fetch_latest_measurement_by_device_id_and_sensor_id(
    State(app_state): ApplicationState,
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
                HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
            })?;
    // Insert into cache
    cache
        .insert((device_id, sensor_id), measurement.clone())
        .await;
    Ok(Json(measurement))
}

/// HTTP handler to retrieve all measurements for a specific device-sensor pair.
///
/// This endpoint fetches the complete time series of measurements for the
/// specified device-sensor combination, ordered chronologically. Useful for
/// creating detailed charts and historical analysis.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
/// * `device_id` - Device ID extracted from URL path parameter
/// * `sensor_id` - Sensor ID extracted from URL path parameter
///
/// # Returns
///
/// JSON array of measurements for the device-sensor pair, or error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of measurements ordered chronologically
/// - `500 Internal Server Error` - Database error or device/sensor not found
#[instrument]
pub async fn fetch_measurement_by_device_id_and_sensor_id(
    State(app_state): ApplicationState,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
        })?;
    Ok(Json(measurements))
}

/// HTTP handler to retrieve the latest measurement for each device-sensor combination.
///
/// This endpoint uses DISTINCT ON to get only the most recent measurement
/// for each unique device-sensor pair across the entire system. Perfect for
/// dashboard views showing current status of all sensors.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
///
/// # Returns
///
/// JSON array of the latest measurements for each device-sensor pair.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of latest measurements for all device-sensor pairs
/// - `500 Internal Server Error` - Database error occurred
///
/// # Note
///
/// This endpoint is optimized for dashboard displays where you need to see
/// the current value for every sensor in the system.
#[instrument]
pub async fn fetch_all_latest_measurements(
    State(app_state): ApplicationState,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_all_latest_measurements(&pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
        })?;
    // Insert all latest measurements into cache
    Ok(Json(measurements))
}

/// HTTP handler to get statistical summary for a specific device-sensor combination.
///
/// This endpoint calculates and returns aggregate statistics (min, max, count, average,
/// standard deviation, variance) for all measurements from the specified device-sensor pair.
/// Useful for monitoring data quality and detecting anomalies.
///
/// # Arguments
///
/// * `app_state` - Application state containing database pool and cache
/// * `device_id` - Device ID extracted from URL path parameter
/// * `sensor_id` - Sensor ID extracted from URL path parameter
///
/// # Returns
///
/// JSON object containing statistical summary, or error if no data exists or database fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON object with statistical summary
/// - `500 Internal Server Error` - No measurements exist for this combination or database error
///
/// # JSON Response Format
///
/// ```json
/// {
///   "min": 10.5,
///   "max": 35.2,
///   "count": 1000,
///   "avg": 22.8,
///   "stddev": 5.4,
///   "variance": 29.2
/// }
/// ```
#[instrument]
pub async fn fetch_stats_by_device_id_and_sensor_id(
    State(app_state): ApplicationState,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<MeasurementStats>, HandlerError> {
    let (pool, _cache) = app_state;
    let stats = Measurement::read_stats_by_device_id_and_sensor_id(&pool, device_id, sensor_id)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
        })?;
    Ok(Json(stats))
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::Receiver;

    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    use super::*;

    #[sqlx::test]
    async fn should_store_single_measurement_without_ts(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurement = NewMeasurement::new(None, 1, 1, 1.0);
        let (tx, mut rx): (Sender<NewMeasurement>, Receiver<NewMeasurement>) =
            tokio::sync::mpsc::channel(100);

        let result = store_measurements(
            State(tx),
            Json(NewMeasurements::Measurement(new_measurement)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);

        assert!(
            rx.recv().await.is_some(),
            "Measurement should be sent to background thread"
        );
    }

    #[sqlx::test]
    async fn should_store_single_measurement_with_ts(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let (tx, mut rx): (Sender<NewMeasurement>, Receiver<NewMeasurement>) =
            tokio::sync::mpsc::channel(100);
        let new_measurement = NewMeasurement::new(Some(chrono::Utc::now()), 1, 1, 1.0);
        let result = store_measurements(
            State(tx),
            Json(NewMeasurements::Measurement(new_measurement)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);

        assert!(
            rx.recv().await.is_some(),
            "Measurement should be sent to background thread"
        );
    }

    #[sqlx::test]
    async fn should_store_multiple_measurements(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurements = vec![
            NewMeasurement::new(None, 1, 1, 1.0),
            NewMeasurement::new(None, 1, 1, 2.0),
        ];

        // Create a channel to send measurements to the background thread
        let (tx, mut rx): (Sender<NewMeasurement>, Receiver<NewMeasurement>) =
            tokio::sync::mpsc::channel(100);

        let result = store_measurements(
            State(tx),
            Json(NewMeasurements::Measurements(new_measurements)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);

        // Check that both measurements were sent to the background thread
        assert!(
            rx.recv().await.is_some(),
            "First measurement should be sent to background thread"
        );
    }

    #[sqlx::test]
    async fn should_store_multiple_measurements_with_and_without_ts(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let (tx, mut rx): (Sender<NewMeasurement>, Receiver<NewMeasurement>) =
            tokio::sync::mpsc::channel(100);
        let new_measurements = vec![
            NewMeasurement::new(None, 1, 1, 1.0),
            NewMeasurement::new(Some(chrono::Utc::now()), 1, 1, 2.0),
        ];
        let result = store_measurements(
            State(tx),
            Json(NewMeasurements::Measurements(new_measurements)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
        // Check that both measurements were sent to the background thread
        assert!(
            rx.recv().await.is_some(),
            "First measurement should be sent to background thread"
        );
        assert!(
            rx.recv().await.is_some(),
            "Second measurement should be sent to background thread"
        );
    }
}
