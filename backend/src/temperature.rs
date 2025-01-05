use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewTemperature {
    pub room: String,
    pub temperature: f32,
}

impl NewTemperature {
    pub fn new(room: String, temperature: f32) -> Self {
        Self { room, temperature }
    }

    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        let transaction = pool.begin().await?;
        let sensor_id: i32 =
            sqlx::query_scalar("SELECT id from sensors where name = 'temperature'")
                .fetch_one(pool)
                .await?;
        let device_id: i32 = sqlx::query_scalar("SELECT id from devices where location = ($1)")
            .bind(self.room.clone())
            .fetch_one(pool)
            .await?;
        sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(device_id)
            .bind(sensor_id)
            .bind(self.temperature)
            .execute(pool)
            .await?;
        transaction.commit().await?;
        Ok(())
    }
}

impl fmt::Display for NewTemperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.room, self.temperature)
    }
}
