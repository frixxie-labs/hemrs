use metrics::{counter, gauge};
use moka::future::Cache;
use sqlx::PgPool;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info, warn};

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
                    if measurement.timestamp < now - chrono::Duration::seconds(300) {
                        continue;
                    }
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
        info!("Received new measurement: {:?}", measurement);
        info!("Current queue size: {}", rx.len());
        if let Err(e) = insert_measurement(measurement, &pool, &cache).await {
            warn!("Failed to insert measurement: {}", e);
        } else {
            counter!("new_measurements").increment(1);
        }
    }
}

async fn insert_measurement(
    measurement: NewMeasurement,
    pool: &PgPool,
    cache: &Cache<(i32, i32), Measurement>,
) -> anyhow::Result<()> {
    let device = Device::read_by_id(pool, measurement.device).await?;

    let sensor = Sensor::read_by_id(pool, measurement.sensor).await?;
    let entry = Measurement {
        value: measurement.measurement,
        timestamp: measurement.timestamp.unwrap_or_else(chrono::Utc::now),
        device_name: device.name,
        device_location: device.location,
        sensor_name: sensor.name,
        unit: sensor.unit,
    };
    cache.insert((device.id, sensor.id), entry.clone()).await;
    measurement.insert(pool).await?;
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
