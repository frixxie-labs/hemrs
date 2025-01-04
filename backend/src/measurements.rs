use anyhow::Result;
use chrono::{DateTime, Utc};
use sensors::{Dht11, Measurement as GenericMeasurement, Temperature};
use serde::Serialize;
use sqlx::FromRow;

use crate::{
    database::{
        db_connection_pool::{DbConnectionPool, Postgres},
        insert::Insert,
        query::Query,
        read::Read,
    },
    handlers::InnerMeasurementQuery,
};

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
        connection: Postgres,
    ) -> Result<Vec<Self>> {
        let pool = connection.get_connection().await;
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts",
        )
        .bind(device_id)
        .bind(sensor_id)
        .fetch_all(&pool)
        .await?;
        Ok(res)
    }

    pub async fn read_by_device_id(device_id: i32, connection: Postgres) -> Result<Vec<Self>> {
        let pool = connection.get_connection().await;
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) ORDER BY ts",
        )
        .bind(device_id)
        .fetch_all(&pool)
        .await?;
        Ok(res)
    }
}

impl Insert<Postgres> for Dht11 {
    async fn insert(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        let transaction = pool.begin().await?;
        let dht11_temperature_id: i32 =
            sqlx::query_scalar("SELECT id from sensors where name = 'dht11_temperature'")
                .fetch_one(&pool)
                .await?;
        let dht11_humidity_id: i32 =
            sqlx::query_scalar("SELECT id from sensors where name = 'dht11_humidity'")
                .fetch_one(&pool)
                .await?;
        let device_id: i32 = sqlx::query_scalar("SELECT id from devices where location = ($1)")
            .bind(self.room.clone())
            .fetch_one(&pool)
            .await?;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(device_id)
            .bind(dht11_temperature_id)
            .bind(self.temp)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(device_id)
            .bind(dht11_humidity_id)
            .bind(self.hum)
            .execute(&pool)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}

impl Insert<Postgres> for GenericMeasurement {
    async fn insert(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(self.device)
            .bind(self.sensor)
            .bind(self.measurement)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Insert<Postgres> for Temperature {
    async fn insert(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        let transaction = pool.begin().await?;
        let sensor_id: i32 =
            sqlx::query_scalar("SELECT id from sensors where name = 'temperature'")
                .fetch_one(&pool)
                .await?;
        let device_id: i32 = sqlx::query_scalar("SELECT id from devices where location = ($1)")
            .bind(self.room.clone())
            .fetch_one(&pool)
            .await?;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(device_id)
            .bind(sensor_id)
            .bind(self.temperature)
            .execute(&pool)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}

impl Read<Postgres> for Measurement {
    async fn read(connection: Postgres) -> Result<Self> {
        let pool = connection.get_connection().await;
        let measurement = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await?;
        Ok(measurement)
    }
}

impl Read<Postgres> for Vec<Measurement> {
    async fn read(connection: Postgres) -> Result<Self> {
        let pool = connection.get_connection().await;
        let dht11_entries =
            sqlx::query_as::<_, Measurement>("SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts")
                .fetch_all(&pool)
                .await?;
        Ok(dht11_entries)
    }
}

impl Query<Postgres, Measurement> for InnerMeasurementQuery {
    async fn query(self, connection: Postgres) -> anyhow::Result<Measurement> {
        let pool = connection.get_connection().await;
        let sensor_id: i32 = sqlx::query_scalar("SELECT id from sensors where name = ($1)")
            .bind(self.sensor_name)
            .fetch_one(&pool)
            .await?;
        let device_id: i32 = sqlx::query_scalar("SELECT id from devices where name = ($1)")
            .bind(self.device_name)
            .fetch_one(&pool)
            .await?;
        let res = sqlx::query_as::<_, Measurement>("SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts")
            .bind(device_id)
            .bind(sensor_id)
            .fetch_one(&pool)
            .await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use sensors::Measurement as GenericMeasurement;
    use sensors::{Dht11, Temperature};
    use sqlx::PgPool;

    use crate::database::insert::Insert;
    use crate::database::query::Query;
    use crate::handlers::InnerMeasurementQuery;
    use crate::{
        database::{db_connection_pool::Postgres, read::Read},
        devices::NewDevice,
        measurements::Measurement,
    };

    #[sqlx::test]
    fn dht11_insert(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let postgres = Postgres::new(pool);
        let dht11_measurement = Dht11::new("test".to_string(), 10.0, 20.0);
        dht11_measurement.insert(postgres).await.unwrap();
    }

    #[sqlx::test]
    fn tempterture_insert(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let postgres = Postgres::new(pool);
        let temperature_measurement = Temperature::new("test".to_string(), 10.0);
        temperature_measurement.insert(postgres).await.unwrap();
    }

    #[sqlx::test]
    fn measurement_insert(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let postgres = Postgres::new(pool);
        let measurement = GenericMeasurement::new(1, 1, 1.0);
        measurement.insert(postgres).await.unwrap();
    }

    #[sqlx::test]
    fn measurements_read(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let postgres = Postgres::new(pool);
        let dht11_measurement = Dht11::new("test".to_string(), 10.0, 20.0);
        dht11_measurement.insert(postgres.clone()).await.unwrap();

        let measurements = Vec::<Measurement>::read(postgres.clone()).await.unwrap();
        assert!(!measurements.is_empty());

        let _measurement = Measurement::read(postgres.clone()).await.unwrap();
    }

    #[sqlx::test]
    fn measurements_query(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let postgres = Postgres::new(pool);
        let dht11_measurement = Dht11::new("test".to_string(), 10.0, 20.0);
        dht11_measurement.insert(postgres.clone()).await.unwrap();

        let query = InnerMeasurementQuery {
            device_name: "test".to_string(),
            sensor_name: "dht11_temperature".to_string(),
        };

        let measurement = query.query(postgres.clone()).await;
        assert!(measurement.is_ok());
        let result = measurement.unwrap();
        assert_eq!(result.device_name, "test");
        assert_eq!(result.sensor_name, "dht11_temperature");
    }
}
