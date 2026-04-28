use metrics::{counter, gauge, histogram};
use moka::future::Cache;
use sqlx::PgPool;
use tokio::sync::mpsc::Receiver;
use tokio::time::Instant;
use tracing::{debug, error, info};

use crate::{
    devices::Device,
    measurements::{Measurement, NewMeasurement},
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
            Ok(()) => {
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
    cache: &Cache<(i32, i32), Measurement>,
) -> anyhow::Result<()> {
    debug!(
        device_id = measurement.device,
        sensor_id = measurement.sensor,
        "Looking up device and sensor"
    );

    let device_lookup_start = Instant::now();
    let device = Device::read_by_id(pool, measurement.device).await?;
    histogram!("device_lookup_duration_seconds").record(device_lookup_start.elapsed());

    let sensor_lookup_start = Instant::now();
    let sensor = Sensor::read_by_id(pool, measurement.sensor).await?;
    histogram!("sensor_lookup_duration_seconds").record(sensor_lookup_start.elapsed());

    debug!(
        device_name = %device.name,
        sensor_name = %sensor.name,
        "Resolved device and sensor, updating cache"
    );

    let entry = Measurement {
        value: measurement.measurement,
        timestamp: measurement.timestamp.unwrap_or_else(chrono::Utc::now),
        device_name: device.name,
        device_location: device.location,
        sensor_name: sensor.name,
        unit: sensor.unit,
    };

    let cache_insert_start = Instant::now();
    cache.insert((device.id, sensor.id), entry.clone()).await;
    histogram!("cache_insert_duration_seconds").record(cache_insert_start.elapsed());

    let measurement_insert_start = Instant::now();
    measurement.insert(pool).await?;
    histogram!("measurement_insert_duration_seconds").record(measurement_insert_start.elapsed());

    Ok(())
}

pub async fn refresh_views(pool: &PgPool) -> anyhow::Result<()> {
    loop {
        debug!("Refreshing view");
        Device::refresh_device_sensors_view(pool).await?;
        info!("View refreshed successfully");
        tokio::time::sleep(tokio::time::Duration::from_secs(6000)).await;
    }
}
