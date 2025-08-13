use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::devices::Device;

/// Represents a new sensor to be created in the system.
///
/// This struct is used when registering new sensors that will be attached to devices
/// and report measurements. Each sensor measures a specific physical quantity
/// with an associated unit of measurement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSensor {
    /// The human-readable name describing what this sensor measures
    pub name: String,
    /// The unit of measurement for this sensor's readings (e.g., "°C", "m/s", "lux")
    pub unit: String,
}

/// Represents a sensor in the IoT monitoring system.
///
/// Sensors are measurement instruments that can be attached to devices to collect
/// various types of environmental or operational data. Each sensor has a unique ID
/// and defines what it measures along with the unit of measurement.
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Sensor {
    /// Unique identifier for the sensor
    pub id: i32,
    /// The human-readable name describing what this sensor measures
    pub name: String,
    /// The unit of measurement for this sensor's readings (e.g., "°C", "m/s", "lux")
    pub unit: String,
}

impl Sensor {
    /// Creates a new Sensor instance with the specified parameters.
    ///
    /// # Arguments
    /// 
    /// * `id` - The unique identifier for the sensor
    /// * `name` - The human-readable name describing what this sensor measures
    /// * `unit` - The unit of measurement for this sensor's readings
    ///
    /// # Returns
    /// 
    /// A new Sensor instance with the provided values.
    pub fn new(id: i32, name: String, unit: String) -> Self {
        Self { id, name, unit }
    }

    /// Retrieves all sensors from the database.
    ///
    /// This method fetches all registered sensors in the system, returning them
    /// as a vector. Useful for listing all available sensor types in management interfaces.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of all sensors, or an error if the query fails.
    ///
    /// # Errors
    /// 
    /// Returns an error if the database query fails or if there are connection issues.
    pub async fn read(pool: &PgPool) -> Result<Vec<Sensor>> {
        let sensors = sqlx::query_as::<_, Sensor>("SELECT id, name, unit FROM sensors")
            .fetch_all(pool)
            .await?;
        Ok(sensors)
    }

    /// Retrieves a specific sensor by its ID.
    ///
    /// This method looks up a sensor using its unique identifier and returns
    /// the sensor information if found.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    /// * `sensor_id` - The unique ID of the sensor to retrieve
    ///
    /// # Returns
    /// 
    /// A Result containing the sensor if found, or an error if not found or query fails.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The sensor with the specified ID doesn't exist
    /// - The database query fails
    /// - There are connection issues
    pub async fn read_by_id(pool: &PgPool, sensor_id: i32) -> Result<Sensor> {
        let sensors =
            sqlx::query_as::<_, Sensor>("SELECT id, name, unit FROM sensors WHERE id = $1")
                .bind(sensor_id)
                .fetch_one(pool)
                .await?;
        Ok(sensors)
    }

    /// Deletes this sensor from the database.
    ///
    /// This method removes the sensor and all associated measurement data from the system.
    /// Note that this operation may cascade to related measurements depending on
    /// the database schema constraints.
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
    /// - The sensor doesn't exist
    /// - There are foreign key constraints preventing deletion
    /// - The database query fails
    pub async fn delete(self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM sensors WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Updates the sensor information in the database.
    ///
    /// This method updates the sensor's name and unit with the values from this instance.
    /// After updating, it refreshes the device_sensors materialized view to maintain
    /// consistency with cached device-sensor relationships.
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
    /// - The sensor with this ID doesn't exist
    /// - The database update query fails
    /// - The materialized view refresh fails
    pub async fn update(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query("UPDATE sensors SET name = $1,unit = $2 WHERE id = $3")
            .bind(self.name)
            .bind(self.unit)
            .bind(self.id)
            .execute(pool)
            .await?;
        Device::refresh_device_sensors_view(pool).await?;
        Ok(())
    }

    /// Retrieves all sensors associated with a specific device.
    ///
    /// This method queries the device_sensors materialized view to get all sensors
    /// that have reported measurements from the specified device. The results are
    /// ordered by sensor ID for consistent output.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    /// * `device_id` - The ID of the device to get sensors for
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of sensors associated with the device,
    /// or an error if the query fails.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device doesn't exist
    /// - The database query fails
    /// - The materialized view is out of sync (should be refreshed)
    pub async fn read_by_device_id(pool: &PgPool, device_id: i32) -> Result<Vec<Sensor>> {
        let sensors = sqlx::query_as::<_, Sensor>("SELECT ds.sensor_id as id, ds.name, ds.unit from device_sensors ds WHERE ds.device_id = $1 order by ds.sensor_id")
            .bind(device_id)
            .fetch_all(pool)
            .await?;
        Ok(sensors)
    }
}

impl NewSensor {
    /// Creates a new NewSensor instance for sensor creation.
    ///
    /// # Arguments
    /// 
    /// * `name` - The human-readable name describing what this sensor measures
    /// * `unit` - The unit of measurement for this sensor's readings
    ///
    /// # Returns
    /// 
    /// A new NewSensor instance ready for insertion into the database.
    pub fn new(name: String, unit: String) -> Self {
        Self { name, unit }
    }

    /// Inserts this new sensor into the database.
    ///
    /// This method creates a new sensor record in the database with the provided
    /// name and unit. After insertion, it refreshes the device_sensors materialized
    /// view to ensure the new sensor can be properly associated with devices.
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
    /// - The sensor name or unit violates database constraints
    /// - The insertion query fails
    /// - The materialized view refresh fails
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        sqlx::query("INSERT INTO sensors (name, unit) VALUES ($1, $2)")
            .bind(self.name)
            .bind(self.unit)
            .execute(pool)
            .await?;
        Device::refresh_device_sensors_view(pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        devices::{Device, NewDevice},
        measurements::NewMeasurement,
        sensors::{NewSensor, Sensor},
    };

    #[sqlx::test]
    async fn insert(pool: PgPool) {
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();
        let sensors = Sensor::read(&pool).await.unwrap();

        assert!(!sensors.is_empty());
        assert_eq!(sensors.last().unwrap().name, "test");
        assert_eq!(sensors.last().unwrap().unit, "test");
    }

    #[sqlx::test]
    async fn delete(pool: PgPool) {
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();
        let sensors = Sensor::read(&pool).await.unwrap();
        let sensor = sensors.last().unwrap().clone().delete(&pool).await;
        assert!(sensor.is_ok());
    }

    #[sqlx::test]
    async fn update(pool: PgPool) {
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();
        let sensors = Sensor::read(&pool).await.unwrap();
        let sensor = sensors.last().unwrap().clone();
        let sensor = Sensor::new(sensor.id, "test2".to_string(), "test2".to_string());
        sensor.clone().update(&pool).await.unwrap();

        let sensors = Sensor::read(&pool).await.unwrap();
        assert_eq!(sensors.last().unwrap().name, "test2");
        assert_eq!(sensors.last().unwrap().unit, "test2");
    }

    #[sqlx::test]
    async fn read_by_device_id(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.clone().insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test2".to_string(), "test".to_string());
        sensor.clone().insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();
        let measurement2 = NewMeasurement::new(None, 1, 2, 1.0);
        measurement2.insert(&pool).await.unwrap();
        let measurement3 = NewMeasurement::new(None, 1, 2, 1.0);
        measurement3.insert(&pool).await.unwrap();

        Device::refresh_device_sensors_view(&pool).await.unwrap();

        let sensors = Sensor::read_by_device_id(&pool, 1).await.unwrap();
        assert!(!sensors.is_empty());
        assert_eq!(sensors.len(), 2);
    }
}
