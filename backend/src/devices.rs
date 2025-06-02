use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDevice {
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

    pub async fn read(pool: &PgPool) -> Result<Vec<Devices>> {
        let devices = sqlx::query_as::<_, Devices>("SELECT id, name, location FROM devices")
            .fetch_all(pool)
            .await?;
        Ok(devices)
    }

    pub async fn read_by_id(pool: &PgPool, device_id: i32) -> Result<Devices> {
        let device = sqlx::query_as::<_, Devices>("SELECT id, name, location FROM devices WHERE id = $1")
            .bind(device_id)
            .fetch_one(pool)
            .await?;
        Ok(device)
    }

    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(self, pool: &PgPool) -> Result<()> {
        sqlx::query("UPDATE devices SET name = $1,location = $2 WHERE id = $3")
            .bind(self.name)
            .bind(self.location)
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl NewDevice {
    pub fn new(name: String, location: String) -> Self {
        Self { name, location }
    }

    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        sqlx::query("INSERT INTO devices (name, location) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.location)
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::devices::{Devices, NewDevice};

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let devices = Devices::read(&pool).await.unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].name, "test");
        assert_eq!(devices[0].location, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.clone().insert(&pool).await.unwrap();
        let devices = Devices::read(&pool).await.unwrap();
        let device = devices[0].clone().delete(&pool).await;
        assert!(device.is_ok());

        let devices = Devices::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.clone().insert(&pool).await.unwrap();
        let devices = Devices::read(&pool).await.unwrap();
        let device = devices[0].clone();
        let device = Devices::new(device.id, "test2".to_string(), "test2".to_string());
        device.clone().update(&pool).await.unwrap();

        let devices = Devices::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "test2");
        assert_eq!(devices[0].location, "test2");
    }
}
