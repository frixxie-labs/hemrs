use std::time::Duration;

use anyhow::Result;
use moka::{future::Cache, ops::compute::Op};
use sqlx::PgPool;

use super::Measurement;
use crate::{devices::DeviceId, sensors::SensorId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeasurementKey {
    pub device_id: DeviceId,
    pub sensor_id: SensorId,
}

impl MeasurementKey {
    /// Every device/sensor pair with recorded measurements, as of the last view refresh.
    pub async fn read_all(pool: &PgPool) -> Result<Vec<Self>> {
        let keys = sqlx::query_as!(
            MeasurementKey,
            "SELECT device_id AS \"device_id!: DeviceId\", sensor_id AS \"sensor_id!: SensorId\" FROM device_sensors ORDER BY device_id, sensor_id"
        )
        .fetch_all(pool)
        .await?;
        Ok(keys)
    }
}

#[derive(Debug, Clone)]
pub struct LatestMeasurements {
    cache: Cache<MeasurementKey, Measurement>,
}

impl LatestMeasurements {
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(capacity)
                .time_to_live(ttl)
                .build(),
        }
    }

    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    pub async fn read(&self, pool: &PgPool, key: MeasurementKey) -> Result<Measurement> {
        match self.cache.get(&key).await {
            Some(measurement) => Ok(measurement),
            None => self.refresh(pool, key).await,
        }
    }

    /// Bypass cached reads, but do not let an in-flight query overwrite a newer insert.
    pub async fn refresh(&self, pool: &PgPool, key: MeasurementKey) -> Result<Measurement> {
        let measurement =
            Measurement::read_latest_by_device_id_and_sensor_id(key.device_id, key.sensor_id, pool)
                .await?;
        Ok(self.remember(key, measurement).await)
    }

    /// On a cold cache, check persisted history before accepting a potentially backfilled value.
    pub async fn record(
        &self,
        pool: &PgPool,
        key: MeasurementKey,
        measurement: Measurement,
    ) -> Result<()> {
        self.cache
            .entry(key)
            .and_try_compute_with(|current| async move {
                let operation = match current {
                    Some(entry) if entry.value().timestamp > measurement.timestamp => Op::Nop,
                    Some(_) => Op::Put(measurement),
                    None => Op::Put(
                        Measurement::read_latest_by_device_id_and_sensor_id(
                            key.device_id,
                            key.sensor_id,
                            pool,
                        )
                        .await?,
                    ),
                };
                Ok::<_, anyhow::Error>(operation)
            })
            .await?;
        Ok(())
    }

    async fn remember(&self, key: MeasurementKey, candidate: Measurement) -> Measurement {
        self.cache
            .entry(key)
            .and_compute_with(|current| async move {
                match current {
                    Some(entry) if entry.value().timestamp > candidate.timestamp => Op::Nop,
                    _ => Op::Put(candidate),
                }
            })
            .await
            .into_entry()
            .expect("remember always retains or inserts a measurement")
            .into_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{devices::DeviceId, sensors::SensorId};
    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    #[sqlx::test]
    async fn read_all_keys_lists_each_device_sensor_pair_once(pool: PgPool) {
        for name in ["a", "b"] {
            NewDevice {
                name: name.to_string(),
                location: name.to_string(),
            }
            .insert(&pool)
            .await
            .unwrap();
        }
        NewSensor {
            name: "s".to_string(),
            unit: "C".to_string(),
        }
        .insert(&pool)
        .await
        .unwrap();
        for (device, value) in [(1, 1.0), (1, 2.0), (2, 3.0)] {
            NewMeasurement {
                timestamp: None,
                device: DeviceId(device),
                sensor: SensorId(1),
                measurement: value,
            }
            .insert(&pool)
            .await
            .unwrap();
        }
        crate::devices::Device::refresh_device_sensors_view(&pool)
            .await
            .unwrap();

        let keys = MeasurementKey::read_all(&pool).await.unwrap();

        assert_eq!(
            keys,
            vec![
                MeasurementKey {
                    device_id: DeviceId(1),
                    sensor_id: SensorId(1)
                },
                MeasurementKey {
                    device_id: DeviceId(2),
                    sensor_id: SensorId(1)
                },
            ]
        );
    }

    #[tokio::test]
    async fn concurrent_updates_never_replace_newer_cached_values() {
        let latest = LatestMeasurements::new(8, Duration::from_secs(60));
        let key = MeasurementKey {
            device_id: DeviceId(1),
            sensor_id: SensorId(1),
        };
        let now = chrono::Utc::now();
        let candidates = (0..100).rev().map(|offset| {
            let latest = latest.clone();
            async move {
                latest
                    .remember(
                        key,
                        Measurement {
                            timestamp: now + chrono::Duration::seconds(offset),
                            value: offset as f32,
                            unit: "C".into(),
                            device_name: "device".into(),
                            device_location: "home".into(),
                            sensor_name: "temperature".into(),
                        },
                    )
                    .await;
            }
        });
        futures::future::join_all(candidates).await;
        assert_eq!(latest.cache.get(&key).await.unwrap().value, 99.0);
    }

    #[sqlx::test]
    async fn cached_reads_and_explicit_refresh_have_distinct_policies(pool: PgPool) {
        NewDevice {
            name: "device".into(),
            location: "home".into(),
        }
        .insert(&pool)
        .await
        .unwrap();
        NewSensor {
            name: "temperature".into(),
            unit: "C".into(),
        }
        .insert(&pool)
        .await
        .unwrap();
        let latest = LatestMeasurements::new(8, Duration::from_secs(60));
        let key = MeasurementKey {
            device_id: DeviceId(1),
            sensor_id: SensorId(1),
        };
        let now = chrono::Utc::now();
        for (offset, value) in [(0, 10.0), (1, 20.0)] {
            NewMeasurement {
                timestamp: Some(now + chrono::Duration::seconds(offset)),
                device: DeviceId(1),
                sensor: SensorId(1),
                measurement: value,
            }
            .insert(&pool)
            .await
            .unwrap();
            assert_eq!(latest.read(&pool, key).await.unwrap().value, 10.0);
        }
        assert_eq!(latest.refresh(&pool, key).await.unwrap().value, 20.0);
        pool.close().await;
        assert_eq!(latest.read(&pool, key).await.unwrap().value, 20.0);
        assert!(latest.refresh(&pool, key).await.is_err());
    }
}
