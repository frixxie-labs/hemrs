use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewMeasurement {
    pub device: i32,
    pub sensor: i32,
    pub measurement: f32,
}

impl NewMeasurement {
    pub fn new(device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            device,
            sensor,
            measurement,
        }
    }

    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(self.device)
            .bind(self.sensor)
            .bind(self.measurement)
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NewMeasurements {
    Measurement(NewMeasurement),
}

impl fmt::Display for NewMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.device, self.sensor, self.measurement)
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Measurement {
    timestamp: DateTime<Utc>,
    value: f32,
    unit: String,
    device_name: String,
    device_location: String,
    sensor_name: String,
}

impl Measurement {
    pub async fn read_by_device_id_and_sensor_id(
        device_id: i32,
        sensor_id: i32,
        pool: &PgPool,
    ) -> Result<Vec<Self>> {
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts",
        )
        .bind(device_id)
        .bind(sensor_id)
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_latest_by_device_id_and_sensor_id(
        device_id: i32,
        sensor_id: i32,
        pool: &PgPool,
    ) -> Result<Self> {
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts desc LIMIT 1",
        )
        .bind(device_id)
        .bind(sensor_id)
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_by_device_id(device_id: i32, pool: &PgPool) -> Result<Vec<Self>> {
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) ORDER BY ts",
        )
        .bind(device_id)
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_all(pool: &PgPool) -> Result<Vec<Measurement>> {
        let dht11_entries =
            sqlx::query_as::<_, Measurement>("SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts")
                .fetch_all(pool)
                .await?;
        Ok(dht11_entries)
    }

    pub async fn read_latest(pool: &PgPool) -> Result<Self> {
        let measurement = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts DESC LIMIT 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(measurement)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::measurements::NewMeasurement;
    use crate::sensors::NewSensor;
    use crate::{devices::NewDevice, measurements::Measurement};

    #[sqlx::test]
    fn measurement_insert(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();
    }

    #[sqlx::test]
    fn measurements_read(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_all(&pool).await.unwrap();
        assert!(!measurements.is_empty());
    }
}
