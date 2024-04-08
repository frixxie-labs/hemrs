use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    create::Create,
    db_connection_pool::{DbConnectionPool, Postgres},
    delete::Delete,
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

impl Read<Postgres> for Vec<Sensor> {
    async fn read(connection: Postgres) -> Result<Self> {
        let pool = connection.get_connection().await;
        let devices = sqlx::query_as::<_, Sensor>("SELECT name, unit FROM sensors")
            .fetch_all(&pool)
            .await?;
        Ok(devices)
    }
}

impl Create<Postgres> for Sensor {
    async fn create(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO devices (name, unit) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.unit)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Delete<Postgres> for Sensor {
    async fn delete(self, connection: Postgres) -> Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("DELETE FROM devices WHERE name = $1")
            .bind(self.name)
            .execute(&pool)
            .await?;
        Ok(())
    }
}
