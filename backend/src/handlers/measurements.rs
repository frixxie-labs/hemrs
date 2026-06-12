use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use metrics::gauge;
use moka::future::Cache;
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tracing::instrument;
use utoipa::IntoParams;

use crate::measurements::{Measurement, MeasurementStats, NewMeasurement, NewMeasurements};

use super::error::HandlerError;

type ApplicationState = State<(PgPool, Cache<(i32, i32), Measurement>)>;

static MAX_OBSERVED_MEASUREMENT_QUEUE_LEN: AtomicUsize = AtomicUsize::new(0);

fn record_measurement_queue_len(tx: &Sender<NewMeasurement>) {
    let queue_len = tx.max_capacity().saturating_sub(tx.capacity());
    let previous_max = MAX_OBSERVED_MEASUREMENT_QUEUE_LEN.fetch_max(queue_len, Ordering::Relaxed);
    let max_queue_len = previous_max.max(queue_len);

    gauge!("measurement_insert_queue_max_len").set(max_queue_len as f64);
}

async fn enqueue_measurement(
    tx: &Sender<NewMeasurement>,
    measurement: NewMeasurement,
) -> anyhow::Result<()> {
    tx.send(measurement)
        .await
        .context("Failed to send measurement to background thread")?;
    record_measurement_queue_len(tx);
    Ok(())
}

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
            enqueue_measurement(&tx, new_measurement).await?;
        }
        NewMeasurements::Measurements(new_measurements) => {
            for measurement in new_measurements {
                enqueue_measurement(&tx, measurement).await?;
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
    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    use super::*;

    type MeasurementParts = (i32, i32, f32, bool);

    fn fixed_timestamp() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_700_000_000, 123_456_789).unwrap()
    }

    fn measurement_from_parts(
        (device, sensor, measurement, with_ts): MeasurementParts,
    ) -> NewMeasurement {
        NewMeasurement::new(with_ts.then(fixed_timestamp), device, sensor, measurement)
    }

    fn measurements_match(actual: &NewMeasurement, expected: &NewMeasurement) -> bool {
        actual.timestamp == expected.timestamp
            && actual.device == expected.device
            && actual.sensor == expected.sensor
            && actual.measurement.to_bits() == expected.measurement.to_bits()
    }

    fn store_measurements_queues_expected(
        payload: NewMeasurements,
        expected: Vec<NewMeasurement>,
    ) -> bool {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let (tx, mut rx): (Sender<NewMeasurement>, _) =
                tokio::sync::mpsc::channel(expected.len().max(1));

            let Ok(response) = store_measurements(State(tx), Json(payload)).await else {
                return false;
            };

            if response.status() != axum::http::StatusCode::CREATED {
                return false;
            }

            for expected_measurement in &expected {
                let Some(actual) = rx.recv().await else {
                    return false;
                };
                if !measurements_match(&actual, expected_measurement) {
                    return false;
                }
            }

            rx.recv().await.is_none()
        })
    }

    #[quickcheck_macros::quickcheck]
    fn store_single_measurement_queues_exact_measurement(parts: MeasurementParts) -> bool {
        let measurement = measurement_from_parts(parts);
        store_measurements_queues_expected(
            NewMeasurements::Measurement(measurement.clone()),
            vec![measurement],
        )
    }

    #[quickcheck_macros::quickcheck]
    fn store_measurement_batch_queues_all_measurements_in_order(
        items: Vec<MeasurementParts>,
    ) -> bool {
        let measurements = items
            .into_iter()
            .map(measurement_from_parts)
            .collect::<Vec<_>>();
        store_measurements_queues_expected(
            NewMeasurements::Measurements(measurements.clone()),
            measurements,
        )
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
