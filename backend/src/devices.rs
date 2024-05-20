use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::database::{
    db_connection_pool::{DbConnectionPool, Postgres},
    delete::Delete,
    insert::Insert,
    read::Read,
    update::Update,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Devices {
    pub id: i32,
    pub name: String,
    pub location: String,
}

impl Devices {
    pub fn new(id: i32, name: String, location: String) -> Self {
        Self { id, name, location }
    }
}

impl Device {
    pub fn new(name: String, location: String) -> Self {
        Self { name, location }
    }
}

impl Read<Postgres> for Vec<Devices> {
    async fn read(connection: Postgres) -> anyhow::Result<Self> {
        let pool = connection.get_connection().await;
        let devices = sqlx::query_as::<_, Devices>("SELECT id, name, location FROM devices")
            .fetch_all(&pool)
            .await?;
        Ok(devices)
    }
}

impl Insert<Postgres> for Device {
    async fn insert(self, connection: Postgres) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("INSERT INTO devices (name, location) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.location)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Delete<Postgres> for Devices {
    async fn delete(self, connection: Postgres) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(self.id)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

impl Update<Postgres> for Devices {
    async fn update(self, connection: Postgres) -> anyhow::Result<()> {
        let pool = connection.get_connection().await;
        sqlx::query("UPDATE devices SET name = $1,location = $2 WHERE id = $3")
            .bind(self.name)
            .bind(self.location)
            .bind(self.id)
            .execute(&pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        database::{
            db_connection_pool::Postgres, delete::Delete, insert::Insert, read::Read,
            update::Update,
        },
        devices::{Device, Devices},
    };

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let device = Device::new("test".to_string(), "test".to_string());
        device.insert(postgres.clone()).await.unwrap();
        let devices = Vec::<Devices>::read(postgres.clone()).await.unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].name, "test");
        assert_eq!(devices[0].location, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let device = Device::new("test".to_string(), "test".to_string());
        device.clone().insert(postgres.clone()).await.unwrap();
        let devices = Vec::<Devices>::read(postgres.clone()).await.unwrap();
        let device = devices[0].clone().delete(postgres.clone()).await;
        assert!(device.is_ok());

        let devices = Vec::<Devices>::read(postgres.clone()).await.unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let postgres = Postgres::new(pool);

        let device = Device::new("test".to_string(), "test".to_string());
        device.clone().insert(postgres.clone()).await.unwrap();
        let devices = Vec::<Devices>::read(postgres.clone()).await.unwrap();
        let device = devices[0].clone();
        let device = Devices::new(device.id, "test2".to_string(), "test2".to_string());
        device.clone().update(postgres.clone()).await.unwrap();

        let devices = Vec::<Devices>::read(postgres.clone()).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "test2");
        assert_eq!(devices[0].location, "test2");
    }
}
