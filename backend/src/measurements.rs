use anyhow::Result;
use chrono::{DateTime, Utc};
use sensors::Dht11;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

use crate::{
    create::Create,
    db_connection_pool::{DbConnectionPool, Postgres},
    read::Read,
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

impl Create<PgPool> for Dht11 {
    type Connection = Postgres;

    async fn create(self, connection: Self::Connection) -> Result<()> {
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

impl Read<PgPool> for Measurement {
    type Connection = Postgres;

    async fn read(connection: Self::Connection) -> Result<Self> {
        let pool = connection.get_connection().await;
        let measurement = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestasmp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await?;
        Ok(measurement)
    }
}

impl Read<PgPool> for Vec<Measurement> {
    type Connection = Postgres;

    async fn read(connection: Self::Connection) -> Result<Self> {
        let pool = connection.get_connection().await;
        let dht11_entries =
            sqlx::query_as::<_, Measurement>("SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts DESC LIMIT 1")
                .fetch_all(&pool)
                .await?;
        Ok(dht11_entries)
    }
}
