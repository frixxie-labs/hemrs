use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

/// Represents a new device to be created in the system.
///
/// This struct is used when creating new IoT devices that will report sensor measurements.
/// It contains the basic information needed to register a device before it starts
/// sending measurement data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDevice {
    /// The human-readable name of the device
    pub name: String,
    /// The physical location where the device is deployed
    pub location: String,
}

/// Represents an IoT device in the system.
///
/// Devices are physical units that contain sensors and report measurement data.
/// Each device has a unique ID assigned by the database and descriptive information
/// about its name and location.
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Device {
    /// Unique identifier for the device
    pub id: i32,
    /// The human-readable name of the device
    pub name: String,
    /// The physical location where the device is deployed
    pub location: String,
}

impl Device {
    /// Creates a new Device instance with the specified parameters.
    ///
    /// # Arguments
    /// 
    /// * `id` - The unique identifier for the device
    /// * `name` - The human-readable name of the device
    /// * `location` - The physical location where the device is deployed
    ///
    /// # Returns
    /// 
    /// A new Device instance with the provided values.
    pub fn new(id: i32, name: String, location: String) -> Self {
        Self { id, name, location }
    }

    /// Refreshes the materialized view that caches device-sensor relationships.
    ///
    /// This method should be called after any operations that modify devices or sensors
    /// to ensure the device_sensors materialized view remains up-to-date. The materialized
    /// view is used for performance optimization when querying device-sensor relationships.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// Result indicating success or failure of the refresh operation.
    ///
    /// # Errors
    /// 
    /// Returns an error if the database query fails.
    pub async fn refresh_device_sensors_view(pool: &PgPool) -> Result<()> {
        sqlx::query("REFRESH MATERIALIZED VIEW device_sensors")
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Retrieves all devices from the database.
    ///
    /// This method fetches all registered devices in the system, returning them
    /// as a vector. Useful for listing all available devices in management interfaces.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of all devices, or an error if the query fails.
    ///
    /// # Errors
    /// 
    /// Returns an error if the database query fails or if there are connection issues.
    pub async fn read(pool: &PgPool) -> Result<Vec<Device>> {
        let devices = sqlx::query_as::<_, Device>("SELECT id, name, location FROM devices")
            .fetch_all(pool)
            .await?;
        Ok(devices)
    }

    /// Retrieves a specific device by its ID.
    ///
    /// This method looks up a device using its unique identifier and returns
    /// the device information if found.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    /// * `device_id` - The unique ID of the device to retrieve
    ///
    /// # Returns
    /// 
    /// A Result containing the device if found, or an error if not found or query fails.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device with the specified ID doesn't exist
    /// - The database query fails
    /// - There are connection issues
    pub async fn read_by_id(pool: &PgPool, device_id: i32) -> Result<Device> {
        let device =
            sqlx::query_as::<_, Device>("SELECT id, name, location FROM devices WHERE id = $1")
                .bind(device_id)
                .fetch_one(pool)
                .await?;
        Ok(device)
    }

    /// Deletes this device from the database.
    ///
    /// This method removes the device and all associated data from the system.
    /// Note that this operation may cascade to related measurements and other data
    /// depending on the database schema constraints.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result indicating success or failure of the deletion.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device doesn't exist
    /// - There are foreign key constraints preventing deletion
    /// - The database query fails
    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Updates the device information in the database.
    ///
    /// This method updates the device's name and location with the values
    /// from this instance. After updating, it refreshes the device_sensors
    /// materialized view to maintain consistency.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result indicating success or failure of the update operation.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device with this ID doesn't exist
    /// - The database update query fails
    /// - The materialized view refresh fails
    pub async fn update(self, pool: &PgPool) -> Result<()> {
        sqlx::query("UPDATE devices SET name = $1,location = $2 WHERE id = $3")
            .bind(self.name)
            .bind(self.location)
            .bind(self.id)
            .execute(pool)
            .await?;
        Self::refresh_device_sensors_view(pool).await?;
        Ok(())
    }
}

impl NewDevice {
    /// Creates a new NewDevice instance for device creation.
    ///
    /// # Arguments
    /// 
    /// * `name` - The human-readable name for the new device
    /// * `location` - The physical location where the device will be deployed
    ///
    /// # Returns
    /// 
    /// A new NewDevice instance ready for insertion into the database.
    pub fn new(name: String, location: String) -> Self {
        Self { name, location }
    }

    /// Inserts this new device into the database.
    ///
    /// This method creates a new device record in the database with the provided
    /// name and location. After insertion, it refreshes the device_sensors
    /// materialized view to include the new device in cached queries.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result indicating success or failure of the insertion.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device name or location violates database constraints
    /// - The insertion query fails
    /// - The materialized view refresh fails
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        sqlx::query("INSERT INTO devices (name, location) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.location)
            .execute(pool)
            .await?;
        Device::refresh_device_sensors_view(pool).await?;
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
