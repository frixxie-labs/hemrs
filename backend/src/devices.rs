use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fmt;
use tracing::{debug, info};
use utoipa::ToSchema;

use crate::coordinates::Coordinates;

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
    /// Geographic position of the device, if known
    #[serde(default)]
    pub coordinates: Option<Coordinates>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Device {
    /// Unique identifier for the device
    pub id: DeviceId,
    /// The human-readable name of the device
    pub name: String,
    /// The physical location where the device is deployed
    pub location: String,
    /// Geographic position of the device, if known
    #[serde(default)]
    pub coordinates: Option<Coordinates>,
}

struct DeviceRow {
    id: DeviceId,
    name: String,
    location: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

impl TryFrom<DeviceRow> for Device {
    type Error = anyhow::Error;

    fn try_from(row: DeviceRow) -> Result<Self> {
        Ok(Device {
            id: row.id,
            name: row.name,
            location: row.location,
            coordinates: Coordinates::from_columns(row.latitude, row.longitude)?,
        })
    }
}

fn split(coordinates: Option<Coordinates>) -> (Option<f64>, Option<f64>) {
    coordinates
        .map(|c| (Some(c.latitude()), Some(c.longitude())))
        .unwrap_or_default()
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
        sqlx::query_as!(
            DeviceRow,
            "SELECT id AS \"id: DeviceId\", name, location, \
             ST_Y(geom::geometry) AS latitude, ST_X(geom::geometry) AS longitude \
             FROM devices"
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(Device::try_from)
        .collect()
    }

    pub async fn read_by_id(pool: &PgPool, device_id: DeviceId) -> Result<Device> {
        sqlx::query_as!(
            DeviceRow,
            "SELECT id AS \"id: DeviceId\", name, location, \
             ST_Y(geom::geometry) AS latitude, ST_X(geom::geometry) AS longitude \
             FROM devices WHERE id = $1",
            device_id.0
        )
        .fetch_one(pool)
        .await?
        .try_into()
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
        let (latitude, longitude) = split(self.coordinates);
        sqlx::query!(
            "UPDATE devices SET name = $1, location = $2, \
             geom = ST_SetSRID(ST_MakePoint($4, $5), 4326)::geography \
             WHERE id = $3",
            self.name,
            self.location,
            self.id.0,
            longitude,
            latitude
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
        let (latitude, longitude) = split(self.coordinates);
        sqlx::query!(
            "INSERT INTO devices (name, location, geom) \
             VALUES ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography)",
            self.name,
            self.location,
            longitude,
            latitude
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

    #[test]
    fn device_without_coordinates_still_deserializes() {
        let device: NewDevice = serde_json::from_str(r#"{"name":"n","location":"l"}"#).unwrap();
        assert_eq!(device.coordinates, None);
    }

    use sqlx::PgPool;

    use crate::{
        coordinates::Coordinates,
        devices::{Device, NewDevice},
    };

    #[sqlx::test]
    async fn coordinates_round_trip_through_insert_and_update(pool: PgPool) {
        let home = Coordinates::new(59.3293, 18.0686).unwrap();
        NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
            coordinates: Some(home),
        }
        .insert(&pool)
        .await
        .unwrap();
        let device = Device::read(&pool).await.unwrap().remove(0);
        assert_eq!(device.coordinates, Some(home));

        Device {
            coordinates: None,
            ..device.clone()
        }
        .update(&pool)
        .await
        .unwrap();
        let device = Device::read_by_id(&pool, device.id).await.unwrap();
        assert_eq!(device.coordinates, None);
    }

    // ── sqlx integration tests ────────────────────────────────────────────────

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let device = NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
            coordinates: None,
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
            coordinates: None,
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
            coordinates: None,
        };
        device.clone().insert(&pool).await.unwrap();
        let devices = Device::read(&pool).await.unwrap();
        let device = devices[0].clone();
        let device = Device {
            id: device.id,
            name: "test2".to_string(),
            location: "test2".to_string(),
            coordinates: None,
        };
        device.clone().update(&pool).await.unwrap();

        let devices = Device::read(&pool).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "test2");
        assert_eq!(devices[0].location, "test2");
    }
}
