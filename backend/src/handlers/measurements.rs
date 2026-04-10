use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tracing::instrument;
use utoipa::IntoParams;

use crate::measurements::{Measurement, MeasurementStats, NewMeasurement, NewMeasurements};

use super::error::HandlerError;

type ApplicationState = State<(PgPool, Cache<(i32, i32), Measurement>)>;

#[utoipa::path(
    post,
    path = "api/measurements",
    request_body = NewMeasurements,
    responses(
        (status = 201, description = "Measurement(s) inserted successfully"),
        (status = 500, description = "Internal server error"),
    )
)]
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
            tx.send(new_measurement)
                .await
                .context("Failed to send measurement to background thread")?;
        }
        NewMeasurements::Measurements(new_measurements) => {
            for measurement in new_measurements {
                tx.send(measurement)
                    .await
                    .context("Failed to send measurement to background thread")?;
            }
        }
    };

    let resp = Response::builder()
        .status(201)
        .body("Measurement(s) inserted successfully".into())
        .context("Failed to build response")?;

    Ok(resp)
}

#[utoipa::path(
    get,
    path = "api/measurements/latest",
    responses(
        (status = 200, description = "Latest measurement", body = Measurement),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_latest_measurement(
    State(app_state): ApplicationState,
) -> Result<Json<Measurement>, HandlerError> {
    let (pool, _cache) = app_state;

    let entry = Measurement::read_latest(&pool)
        .await
        .context("Failed to fetch data from database")?;

    Ok(Json(entry))
}

#[utoipa::path(
    get,
    path = "api/measurements/count",
    responses(
        (status = 200, description = "Total count of measurements", body = usize),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_measurements_count(
    State(app_state): ApplicationState,
) -> Result<Json<usize>, HandlerError> {
    let (pool, _cache) = app_state;
    let count = Measurement::read_total_measurements(&pool)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(count as usize))
}

#[utoipa::path(
    get,
    path = "api/measurements",
    responses(
        (status = 200, description = "List of all measurements", body = [Measurement]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_all_measurements(
    State(app_state): ApplicationState,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let entries = Measurement::read_all(&pool)
        .await
        .context("Failed to fetch data from database")?;

    Ok(Json(entries))
}

#[utoipa::path(
    get,
    path = "api/measurements/device/{device_id}",
    params(
        ("device_id" = i32, Path, description = "Device ID")
    ),
    responses(
        (status = 200, description = "List of measurements for device", body = [Measurement]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_measurement_by_device_id(
    State(app_state): ApplicationState,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_by_device_id(device_id, &pool)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(measurements))
}

#[utoipa::path(
    get,
    path = "api/measurements/device/{device_id}/sensor/{sensor_id}/latest",
    params(
        ("device_id" = i32, Path, description = "Device ID"),
        ("sensor_id" = i32, Path, description = "Sensor ID")
    ),
    responses(
        (status = 200, description = "Latest measurement for device and sensor", body = Measurement),
        (status = 500, description = "Internal server error"),
    )
)]
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
            .context("Failed to fetch data from database")?;
    // Insert into cache
    cache
        .insert((device_id, sensor_id), measurement.clone())
        .await;
    Ok(Json(measurement))
}

#[utoipa::path(
    get,
    path = "api/measurements/device/{device_id}/sensor/{sensor_id}",
    params(
        ("device_id" = i32, Path, description = "Device ID"),
        ("sensor_id" = i32, Path, description = "Sensor ID")
    ),
    responses(
        (status = 200, description = "List of measurements for device and sensor", body = [Measurement]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_measurement_by_device_id_and_sensor_id(
    State(app_state): ApplicationState,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(measurements))
}

#[utoipa::path(
    get,
    path = "api/measurements/latest/all",
    responses(
        (status = 200, description = "List of all latest measurements", body = [Measurement]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_all_latest_measurements(
    State(app_state): ApplicationState,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_all_latest_measurements(&pool)
        .await
        .context("Failed to fetch data from database")?;
    // Insert all latest measurements into cache
    Ok(Json(measurements))
}

#[utoipa::path(
    get,
    path = "api/measurements/device/{device_id}/sensor/{sensor_id}/stats",
    params(
        ("device_id" = i32, Path, description = "Device ID"),
        ("sensor_id" = i32, Path, description = "Sensor ID")
    ),
    responses(
        (status = 200, description = "Statistics for measurements by device and sensor", body = MeasurementStats),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_stats_by_device_id_and_sensor_id(
    State(app_state): ApplicationState,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<MeasurementStats>, HandlerError> {
    let (pool, _cache) = app_state;
    let stats = Measurement::read_stats_by_device_id_and_sensor_id(&pool, device_id, sensor_id)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(stats))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct DateRangeParams {
    /// Start of the date range (required), ISO 8601 / RFC 3339 format
    pub start: DateTime<Utc>,
    /// End of the date range (optional, defaults to now), ISO 8601 / RFC 3339 format
    pub end: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    path = "api/measurements/range",
    params(DateRangeParams),
    responses(
        (status = 200, description = "List of measurements within date range", body = [Measurement]),
        (status = 400, description = "Missing or invalid query parameters"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_measurements_by_date_range(
    State(app_state): ApplicationState,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let measurements = Measurement::read_by_date_range(&pool, params.start, params.end)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(measurements))
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

    fn make_app_state(pool: PgPool) -> ApplicationState {
        let cache: Cache<(i32, i32), Measurement> =
            moka::future::Cache::builder().max_capacity(128).build();
        State((pool, cache))
    }

    async fn setup_device_and_sensor(pool: &PgPool) {
        NewDevice::new("test-device".to_string(), "test-location".to_string())
            .insert(pool)
            .await
            .unwrap();
        NewSensor::new("test-sensor".to_string(), "°C".to_string())
            .insert(pool)
            .await
            .unwrap();
    }

    #[sqlx::test]
    async fn fetch_measurements_by_date_range_returns_measurements_in_window(db: PgPool) {
        setup_device_and_sensor(&db).await;

        let now = chrono::Utc::now();
        // Insert a measurement that falls inside the window
        NewMeasurement::new(Some(now), 1, 1, 55.0)
            .insert(&db)
            .await
            .unwrap();

        let state = make_app_state(db);
        let params = Query(DateRangeParams {
            start: now - chrono::Duration::seconds(10),
            end: Some(now + chrono::Duration::seconds(10)),
        });

        let Json(results) = fetch_measurements_by_date_range(state, params)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, 55.0);
    }

    #[sqlx::test]
    async fn fetch_measurements_by_date_range_excludes_measurements_outside_window(db: PgPool) {
        setup_device_and_sensor(&db).await;

        let now = chrono::Utc::now();
        // Outside — 2 hours ago
        NewMeasurement::new(Some(now - chrono::Duration::hours(2)), 1, 1, 99.0)
            .insert(&db)
            .await
            .unwrap();
        // Inside
        NewMeasurement::new(Some(now), 1, 1, 1.0)
            .insert(&db)
            .await
            .unwrap();

        let state = make_app_state(db);
        let params = Query(DateRangeParams {
            start: now - chrono::Duration::minutes(5),
            end: Some(now + chrono::Duration::minutes(5)),
        });

        let Json(results) = fetch_measurements_by_date_range(state, params)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, 1.0);
    }

    #[sqlx::test]
    async fn fetch_measurements_by_date_range_returns_empty_for_future_window(db: PgPool) {
        setup_device_and_sensor(&db).await;

        let now = chrono::Utc::now();
        NewMeasurement::new(Some(now), 1, 1, 3.0)
            .insert(&db)
            .await
            .unwrap();

        let state = make_app_state(db);
        let params = Query(DateRangeParams {
            start: now + chrono::Duration::hours(1),
            end: Some(now + chrono::Duration::hours(2)),
        });

        let Json(results) = fetch_measurements_by_date_range(state, params)
            .await
            .unwrap();

        assert!(results.is_empty());
    }

    #[sqlx::test]
    async fn fetch_measurements_by_date_range_without_end_defaults_to_now(db: PgPool) {
        setup_device_and_sensor(&db).await;

        let now = chrono::Utc::now();
        NewMeasurement::new(Some(now), 1, 1, 8.0)
            .insert(&db)
            .await
            .unwrap();

        let state = make_app_state(db);
        let params = Query(DateRangeParams {
            start: now - chrono::Duration::seconds(10),
            end: None,
        });

        let Json(results) = fetch_measurements_by_date_range(state, params)
            .await
            .unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].value, 8.0);
    }
}
