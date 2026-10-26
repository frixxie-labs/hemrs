use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt;
use tracing::{debug, info};
use utoipa::ToSchema;

/// Identifies a device; serialized and stored as a plain integer.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    sqlx::Type,
    ToSchema,
)]
#[serde(transparent)]
#[sqlx(transparent)]
#[schema(value_type = i32)]
pub struct DeviceId(pub i32);

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

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
    pub id: DeviceId,
    /// The human-readable name of the device
    pub name: String,
    /// The physical location where the device is deployed
    pub location: String,
}

impl Device {
    pub async fn refresh_device_sensors_view(pool: &PgPool) -> Result<()> {
        debug!("Refreshing device_sensors materialized view");
        sqlx::query!("REFRESH MATERIALIZED VIEW device_sensors")
            .execute(pool)
            .await?;
        debug!("Materialized view refreshed");
        Ok(())
    }

    pub async fn read(pool: &PgPool) -> Result<Vec<Device>> {
        let devices = sqlx::query_as!(
            Device,
            "SELECT id AS \"id: DeviceId\", name, location FROM devices"
        )
        .fetch_all(pool)
        .await?;
        Ok(devices)
    }

    pub async fn read_by_id(pool: &PgPool, device_id: DeviceId) -> Result<Device> {
        let device = sqlx::query_as!(
            Device,
            "SELECT id AS \"id: DeviceId\", name, location FROM devices WHERE id = $1",
            device_id.0
        )
        .fetch_one(pool)
        .await?;
        Ok(device)
    }

    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        info!(device_id = %self.id, device_name = %self.name, "Deleting device");
        sqlx::query!("DELETE FROM devices WHERE id = $1", self.id.0)
            .execute(pool)
            .await?;
        info!(device_id = %self.id, "Device deleted");
        Ok(())
    }

    pub async fn update(self, pool: &PgPool) -> Result<()> {
        info!(device_id = %self.id, name = %self.name, location = %self.location, "Updating device");
        sqlx::query!(
            "UPDATE devices SET name = $1,location = $2 WHERE id = $3",
            self.name,
            self.location,
            self.id.0
        )
        .execute(pool)
        .await?;
        Self::refresh_device_sensors_view(pool).await?;
        info!(device_id = %self.id, "Device updated");
        Ok(())
    }
}

impl NewDevice {
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
    #[quickcheck_macros::quickcheck]
    fn device_id_serializes_as_plain_integer(raw: i32) -> bool {
        let json = serde_json::to_string(&super::DeviceId(raw)).unwrap();
        json == raw.to_string()
            && serde_json::from_str::<super::DeviceId>(&json).unwrap() == super::DeviceId(raw)
    }

    use sqlx::PgPool;

    use crate::devices::{Device, NewDevice};

    // ── sqlx integration tests ────────────────────────────────────────────────

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let device = NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
        };
        device.insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].name, "test");
        assert_eq!(devices[0].location, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let device = NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
        };
        device.clone().insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        let device = devices[0].clone().delete(&pool).await;
        assert!(device.is_ok());

        let devices = Device::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let device = NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
        };
        device.clone().insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        let device = devices[0].clone();
        let device = Device {
            id: device.id,
            name: "test2".to_string(),
            location: "test2".to_string(),
        };
        device.clone().update(&pool).await.unwrap();

        let devices = Device::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "test2");
        assert_eq!(devices[0].location, "test2");
    }
}
