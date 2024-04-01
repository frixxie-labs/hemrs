use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::{
    create::Create,
    db_connection_pool::{DbConnectionPool, Postgres},
    read::Read,
};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Sensor {
    name: String,
    unit: String,
}

impl Sensor {
    pub fn new(name: String, unit: String) -> Self {
        Self { name, unit }
    }
}

impl Read<PgPool> for Vec<Sensor> {
    type Connection = Postgres;

    async fn read(connection: Self::Connection) -> Result<Self> {
        let pool = connection.get_connection().await;
        let devices = sqlx::query_as::<_, Sensor>("SELECT (name, unit) FROM devices")
            .fetch_all(&pool)
            .await?;
        Ok(devices)
    }
}

impl Create<PgPool> for Sensor {
    type Connection = Postgres;

    async fn create(self, connection: Self::Connection) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO devices (name, unit) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.unit)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
