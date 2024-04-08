use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    create::Create,
    db_connection_pool::{DbConnectionPool, Postgres},
    delete::Delete,
    read::Read,
    update::Update,
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

impl Read<Postgres> for Vec<Device> {
    async fn read(connection: Postgres) -> anyhow::Result<Self> {
        let pool = connection.get_connection().await;
        let devices = sqlx::query_as::<_, Device>("SELECT name, location FROM devices")
            .fetch_all(&pool)
            .await?;
        Ok(devices)
    }
}

impl Create<Postgres> for Device {
    async fn create(self, connection: Postgres) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO devices (name, location) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.location)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Delete<Postgres> for Device {
    async fn delete(self, connection: Postgres) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("DELETE FROM devices WHERE name = $1")
            .bind(self.name)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Update<Postgres> for Device {}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        create::Create, db_connection_pool::Postgres, delete::Delete, devices::Device, read::Read,
        update::Update,
    };

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let device = Device::new("test".to_string(), "test".to_string());
        device.create(postgres).await.unwrap();
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let device = Device::new("test".to_string(), "test".to_string());
        device.clone().create(postgres.clone()).await.unwrap();
        device.delete(postgres).await.unwrap();
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let mut device = Device::new("test".to_string(), "test".to_string());
        device.clone().create(postgres.clone()).await.unwrap();
        device.name = "newtest".to_string();
        device.update(postgres).await.unwrap();
    }

    #[sqlx::test]
    async fn read(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let device = Device::new("test".to_string(), "test".to_string());
        device.create(postgres.clone()).await.unwrap();

        let devices: Vec<Device> = Vec::<Device>::read(postgres).await.unwrap();
        assert!(devices.len() >= 1);
    }
}
