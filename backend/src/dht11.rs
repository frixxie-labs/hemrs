use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dht11 {
    pub room: String,
    pub temp: f32,
    pub hum: f32,
}

impl Dht11 {
    pub fn new(room: String, temp: f32, hum: f32) -> Self {
        Self { room, temp, hum }
    }

    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        let transaction = pool.begin().await?;
        let dht11_temperature_id: i32 =
            sqlx::query_scalar("SELECT id from sensors where name = 'dht11_temperature'")
                .fetch_one(pool)
                .await?;
        let dht11_humidity_id: i32 =
            sqlx::query_scalar("SELECT id from sensors where name = 'dht11_humidity'")
                .fetch_one(pool)
                .await?;
        let device_id: i32 = sqlx::query_scalar("SELECT id from devices where location = ($1)")
            .bind(self.room.clone())
            .fetch_one(pool)
            .await?;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(device_id)
            .bind(dht11_temperature_id)
            .bind(self.temp)
            .execute(pool)
            .await?;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(device_id)
            .bind(dht11_humidity_id)
            .bind(self.hum)
            .execute(pool)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

}

impl fmt::Display for Dht11 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.room, self.temp, self.hum,)
    }
}
