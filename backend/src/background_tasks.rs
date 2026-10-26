use metrics::{counter, gauge, histogram};
use sqlx::PgPool;
use tokio::sync::{broadcast, mpsc::Receiver};
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use crate::{
    devices::Device,
    measurements::{
        Measurement, MeasurementUpdate, NewMeasurement,
        latest::{LatestMeasurements, MeasurementKey},
    },
    sensors::Sensor,
};

/// Updates metrics in background
pub async fn update_metrics(pool: &PgPool, cache: &LatestMeasurements) {
    loop {
        if let Err(error) = update_metrics_once(pool, cache).await {
            error!(%error, "Metrics update failed; retrying next interval");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

/// Gauges are only published for measurements newer than this, so silent sensors drop out.
const METRICS_FRESHNESS: chrono::Duration = chrono::Duration::seconds(300);

type GaugeLabels = [(&'static str, String); 4];

fn measurement_gauge(
    now: chrono::DateTime<chrono::Utc>,
    measurement: Measurement,
) -> Option<(GaugeLabels, f32)> {
    (now - measurement.timestamp <= METRICS_FRESHNESS).then(|| {
        let labels = [
            ("device_name", measurement.device_name),
            ("device_location", measurement.device_location),
            ("sensor_name", measurement.sensor_name),
            ("unit", measurement.unit),
        ];
        (labels, measurement.value)
    })
}

async fn update_metrics_once(pool: &PgPool, cache: &LatestMeasurements) -> anyhow::Result<()> {
    let now = chrono::Utc::now();
    for key in MeasurementKey::read_all(pool).await? {
        let measurement = cache.read(pool, key).await?;
        if let Some((labels, value)) = measurement_gauge(now, measurement) {
            gauge!("measurements", &labels).set(value);
        }
    }
    counter!("hemrs_pg_pool_size").absolute(pool.size() as u64);
    counter!("hemrs_cache_size").absolute(cache.entry_count());
    Ok(())
}

/// Handles inserting new measurements in a background thread
pub async fn handle_insert_measurement_bg_thread(
    mut rx: Receiver<NewMeasurement>,
    pool: PgPool,
    cache: LatestMeasurements,
    updates: broadcast::Sender<MeasurementUpdate>,
) {
    while let Some(measurement) = rx.recv().await {
        let queue_size = rx.len();
        info!(
            device_id = %measurement.device,
            sensor_id = %measurement.sensor,
            value = measurement.measurement,
            queue_size = queue_size,
            "Received new measurement"
        );

        let start = Instant::now();
        match insert_measurement(measurement, &pool, &cache).await {
            Ok(update) => {
                let _ = updates.send(update);
                let elapsed = start.elapsed();
                histogram!("db_insert_duration_seconds").record(elapsed);
                counter!("new_measurements").increment(1);
                info!(
                    duration_ms = elapsed.as_millis() as u64,
                    "Measurement inserted successfully"
                );
            }
            Err(e) => {
                let elapsed = start.elapsed();
                histogram!("db_insert_duration_seconds").record(elapsed);
                error!(
                    duration_ms = elapsed.as_millis() as u64,
                    error = %e,
                    "Failed to insert measurement"
                );
            }
        }
    }
}

async fn insert_measurement(
    measurement: NewMeasurement,
    pool: &PgPool,
    cache: &LatestMeasurements,
) -> anyhow::Result<MeasurementUpdate> {
    debug!(
        device_id = %measurement.device,
        sensor_id = %measurement.sensor,
        "Looking up device and sensor"
    );

    let lookup_start = Instant::now();
    let (device, sensor) = tokio::join!(
        Device::read_by_id(pool, measurement.device),
        Sensor::read_by_id(pool, measurement.sensor),
    );
    let device = device?;
    let sensor = sensor?;
    histogram!("device_sensor_lookup_duration_seconds").record(lookup_start.elapsed());

    debug!(
        device_name = %device.name,
        sensor_name = %sensor.name,
        "Resolved device and sensor, updating cache"
    );

    let measurement = measurement.resolve_timestamp(chrono::Utc::now());
    let entry = Measurement {
        value: measurement.measurement,
        timestamp: measurement.timestamp,
        device_name: device.name,
        device_location: device.location,
        sensor_name: sensor.name,
        unit: sensor.unit,
    };

    let measurement_insert_start = Instant::now();
    measurement.insert(pool).await?;
    histogram!("measurement_insert_duration_seconds").record(measurement_insert_start.elapsed());

    let cache_insert_start = Instant::now();
    if let Err(error) = cache
        .record(
            pool,
            MeasurementKey {
                device_id: device.id,
                sensor_id: sensor.id,
            },
            entry.clone(),
        )
        .await
    {
        // Persistence succeeded: a cache refresh failure must not suppress its event.
        warn!(%error, device_id = %device.id, sensor_id = %sensor.id, "Measurement persisted but latest cache update failed");
    }
    histogram!("cache_insert_duration_seconds").record(cache_insert_start.elapsed());

    Ok(MeasurementUpdate {
        device_id: device.id,
        sensor_id: sensor.id,
        measurement: entry,
    })
}

pub async fn refresh_views(pool: &PgPool) {
    loop {
        debug!("Refreshing view");
        let delay = match Device::refresh_device_sensors_view(pool).await {
            Ok(()) => {
                info!("View refreshed successfully");
                6000
            }
            Err(error) => {
                error!(%error, "View refresh failed; retrying in 10 seconds");
                10
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{devices::DeviceId, sensors::SensorId};
    use crate::{devices::NewDevice, sensors::NewSensor};
    use std::time::Duration;

    async fn setup_device_and_sensor(pool: &PgPool) {
        NewDevice {
            name: "stream-device".to_string(),
            location: "stream-location".to_string(),
        }
        .insert(pool)
        .await
        .unwrap();
        NewSensor {
            name: "stream-sensor".to_string(),
            unit: "C".to_string(),
        }
        .insert(pool)
        .await
        .unwrap();
    }

    fn measurement_at(timestamp: chrono::DateTime<chrono::Utc>) -> Measurement {
        Measurement {
            timestamp,
            value: 1.0,
            unit: "C".to_string(),
            device_name: "device".to_string(),
            device_location: "location".to_string(),
            sensor_name: "sensor".to_string(),
        }
    }

    #[quickcheck_macros::quickcheck]
    fn gauge_is_emitted_only_for_measurements_within_freshness_window(age_seconds: i32) -> bool {
        let now = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let age = chrono::Duration::seconds(i64::from(age_seconds));
        let gauge = measurement_gauge(now, measurement_at(now - age));
        gauge.is_some() == (age <= METRICS_FRESHNESS)
    }

    #[test]
    fn gauge_labels_describe_the_measurement_source() {
        let now = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let (labels, value) = measurement_gauge(now, measurement_at(now)).unwrap();
        assert_eq!(value, 1.0);
        assert_eq!(
            labels,
            [
                ("device_name", "device".to_string()),
                ("device_location", "location".to_string()),
                ("sensor_name", "sensor".to_string()),
                ("unit", "C".to_string()),
            ]
        );
    }

    #[sqlx::test]
    async fn successful_insert_publishes_measurement_update(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let cache = LatestMeasurements::new(8, Duration::from_secs(60));
        let (measurement_tx, measurement_rx) = tokio::sync::mpsc::channel(1);
        let (update_tx, mut update_rx) = broadcast::channel(8);
        let worker = tokio::spawn(handle_insert_measurement_bg_thread(
            measurement_rx,
            pool.clone(),
            cache.clone(),
            update_tx,
        ));
        let timestamp = chrono::Utc::now();

        measurement_tx
            .send(NewMeasurement {
                timestamp: Some(timestamp),
                device: DeviceId(1),
                sensor: SensorId(1),
                measurement: 23.5,
            })
            .await
            .unwrap();
        drop(measurement_tx);

        let update = tokio::time::timeout(Duration::from_secs(1), update_rx.recv())
            .await
            .unwrap()
            .unwrap();
        worker.await.unwrap();

        assert_eq!(update.device_id, DeviceId(1));
        assert_eq!(update.sensor_id, SensorId(1));
        assert_eq!(update.measurement.timestamp, timestamp);
        assert_eq!(update.measurement.value, 23.5);
        assert_eq!(update.measurement.device_name, "stream-device");
        assert_eq!(update.measurement.sensor_name, "stream-sensor");
        assert_eq!(
            cache
                .read(
                    &pool,
                    MeasurementKey {
                        device_id: DeviceId(1),
                        sensor_id: SensorId(1)
                    }
                )
                .await
                .unwrap()
                .value,
            23.5
        );
        assert_eq!(Measurement::read_all(&pool, 100).await.unwrap().len(), 1);
    }

    #[sqlx::test]
    async fn backfilled_insert_does_not_replace_latest_measurement(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let cache = LatestMeasurements::new(8, Duration::from_secs(60));
        let now = chrono::Utc::now();
        for (timestamp, value) in [(now, 23.5), (now - chrono::Duration::hours(1), 8.0)] {
            insert_measurement(
                NewMeasurement {
                    timestamp: Some(timestamp),
                    device: DeviceId(1),
                    sensor: SensorId(1),
                    measurement: value,
                },
                &pool,
                &cache,
            )
            .await
            .unwrap();
        }
        assert_eq!(
            cache
                .read(
                    &pool,
                    MeasurementKey {
                        device_id: DeviceId(1),
                        sensor_id: SensorId(1)
                    }
                )
                .await
                .unwrap()
                .value,
            23.5
        );
        assert_eq!(Measurement::read_all(&pool, 100).await.unwrap().len(), 2);
    }

    #[sqlx::test]
    async fn backfilled_insert_into_cold_cache_loads_latest_from_history(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let now = chrono::Utc::now();
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 23.5,
        }
        .insert(&pool)
        .await
        .unwrap();
        let cache = LatestMeasurements::new(8, Duration::from_secs(60));
        insert_measurement(
            NewMeasurement {
                timestamp: Some(now - chrono::Duration::hours(1)),
                device: DeviceId(1),
                sensor: SensorId(1),
                measurement: 8.0,
            },
            &pool,
            &cache,
        )
        .await
        .unwrap();
        assert_eq!(
            cache
                .read(
                    &pool,
                    MeasurementKey {
                        device_id: DeviceId(1),
                        sensor_id: SensorId(1)
                    }
                )
                .await
                .unwrap()
                .value,
            23.5
        );
    }

    #[tokio::test]
    async fn maintenance_tasks_remain_running_after_database_errors() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgresql://unused:unused@localhost/unused")
            .unwrap();
        pool.close().await;
        let cache = LatestMeasurements::new(8, Duration::from_secs(60));
        assert!(update_metrics_once(&pool, &cache).await.is_err());
        let metrics_pool = pool.clone();
        let mut metrics_task =
            tokio::spawn(async move { update_metrics(&metrics_pool, &cache).await });
        let mut refresh_task = tokio::spawn(async move { refresh_views(&pool).await });
        assert!(
            tokio::time::timeout(Duration::from_millis(20), &mut metrics_task)
                .await
                .is_err()
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(20), &mut refresh_task)
                .await
                .is_err()
        );
        metrics_task.abort();
        refresh_task.abort();
        assert!(metrics_task.await.unwrap_err().is_cancelled());
        assert!(refresh_task.await.unwrap_err().is_cancelled());
    }

    #[sqlx::test]
    async fn insert_succeeds_without_stream_subscribers(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let cache = LatestMeasurements::new(8, Duration::from_secs(60));
        let (measurement_tx, measurement_rx) = tokio::sync::mpsc::channel(1);
        let (update_tx, _) = broadcast::channel(8);
        let worker = tokio::spawn(handle_insert_measurement_bg_thread(
            measurement_rx,
            pool.clone(),
            cache,
            update_tx,
        ));

        measurement_tx
            .send(NewMeasurement {
                timestamp: None,
                device: DeviceId(1),
                sensor: SensorId(1),
                measurement: 8.0,
            })
            .await
            .unwrap();
        drop(measurement_tx);
        worker.await.unwrap();

        assert_eq!(Measurement::read_all(&pool, 100).await.unwrap().len(), 1);
    }
}
