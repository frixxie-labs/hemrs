use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSensor {
    pub name: String,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Sensors {
    pub id: i32,
    pub name: String,
    pub unit: String,
}

impl Sensors {
    pub fn new(id: i32, name: String, unit: String) -> Self {
        Self { id, name, unit }
    }

    pub async fn read(pool: &PgPool) -> Result<Vec<Sensors>> {
        let devices = sqlx::query_as::<_, Sensors>("SELECT id, name, unit FROM sensors")
            .fetch_all(pool)
            .await?;
        Ok(devices)
    }

    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM sensors WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query("UPDATE sensors SET name = $1,unit = $2 WHERE id = $3")
            .bind(self.name)
            .bind(self.unit)
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn read_by_device_id(pool: &PgPool, device_id: i32) -> Result<Vec<Sensors>> {
        let sensors = sqlx::query_as::<_, Sensors>("SELECT DISTINCT s.id, s.name, s.unit FROM sensors s JOIN measurements m ON s.id = m.sensor_id WHERE m.device_id = $1")
            .bind(device_id)
            .fetch_all(pool)
            .await?;
        Ok(sensors)
    }
}

impl NewSensor {
    pub fn new(name: String, unit: String) -> Self {
        Self { name, unit }
    }

    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        sqlx::query("INSERT INTO sensors (name, unit) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.unit)
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        devices::NewDevice,
        measurements::NewMeasurement,
        sensors::{NewSensor, Sensors},
    };

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();
        let sensors = Sensors::read(&pool).await.unwrap();

        assert!(!sensors.is_empty());
        assert_eq!(sensors.last().unwrap().name, "test");
        assert_eq!(sensors.last().unwrap().unit, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();
        let sensors = Sensors::read(&pool).await.unwrap();
        let sensor = sensors.last().unwrap().clone().delete(&pool).await;
        assert!(sensor.is_ok());
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();
        let sensors = Sensors::read(&pool).await.unwrap();
        let sensor = sensors.last().unwrap().clone();
        let sensor = Sensors::new(sensor.id, "test2".to_string(), "test2".to_string());
        sensor.clone().update(&pool).await.unwrap();

        let sensors = Sensors::read(&pool).await.unwrap();
        assert_eq!(sensors.last().unwrap().name, "test2");
        assert_eq!(sensors.last().unwrap().unit, "test2");
    }

    #[sqlx::test]
    async fn read_by_device_id(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.clone().insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test2".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();
        let measurement2 = NewMeasurement::new(None, 1, 2, 1.0);
        measurement2.insert(&pool).await.unwrap();
        let measurement3 = NewMeasurement::new(None, 1, 2, 1.0);
        measurement3.insert(&pool).await.unwrap();

        let sensors = Sensors::read_by_device_id(&pool, 1).await.unwrap();
        assert!(!sensors.is_empty());
        assert_eq!(sensors.len(), 2);
    }
}
