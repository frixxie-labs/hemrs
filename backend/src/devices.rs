use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::{
    create::Create,
    db_connection_pool::{DbConnectionPool, Postgres},
    read::Read,
};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Device {
    name: String,
    location: String,
}

impl Device {
    pub fn new(name: String, location: String) -> Self {
        Self { name, location }
    }
}

impl Read<PgPool> for Vec<Device> {
    type Connection = Postgres;

    async fn read(connection: Self::Connection) -> anyhow::Result<Self> {
        let pool = connection.get_connection().await;
        let devices = sqlx::query_as::<_, Device>("SELECT (name, location) FROM devices")
            .fetch_all(&pool)
            .await?;
        Ok(devices)
    }
}

impl Create<PgPool> for Device {
    type Connection = Postgres;

    async fn create(self, connection: Self::Connection) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO devices (name, location) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.location)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
