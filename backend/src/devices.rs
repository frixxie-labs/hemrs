use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{debug, info};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewDevice {
    /// The human-readable name of the device
    pub name: String,
    /// The physical location where the device is deployed
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Device {
    /// Unique identifier for the device
    pub id: i32,
    /// The human-readable name of the device
    pub name: String,
    /// The physical location where the device is deployed
    pub location: String,
}

impl Device {
    pub fn new(id: i32, name: String, location: String) -> Self {
        Self { id, name, location }
    }

    pub async fn refresh_device_sensors_view(pool: &PgPool) -> Result<()> {
        debug!("Refreshing device_sensors materialized view");
        sqlx::query!("REFRESH MATERIALIZED VIEW device_sensors")
            .execute(pool)
            .await?;
        debug!("Materialized view refreshed");
        Ok(())
    }

    pub async fn read(pool: &PgPool) -> Result<Vec<Device>> {
        let devices = sqlx::query_as!(Device, "SELECT id, name, location FROM devices")
            .fetch_all(pool)
            .await?;
        Ok(devices)
    }

    pub async fn read_by_id(pool: &PgPool, device_id: i32) -> Result<Device> {
        let device = sqlx::query_as!(
            Device,
            "SELECT id, name, location FROM devices WHERE id = $1",
            device_id
        )
        .fetch_one(pool)
        .await?;
        Ok(device)
    }

    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        info!(device_id = self.id, device_name = %self.name, "Deleting device");
        sqlx::query!("DELETE FROM devices WHERE id = $1", self.id)
            .execute(pool)
            .await?;
        info!(device_id = self.id, "Device deleted");
        Ok(())
    }

    pub async fn update(self, pool: &PgPool) -> Result<()> {
        info!(device_id = self.id, name = %self.name, location = %self.location, "Updating device");
        sqlx::query!(
            "UPDATE devices SET name = $1,location = $2 WHERE id = $3",
            self.name,
            self.location,
            self.id
        )
        .execute(pool)
        .await?;
        Self::refresh_device_sensors_view(pool).await?;
        info!(device_id = self.id, "Device updated");
        Ok(())
    }
}

impl NewDevice {
    pub fn new(name: String, location: String) -> Self {
        Self { name, location }
    }

    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        info!(name = %self.name, location = %self.location, "Inserting new device");
        sqlx::query!(
            "INSERT INTO devices (name, location) VALUES ($1, $2)",
            self.name,
            self.location
        )
        .execute(pool)
        .await?;
        Device::refresh_device_sensors_view(pool).await?;
        info!(name = %self.name, "Device inserted");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::devices::{Device, NewDevice};

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].name, "test");
        assert_eq!(devices[0].location, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.clone().insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        let device = devices[0].clone().delete(&pool).await;
        assert!(device.is_ok());

        let devices = Device::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.clone().insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        let device = devices[0].clone();
        let device = Device::new(device.id, "test2".to_string(), "test2".to_string());
        device.clone().update(&pool).await.unwrap();

        let devices = Device::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "test2");
        assert_eq!(devices[0].location, "test2");
    }
}
