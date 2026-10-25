use std::time::Duration;

use anyhow::Result;
use moka::{future::Cache, ops::compute::Op};
use sqlx::PgPool;

use super::Measurement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeasurementKey {
    pub device_id: i32,
    pub sensor_id: i32,
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
    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    #[tokio::test]
    async fn concurrent_updates_never_replace_newer_cached_values() {
        let latest = LatestMeasurements::new(8, Duration::from_secs(60));
        let key = MeasurementKey {
            device_id: 1,
            sensor_id: 1,
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
            device_id: 1,
            sensor_id: 1,
        };
        let now = chrono::Utc::now();
        for (offset, value) in [(0, 10.0), (1, 20.0)] {
            NewMeasurement {
                timestamp: Some(now + chrono::Duration::seconds(offset)),
                device: 1,
                sensor: 1,
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
