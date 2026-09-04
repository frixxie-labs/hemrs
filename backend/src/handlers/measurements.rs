use anyhow::Context;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderValue, header},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
};
use chrono::{DateTime, Utc};
use futures::{Stream, stream};
use moka::future::Cache;
use serde::Deserialize;
use sqlx::PgPool;
use std::{convert::Infallible, future::Future, time::Duration};
use tokio::{
    sync::{broadcast, mpsc::Sender},
    time::{MissedTickBehavior, interval},
};
use tracing::{instrument, warn};
use utoipa::IntoParams;

use crate::measurements::{
    Measurement, MeasurementStats, MeasurementUpdate, NewMeasurement, NewMeasurements,
};

use super::error::HandlerError;

type ApplicationState = State<(PgPool, Cache<(i32, i32), Measurement>)>;

const MEASUREMENT_STREAM_INTERVAL: Duration = Duration::from_secs(15);

#[utoipa::path(
    get,
    path = "api/devices/{device_id}/sensors/{sensor_id}/measurements/stream",
    params(
        ("device_id" = i32, Path, description = "Device ID"),
        ("sensor_id" = i32, Path, description = "Sensor ID")
    ),
    responses(
        (status = 200, description = "Live measurement event stream", body = MeasurementUpdate, content_type = "text/event-stream"),
    )
)]
#[instrument(skip(pool, cache, updates))]
pub async fn stream_measurements(
    State((pool, cache, updates)): State<(
        PgPool,
        Cache<(i32, i32), Measurement>,
        broadcast::Sender<MeasurementUpdate>,
    )>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Response {
    stream_measurements_with_interval(
        pool,
        cache,
        updates,
        device_id,
        sensor_id,
        MEASUREMENT_STREAM_INTERVAL,
    )
    .await
}

async fn stream_measurements_with_interval(
    pool: PgPool,
    cache: Cache<(i32, i32), Measurement>,
    updates: broadcast::Sender<MeasurementUpdate>,
    device_id: i32,
    sensor_id: i32,
    repeat_interval: Duration,
) -> Response {
    let receiver = updates.subscribe();
    let measurement = match cache.get(&(device_id, sensor_id)).await {
        Some(measurement) => Some(measurement),
        None => {
            match Measurement::read_latest_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
                .await
            {
                Ok(measurement) => {
                    cache
                        .insert((device_id, sensor_id), measurement.clone())
                        .await;
                    Some(measurement)
                }
                Err(error) => {
                    warn!(%error, device_id, sensor_id, "Failed to load latest measurement for stream");
                    None
                }
            }
        }
    };
    let latest = measurement.map(|measurement| MeasurementUpdate {
        device_id,
        sensor_id,
        measurement,
    });
    let refresh_latest = move || {
        let pool = pool.clone();
        let cache = cache.clone();
        async move {
            match Measurement::read_latest_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
                .await
            {
                Ok(measurement) => {
                    cache
                        .insert((device_id, sensor_id), measurement.clone())
                        .await;
                    Some(MeasurementUpdate {
                        device_id,
                        sensor_id,
                        measurement,
                    })
                }
                Err(error) => {
                    warn!(%error, device_id, sensor_id, "Failed to refresh latest measurement for stream");
                    None
                }
            }
        }
    };
    measurement_stream_response(
        receiver,
        device_id,
        sensor_id,
        latest,
        repeat_interval,
        refresh_latest,
    )
}

fn measurement_stream_response<Refresh, RefreshFuture>(
    receiver: broadcast::Receiver<MeasurementUpdate>,
    device_id: i32,
    sensor_id: i32,
    latest: Option<MeasurementUpdate>,
    repeat_interval: Duration,
    refresh_latest: Refresh,
) -> Response
where
    Refresh: FnMut() -> RefreshFuture + Send + 'static,
    RefreshFuture: Future<Output = Option<MeasurementUpdate>> + Send + 'static,
{
    let stream = measurement_stream(
        receiver,
        device_id,
        sensor_id,
        latest,
        repeat_interval,
        refresh_latest,
    );
    let mut response = Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(MEASUREMENT_STREAM_INTERVAL)
                .text("keep-alive"),
        )
        .into_response();
    response.headers_mut().insert(
        header::HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    response
}

fn measurement_stream<Refresh, RefreshFuture>(
    receiver: broadcast::Receiver<MeasurementUpdate>,
    device_id: i32,
    sensor_id: i32,
    latest: Option<MeasurementUpdate>,
    repeat_interval: Duration,
    refresh_latest: Refresh,
) -> impl Stream<Item = Result<Event, Infallible>>
where
    Refresh: FnMut() -> RefreshFuture,
    RefreshFuture: Future<Output = Option<MeasurementUpdate>>,
{
    let mut interval = interval(repeat_interval);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    stream::unfold(
        (receiver, latest, interval, refresh_latest),
        move |state| async move {
            let (mut receiver, mut latest, mut interval, mut refresh_latest) = state;
            loop {
                tokio::select! {
                    result = receiver.recv() => match result {
                        Ok(update) if update.device_id == device_id && update.sensor_id == sensor_id => {
                            latest = Some(update.clone());
                            match Event::default().event("measurement").json_data(update) {
                                Ok(event) => return Some((Ok(event), (receiver, latest, interval, refresh_latest))),
                                Err(error) => warn!(%error, "Failed to serialize measurement update"),
                            }
                        }
                        Ok(_) => {}
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!(skipped, device_id, sensor_id, "Measurement stream lagged");
                        }
                        Err(broadcast::error::RecvError::Closed) => return None,
                    },
                    _ = interval.tick() => {
                        if let Some(update) = refresh_latest().await {
                            latest = Some(update);
                        }
                        if let Some(update) = latest.clone() {
                            match Event::default().event("measurement").json_data(update) {
                                Ok(event) => return Some((Ok(event), (receiver, latest, interval, refresh_latest))),
                                Err(error) => warn!(%error, "Failed to serialize latest measurement"),
                            }
                        }
                    }
                }
            }
        },
    )
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
    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};
    use futures::StreamExt;

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

    fn measurement_update(device_id: i32, sensor_id: i32, value: f32) -> MeasurementUpdate {
        MeasurementUpdate {
            device_id,
            sensor_id,
            measurement: Measurement {
                timestamp: fixed_timestamp(),
                value,
                unit: "C".to_string(),
                device_name: "device".to_string(),
                device_location: "location".to_string(),
                sensor_name: "sensor".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn stream_measurements_emits_matching_updates() {
        let (updates, _) = broadcast::channel(8);
        let response = measurement_stream_response(
            updates.subscribe(),
            1,
            2,
            None,
            MEASUREMENT_STREAM_INTERVAL,
            || std::future::ready(None),
        );

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/event-stream"
        );
        assert_eq!(response.headers().get("x-accel-buffering").unwrap(), "no");

        let mut body = response.into_body().into_data_stream();
        updates.send(measurement_update(1, 2, 12.5)).unwrap();
        let chunk = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let event = String::from_utf8(chunk.to_vec()).unwrap();

        assert!(event.starts_with("event: measurement\n"));
        assert!(event.contains("\"device_id\":1"));
        assert!(event.contains("\"sensor_id\":2"));
        assert!(event.contains("\"value\":12.5"));
    }

    #[tokio::test]
    async fn stream_measurements_ignores_updates_for_other_pairs() {
        let (updates, _) = broadcast::channel(8);
        let response = measurement_stream_response(
            updates.subscribe(),
            1,
            2,
            None,
            MEASUREMENT_STREAM_INTERVAL,
            || std::future::ready(None),
        );
        let mut body = response.into_body().into_data_stream();

        updates.send(measurement_update(9, 9, 1.0)).unwrap();
        updates.send(measurement_update(1, 2, 2.0)).unwrap();

        let chunk = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let event = String::from_utf8(chunk.to_vec()).unwrap();
        assert!(event.contains("\"value\":2.0"));
        assert!(!event.contains("\"value\":1.0"));
    }

    #[tokio::test]
    async fn stream_measurements_resumes_after_lag() {
        let (updates, _) = broadcast::channel(1);
        let response = measurement_stream_response(
            updates.subscribe(),
            1,
            2,
            None,
            MEASUREMENT_STREAM_INTERVAL,
            || std::future::ready(None),
        );
        let mut body = response.into_body().into_data_stream();

        updates.send(measurement_update(1, 2, 1.0)).unwrap();
        updates.send(measurement_update(1, 2, 2.0)).unwrap();

        let chunk = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let event = String::from_utf8(chunk.to_vec()).unwrap();
        assert!(event.contains("\"value\":2.0"));
    }

    #[tokio::test]
    async fn stream_measurements_periodically_repeats_latest_update() {
        let (updates, _) = broadcast::channel(8);
        let response = measurement_stream_response(
            updates.subscribe(),
            1,
            2,
            Some(measurement_update(1, 2, 12.5)),
            Duration::from_millis(10),
            || std::future::ready(None),
        );
        let mut body = response.into_body().into_data_stream();

        let first = body.next().await.unwrap().unwrap();
        assert!(
            String::from_utf8(first.to_vec())
                .unwrap()
                .contains("\"value\":12.5")
        );

        updates.send(measurement_update(1, 2, 13.5)).unwrap();
        let live = body.next().await.unwrap().unwrap();
        assert!(
            String::from_utf8(live.to_vec())
                .unwrap()
                .contains("\"value\":13.5")
        );

        let repeated = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            String::from_utf8(repeated.to_vec())
                .unwrap()
                .contains("\"value\":13.5")
        );
    }

    #[sqlx::test]
    async fn stream_measurements_refreshes_latest_from_database(db: PgPool) {
        setup_device_and_sensor(&db).await;
        let timestamp = fixed_timestamp();
        NewMeasurement::new(Some(timestamp), 1, 1, 12.5)
            .insert(&db)
            .await
            .unwrap();
        let cache = Cache::builder().max_capacity(8).build();
        let (updates, _) = broadcast::channel(8);
        let response = stream_measurements_with_interval(
            db.clone(),
            cache.clone(),
            updates.clone(),
            1,
            1,
            Duration::from_millis(10),
        )
        .await;
        let mut body = response.into_body().into_data_stream();

        let initial = body.next().await.unwrap().unwrap();
        assert!(
            String::from_utf8(initial.to_vec())
                .unwrap()
                .contains("\"value\":12.5")
        );

        NewMeasurement::new(Some(timestamp + chrono::Duration::seconds(1)), 1, 1, 13.5)
            .insert(&db)
            .await
            .unwrap();

        let refreshed = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            String::from_utf8(refreshed.to_vec())
                .unwrap()
                .contains("\"value\":13.5")
        );
        assert_eq!(cache.get(&(1, 1)).await.unwrap().value, 13.5);
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
