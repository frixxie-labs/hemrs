use metrics::{counter, gauge, histogram};
use moka::future::Cache;
use sqlx::PgPool;
use tokio::sync::{broadcast, mpsc::Receiver};
use tokio::time::Instant;
use tracing::{debug, error, info};

use crate::{
    devices::Device,
    measurements::{Measurement, MeasurementUpdate, NewMeasurement},
    sensors::Sensor,
};

/// Updates metrics in background
pub async fn update_metrics(pool: &PgPool, cache: &Cache<(i32, i32), Measurement>) {
    loop {
        debug!("Running background thread");
        let devices = Device::read(pool).await.unwrap();
        let mut device_sensors: Vec<(Device, Sensor)> = Vec::new();
        for device in devices {
            let sensors = Sensor::read_by_device_id(pool, device.id).await.unwrap();
            for sensor in sensors {
                device_sensors.push((device.clone(), sensor));
            }
        }

        let now = chrono::Utc::now();
        for (device, sensor) in device_sensors {
            //check cache first
            if let Some(measurement) = cache.get(&(device.id, sensor.id)).await {
                if measurement.timestamp >= now - chrono::Duration::seconds(300) {
                    let lables = [
                        ("device_name", measurement.device_name),
                        ("device_location", measurement.device_location),
                        ("sensor_name", measurement.sensor_name),
                        ("unit", measurement.unit),
                    ];
                    gauge!("measurements", &lables).set(measurement.value);
                }
            } else {
                // If not in cache, read from DB
                let measurement =
                    Measurement::read_latest_by_device_id_and_sensor_id(device.id, sensor.id, pool)
                        .await
                        .unwrap();
                if measurement.timestamp >= now - chrono::Duration::seconds(300) {
                    let lables = [
                        ("device_name", measurement.device_name.clone()),
                        ("device_location", measurement.device_location.clone()),
                        ("sensor_name", measurement.sensor_name.clone()),
                        ("unit", measurement.unit.clone()),
                    ];
                    gauge!("measurements", &lables).set(measurement.value);
                    // Store in cache
                    cache
                        .insert((device.id, sensor.id), measurement.clone())
                        .await;
                }
            }
        }
        counter!("hemrs_pg_pool_size").absolute(pool.size() as u64);
        counter!("hemrs_cache_size").absolute(cache.entry_count());
        debug!("Background thread finished");
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

/// Handles inserting new measurements in a background thread
pub async fn handle_insert_measurement_bg_thread(
    mut rx: Receiver<NewMeasurement>,
    pool: PgPool,
    cache: Cache<(i32, i32), Measurement>,
    updates: broadcast::Sender<MeasurementUpdate>,
) {
    while let Some(measurement) = rx.recv().await {
        let queue_size = rx.len();
        info!(
            device_id = measurement.device,
            sensor_id = measurement.sensor,
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
    mut measurement: NewMeasurement,
    pool: &PgPool,
    cache: &Cache<(i32, i32), Measurement>,
) -> anyhow::Result<MeasurementUpdate> {
    debug!(
        device_id = measurement.device,
        sensor_id = measurement.sensor,
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

    let timestamp = measurement.timestamp.unwrap_or_else(chrono::Utc::now);
    measurement.timestamp = Some(timestamp);
    let entry = Measurement {
        value: measurement.measurement,
        timestamp,
        device_name: device.name,
        device_location: device.location,
        sensor_name: sensor.name,
        unit: sensor.unit,
    };

    let measurement_insert_start = Instant::now();
    measurement.insert(pool).await?;
    histogram!("measurement_insert_duration_seconds").record(measurement_insert_start.elapsed());

    let cache_insert_start = Instant::now();
    cache.insert((device.id, sensor.id), entry.clone()).await;
    histogram!("cache_insert_duration_seconds").record(cache_insert_start.elapsed());

    Ok(MeasurementUpdate {
        device_id: device.id,
        sensor_id: sensor.id,
        measurement: entry,
    })
}

pub async fn refresh_views(pool: &PgPool) -> anyhow::Result<()> {
    loop {
        debug!("Refreshing view");
        Device::refresh_device_sensors_view(pool).await?;
        info!("View refreshed successfully");
        tokio::time::sleep(tokio::time::Duration::from_secs(6000)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{devices::NewDevice, sensors::NewSensor};
    use std::time::Duration;

    async fn setup_device_and_sensor(pool: &PgPool) {
        NewDevice::new("stream-device".to_string(), "stream-location".to_string())
            .insert(pool)
            .await
            .unwrap();
        NewSensor::new("stream-sensor".to_string(), "C".to_string())
            .insert(pool)
            .await
            .unwrap();
    }

    #[sqlx::test]
    async fn successful_insert_publishes_measurement_update(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let cache = Cache::builder().max_capacity(8).build();
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
            .send(NewMeasurement::new(Some(timestamp), 1, 1, 23.5))
            .await
            .unwrap();
        drop(measurement_tx);

        let update = tokio::time::timeout(Duration::from_secs(1), update_rx.recv())
            .await
            .unwrap()
            .unwrap();
        worker.await.unwrap();

        assert_eq!(update.device_id, 1);
        assert_eq!(update.sensor_id, 1);
        assert_eq!(update.measurement.timestamp, timestamp);
        assert_eq!(update.measurement.value, 23.5);
        assert_eq!(update.measurement.device_name, "stream-device");
        assert_eq!(update.measurement.sensor_name, "stream-sensor");
        assert_eq!(cache.get(&(1, 1)).await.unwrap().value, 23.5);
        assert_eq!(Measurement::read_all(&pool).await.unwrap().len(), 1);
    }

    #[sqlx::test]
    async fn insert_succeeds_without_stream_subscribers(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let cache = Cache::builder().max_capacity(8).build();
        let (measurement_tx, measurement_rx) = tokio::sync::mpsc::channel(1);
        let (update_tx, _) = broadcast::channel(8);
        let worker = tokio::spawn(handle_insert_measurement_bg_thread(
            measurement_rx,
            pool.clone(),
            cache,
            update_tx,
        ));

        measurement_tx
            .send(NewMeasurement::new(None, 1, 1, 8.0))
            .await
            .unwrap();
        drop(measurement_tx);
        worker.await.unwrap();

        assert_eq!(Measurement::read_all(&pool).await.unwrap().len(), 1);
    }
}
