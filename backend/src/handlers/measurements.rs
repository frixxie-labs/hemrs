use anyhow::Context;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tracing::instrument;
use utoipa::IntoParams;

use crate::devices::DeviceId;
use crate::measurements::{
    Measurement, MeasurementStats, NewMeasurement, NewMeasurements,
    date_range::DateRange,
    latest::{LatestMeasurements, MeasurementKey},
};
use crate::sensors::SensorId;

use super::error::HandlerError;

type ApplicationState = State<(PgPool, LatestMeasurements)>;

#[utoipa::path(
    post,
    path = "api/measurements",
    request_body = NewMeasurements,
    responses(
        (status = 201, description = "Measurement(s) inserted successfully"),
        (status = 503, description = "Measurement ingestion unavailable"),
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
                .map_err(|_| HandlerError::Unavailable("Measurement ingestion unavailable"))?;
        }
        NewMeasurements::Measurements(new_measurements) => {
            for measurement in new_measurements {
                tx.send(measurement)
                    .await
                    .map_err(|_| HandlerError::Unavailable("Measurement ingestion unavailable"))?;
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
        (status = 404, description = "No measurements found"),
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

/// Default number of measurements returned by `GET /api/measurements`.
pub const DEFAULT_MEASUREMENTS_LIMIT: i64 = 1_000;
/// Hard cap on `GET /api/measurements` to keep response memory bounded.
pub const MAX_MEASUREMENTS_LIMIT: i64 = 10_000;

#[derive(Debug, Deserialize, IntoParams)]
pub struct MeasurementsLimitParams {
    /// Number of most recent measurements to return (default 1000, clamped to 1..=10000)
    pub limit: Option<i64>,
}

impl MeasurementsLimitParams {
    fn effective_limit(&self) -> i64 {
        self.limit
            .unwrap_or(DEFAULT_MEASUREMENTS_LIMIT)
            .clamp(1, MAX_MEASUREMENTS_LIMIT)
    }
}

#[utoipa::path(
    get,
    path = "api/measurements",
    params(MeasurementsLimitParams),
    responses(
        (status = 200, description = "Most recent measurements in chronological order, bounded by `limit`", body = [Measurement]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_all_measurements(
    State(app_state): ApplicationState,
    Query(params): Query<MeasurementsLimitParams>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let (pool, _cache) = app_state;
    let entries = Measurement::read_all(&pool, params.effective_limit())
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
    Path(device_id): Path<DeviceId>,
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
        (status = 404, description = "No measurements found for device and sensor"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_latest_measurement_by_device_id_and_sensor_id(
    State(app_state): ApplicationState,
    Path((device_id, sensor_id)): Path<(DeviceId, SensorId)>,
) -> Result<Json<Measurement>, HandlerError> {
    let (pool, cache) = app_state;
    let measurement = cache
        .read(
            &pool,
            MeasurementKey {
                device_id,
                sensor_id,
            },
        )
        .await
        .context("Failed to fetch data from database")?;
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
    Path((device_id, sensor_id)): Path<(DeviceId, SensorId)>,
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
    Path((device_id, sensor_id)): Path<(DeviceId, SensorId)>,
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
    let range = DateRange::new(params.start, params.end.unwrap_or_else(Utc::now))
        .map_err(HandlerError::InvalidInput)?;
    let measurements = Measurement::read_by_date_range(&pool, range)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(measurements))
}

#[cfg(test)]
mod tests {
    use crate::{devices::DeviceId, sensors::SensorId};
    use std::time::Duration;

    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    use super::*;

    #[test]
    fn measurements_limit_defaults_and_clamps() {
        let limit = |limit| MeasurementsLimitParams { limit }.effective_limit();
        assert_eq!(limit(None), DEFAULT_MEASUREMENTS_LIMIT);
        assert_eq!(limit(Some(50)), 50);
        assert_eq!(limit(Some(0)), 1);
        assert_eq!(limit(Some(-5)), 1);
        assert_eq!(limit(Some(i64::MAX)), MAX_MEASUREMENTS_LIMIT);
    }

    type MeasurementParts = (i32, i32, f32, bool);

    fn fixed_timestamp() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_700_000_000, 123_456_789).unwrap()
    }

    fn measurement_from_parts(
        (device, sensor, measurement, with_ts): MeasurementParts,
    ) -> NewMeasurement {
        NewMeasurement {
            timestamp: with_ts.then(fixed_timestamp),
            device: DeviceId(device),
            sensor: SensorId(sensor),
            measurement,
        }
    }

    fn measurements_match(actual: &NewMeasurement, expected: &NewMeasurement) -> bool {
        actual.timestamp == expected.timestamp
            && actual.device == expected.device
            && actual.sensor == expected.sensor
            && actual.measurement.to_bits() == expected.measurement.to_bits()
    }

    #[tokio::test]
    async fn reversed_date_range_is_rejected_before_accessing_database() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgresql://unused:unused@localhost/unused")
            .unwrap();
        pool.close().await;
        let now = Utc::now();
        let result = fetch_measurements_by_date_range(
            make_app_state(pool),
            Query(DateRangeParams {
                start: now,
                end: Some(now - chrono::Duration::seconds(1)),
            }),
        )
        .await;
        assert_eq!(
            result.unwrap_err().into_response().status(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn closed_ingestion_queue_reports_service_unavailable() {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        drop(rx);
        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        let error = store_measurements(State(tx), Json(NewMeasurements::Measurement(measurement)))
            .await
            .unwrap_err();
        assert_eq!(
            error.into_response().status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
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
        let cache = LatestMeasurements::new(128, Duration::from_secs(60));
        State((pool, cache))
    }

    async fn setup_device_and_sensor(pool: &PgPool) {
        NewDevice {
            name: "test-device".to_string(),
            location: "test-location".to_string(),
        }
        .insert(pool)
        .await
        .unwrap();
        NewSensor {
            name: "test-sensor".to_string(),
            unit: "°C".to_string(),
        }
        .insert(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn fetch_measurements_by_date_range_returns_measurements_in_window(db: PgPool) {
        setup_device_and_sensor(&db).await;

        let now = chrono::Utc::now();
        // Insert a measurement that falls inside the window
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 55.0,
        }
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
        NewMeasurement {
            timestamp: Some(now - chrono::Duration::hours(2)),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 99.0,
        }
        .insert(&db)
        .await
        .unwrap();
        // Inside
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        }
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
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 3.0,
        }
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
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 8.0,
        }
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
