use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::database::{
    create::Create, db_connection_pool::{DbConnectionPool, Postgres}, delete::Delete, read::Read, update::Update,
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
        sqlx::query("INSERT INTO sensors (name, unit) VALUES ($1, $2)")
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
        sqlx::query("DELETE FROM sensors WHERE name = $1")
            .bind(self.name)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Update<Postgres> for Sensor {}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        database::{create::Create, db_connection_pool::Postgres, delete::Delete, read::Read, update::Update},
        sensors::Sensor,
    };

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.create(postgres).await.unwrap();
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.clone().create(postgres.clone()).await.unwrap();
        sensor.delete(postgres).await.unwrap();
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let mut sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.clone().create(postgres.clone()).await.unwrap();
        sensor.name = "newtest".to_string();
        sensor.update(postgres).await.unwrap();
    }

    #[sqlx::test]
    async fn read(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let sensor = Sensor::new("test".to_string(), "test".to_string());
        sensor.create(postgres.clone()).await.unwrap();

        let sensors: Vec<Sensor> = Vec::<Sensor>::read(postgres).await.unwrap();
        assert!(sensors.len() >= 1);
    }
}
