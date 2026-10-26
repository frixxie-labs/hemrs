use axum::{
    extract::{Path, State},
    http::{HeaderValue, header},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
};
use futures::{Stream, stream};
use sqlx::PgPool;
use std::{convert::Infallible, future::Future, time::Duration};
use tokio::{
    sync::broadcast,
    time::{MissedTickBehavior, interval},
};
use tracing::{instrument, warn};

use crate::devices::DeviceId;
use crate::measurements::{
    MeasurementUpdate,
    latest::{LatestMeasurements, MeasurementKey},
};
use crate::sensors::SensorId;

type MeasurementStreamState = State<(
    PgPool,
    LatestMeasurements,
    broadcast::Sender<MeasurementUpdate>,
)>;

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
    State((pool, cache, updates)): MeasurementStreamState,
    Path((device_id, sensor_id)): Path<(DeviceId, SensorId)>,
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
    cache: LatestMeasurements,
    updates: broadcast::Sender<MeasurementUpdate>,
    device_id: DeviceId,
    sensor_id: SensorId,
    repeat_interval: Duration,
) -> Response {
    let receiver = updates.subscribe();
    let key = MeasurementKey {
        device_id,
        sensor_id,
    };
    let measurement = match cache.read(&pool, key).await {
        Ok(measurement) => Some(measurement),
        Err(error) => {
            warn!(%error, %device_id, %sensor_id, "Failed to load latest measurement for stream");
            None
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
            match cache.refresh(&pool, key).await {
                Ok(measurement) => Some(MeasurementUpdate {
                    device_id,
                    sensor_id,
                    measurement,
                }),
                Err(error) => {
                    warn!(%error, %device_id, %sensor_id, "Failed to refresh latest measurement for stream");
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
    device_id: DeviceId,
    sensor_id: SensorId,
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
    device_id: DeviceId,
    sensor_id: SensorId,
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
                            latest = Some(newest_update(latest, update.clone()));
                            match Event::default().event("measurement").json_data(update) {
                                Ok(event) => return Some((Ok(event), (receiver, latest, interval, refresh_latest))),
                                Err(error) => warn!(%error, "Failed to serialize measurement update"),
                            }
                        }
                        Ok(_) => {}
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!(skipped, %device_id, %sensor_id, "Measurement stream lagged");
                        }
                        Err(broadcast::error::RecvError::Closed) => return None,
                    },
                    _ = interval.tick() => {
                        if let Some(update) = refresh_latest().await {
                            latest = Some(newest_update(latest, update));
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

fn newest_update(
    current: Option<MeasurementUpdate>,
    candidate: MeasurementUpdate,
) -> MeasurementUpdate {
    match current {
        Some(current) if current.measurement.timestamp > candidate.measurement.timestamp => current,
        _ => candidate,
    }
}

#[cfg(test)]
mod tests {
    use crate::{devices::DeviceId, sensors::SensorId};
    use chrono::{DateTime, Utc};
    use futures::StreamExt;

    use crate::{
        devices::NewDevice,
        measurements::{Measurement, NewMeasurement},
        sensors::NewSensor,
    };

    use super::*;

    fn fixed_timestamp() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_700_000_000, 123_456_789).unwrap()
    }

    fn measurement_update(device_id: i32, sensor_id: i32, value: f32) -> MeasurementUpdate {
        MeasurementUpdate {
            device_id: DeviceId(device_id),
            sensor_id: SensorId(sensor_id),
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
            DeviceId(1),
            SensorId(2),
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
            DeviceId(1),
            SensorId(2),
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
            DeviceId(1),
            SensorId(2),
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
    async fn stream_does_not_repeat_backfill_as_latest() {
        let (updates, _) = broadcast::channel(8);
        let response = measurement_stream_response(
            updates.subscribe(),
            DeviceId(1),
            SensorId(2),
            Some(measurement_update(1, 2, 20.0)),
            Duration::from_millis(10),
            || std::future::ready(None),
        );
        let mut body = response.into_body().into_data_stream();
        body.next().await.unwrap().unwrap();
        let mut historical = measurement_update(1, 2, 10.0);
        historical.measurement.timestamp -= chrono::Duration::hours(1);
        updates.send(historical).unwrap();
        // Backfilled inserts are still broadcast, but must not become the periodic snapshot.
        body.next().await.unwrap().unwrap();
        let repeated = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            String::from_utf8(repeated.to_vec())
                .unwrap()
                .contains("\"value\":20.0")
        );
    }

    #[tokio::test]
    async fn stream_measurements_periodically_repeats_latest_update() {
        let (updates, _) = broadcast::channel(8);
        let response = measurement_stream_response(
            updates.subscribe(),
            DeviceId(1),
            SensorId(2),
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
        NewMeasurement {
            timestamp: Some(timestamp),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 12.5,
        }
        .insert(&db)
        .await
        .unwrap();
        let cache = LatestMeasurements::new(8, Duration::from_secs(60));
        let (updates, _) = broadcast::channel(8);
        let response = stream_measurements_with_interval(
            db.clone(),
            cache.clone(),
            updates.clone(),
            DeviceId(1),
            SensorId(1),
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

        NewMeasurement {
            timestamp: Some(timestamp + chrono::Duration::seconds(1)),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 13.5,
        }
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
        assert_eq!(
            cache
                .read(
                    &db,
                    MeasurementKey {
                        device_id: DeviceId(1),
                        sensor_id: SensorId(1)
                    }
                )
                .await
                .unwrap()
                .value,
            13.5
        );
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
}
