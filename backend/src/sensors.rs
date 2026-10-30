use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt;
use tracing::info;
use utoipa::ToSchema;

use crate::devices::{Device, DeviceId};

/// Identifies a sensor; serialized and stored as a plain integer.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    sqlx::Type,
    ToSchema,
)]
#[serde(transparent)]
#[sqlx(transparent)]
#[schema(value_type = i32)]
pub struct SensorId(pub i32);

impl fmt::Display for SensorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewSensor {
    pub name: String,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Sensor {
    pub id: SensorId,
    pub name: String,
    pub unit: String,
}

impl Sensor {
    pub async fn read(pool: &PgPool) -> Result<Vec<Sensor>> {
        let sensors = sqlx::query_as!(
            Sensor,
            "SELECT id AS \"id: SensorId\", name, unit FROM sensors"
        )
        .fetch_all(pool)
        .await?;
        Ok(sensors)
    }

    pub async fn read_by_id(pool: &PgPool, sensor_id: SensorId) -> Result<Sensor> {
        let sensors = sqlx::query_as!(
            Sensor,
            "SELECT id AS \"id: SensorId\", name, unit FROM sensors WHERE id = $1",
            sensor_id.0
        )
        .fetch_one(pool)
        .await?;
        Ok(sensors)
    }

    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        info!(sensor_id = %self.id, sensor_name = %self.name, "Deleting sensor");
        sqlx::query!("DELETE FROM sensors WHERE id = $1", self.id.0)
            .execute(pool)
            .await?;
        info!(sensor_id = %self.id, "Sensor deleted");
        Ok(())
    }

    pub async fn update(self, pool: &PgPool) -> anyhow::Result<()> {
        info!(sensor_id = %self.id, name = %self.name, unit = %self.unit, "Updating sensor");
        sqlx::query!(
            "UPDATE sensors SET name = $1,unit = $2 WHERE id = $3",
            self.name,
            self.unit,
            self.id.0
        )
        .execute(pool)
        .await?;
        Device::refresh_device_sensors_view(pool).await?;
        info!(sensor_id = %self.id, "Sensor updated");
        Ok(())
    }

    pub async fn read_by_device_id(pool: &PgPool, device_id: DeviceId) -> Result<Vec<Sensor>> {
        let sensors = sqlx::query_as!(
            Sensor,
            "SELECT ds.sensor_id as \"id!: SensorId\", ds.name as \"name!\", ds.unit as \"unit!\" from device_sensors ds WHERE ds.device_id = $1 order by ds.sensor_id",
            device_id.0
        )
        .fetch_all(pool)
        .await?;
        Ok(sensors)
    }
}

impl NewSensor {
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        info!(name = %self.name, unit = %self.unit, "Inserting new sensor");
        sqlx::query!(
            "INSERT INTO sensors (name, unit) VALUES ($1, $2)",
            self.name,
            self.unit
        )
        .execute(pool)
        .await?;
        Device::refresh_device_sensors_view(pool).await?;
        info!(name = %self.name, "Sensor inserted");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{devices::DeviceId, sensors::SensorId};
    #[quickcheck_macros::quickcheck]
    fn sensor_id_serializes_as_plain_integer(raw: i32) -> bool {
        let json = serde_json::to_string(&super::SensorId(raw)).unwrap();
        json == raw.to_string()
            && serde_json::from_str::<super::SensorId>(&json).unwrap() == super::SensorId(raw)
    }

    use sqlx::PgPool;

    use crate::{
        devices::{Device, NewDevice},
        measurements::NewMeasurement,
        sensors::{NewSensor, Sensor},
    };

    // ── sqlx integration tests ────────────────────────────────────────────────

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let sensor = NewSensor {
            name: "test".to_string(),
            unit: "test".to_string(),
        };
        sensor.insert(&pool).await.unwrap();
        let sensors = Sensor::read(&pool).await.unwrap();

        assert!(!sensors.is_empty());
        assert_eq!(sensors.last().unwrap().name, "test");
        assert_eq!(sensors.last().unwrap().unit, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let sensor = NewSensor {
            name: "test".to_string(),
            unit: "test".to_string(),
        };
        sensor.clone().insert(&pool).await.unwrap();
        let sensors = Sensor::read(&pool).await.unwrap();
        let sensor = sensors.last().unwrap().clone().delete(&pool).await;
        assert!(sensor.is_ok());
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let sensor = NewSensor {
            name: "test".to_string(),
            unit: "test".to_string(),
        };
        sensor.clone().insert(&pool).await.unwrap();
        let sensors = Sensor::read(&pool).await.unwrap();
        let sensor = sensors.last().unwrap().clone();
        let sensor = Sensor {
            id: sensor.id,
            name: "test2".to_string(),
            unit: "test2".to_string(),
        };
        sensor.clone().update(&pool).await.unwrap();

        let sensors = Sensor::read(&pool).await.unwrap();
        assert_eq!(sensors.last().unwrap().name, "test2");
        assert_eq!(sensors.last().unwrap().unit, "test2");
    }

    #[sqlx::test]
    async fn read_by_device_id(pool: PgPool) {
        let device = NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
            coordinates: None,
        };
        device.clone().insert(&pool).await.unwrap();
        let sensor = NewSensor {
            name: "test".to_string(),
            unit: "test".to_string(),
        };
        sensor.clone().insert(&pool).await.unwrap();
        let sensor = NewSensor {
            name: "test2".to_string(),
            unit: "test".to_string(),
        };
        sensor.clone().insert(&pool).await.unwrap();

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();
        let measurement2 = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(2),
            measurement: 1.0,
        };
        measurement2.insert(&pool).await.unwrap();
        let measurement3 = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(2),
            measurement: 1.0,
        };
        measurement3.insert(&pool).await.unwrap();

        Device::refresh_device_sensors_view(&pool).await.unwrap();

        let sensors = Sensor::read_by_device_id(&pool, DeviceId(1)).await.unwrap();
        assert!(!sensors.is_empty());
        assert_eq!(sensors.len(), 2);
    }
}
