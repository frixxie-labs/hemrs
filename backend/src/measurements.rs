use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt;

/// Represents a new measurement to be inserted into the system.
///
/// This struct is used when devices report new sensor readings. It contains
/// the measurement value along with metadata about which device and sensor
/// produced the reading, and optionally a specific timestamp.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewMeasurement {
    /// Optional timestamp for the measurement. If None, current time will be used
    pub timestamp: Option<DateTime<Utc>>,
    /// ID of the device that produced this measurement
    pub device: i32,
    /// ID of the sensor that produced this measurement
    pub sensor: i32,
    /// The actual measurement value
    pub measurement: f32,
}

impl NewMeasurement {
    /// Creates a new measurement record.
    ///
    /// # Arguments
    /// 
    /// * `ts` - Optional timestamp for the measurement. If None, current time will be used during insertion
    /// * `device` - ID of the device that produced this measurement
    /// * `sensor` - ID of the sensor that produced this measurement
    /// * `measurement` - The actual measurement value
    ///
    /// # Returns
    /// 
    /// A new NewMeasurement instance ready for insertion.
    pub fn new(ts: Option<DateTime<Utc>>, device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            timestamp: ts,
            device,
            sensor,
            measurement,
        }
    }

    /// Inserts this measurement into the database.
    ///
    /// This method stores the measurement in the database. If a timestamp was provided,
    /// it uses that timestamp. Otherwise, it uses the current database time (CURRENT_TIMESTAMP).
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
    /// - The device or sensor IDs don't exist (foreign key constraint violation)
    /// - The database insertion query fails
    /// - There are connection issues
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        match self.timestamp {
            Some(t) => {
                sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES ($1, $2, $3, $4)")
            .bind(t)
            .bind(self.device)
            .bind(self.sensor)
            .bind(self.measurement)
            .execute(pool)
            .await?;
                Ok(())
            }
            None => {
                sqlx::query("INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)")
            .bind(self.device)
            .bind(self.sensor)
            .bind(self.measurement)
            .execute(pool)
            .await?;
                Ok(())
            }
        }
    }
}

/// Flexible container for measurement data that can handle single or multiple measurements.
///
/// This enum is used in API endpoints to accept either a single measurement or
/// a batch of measurements in the same request. The untagged serde annotation
/// allows automatic deserialization based on the JSON structure.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NewMeasurements {
    /// A single measurement
    Measurement(NewMeasurement),
    /// A batch of measurements for bulk insertion
    Measurements(Vec<NewMeasurement>),
}

impl fmt::Display for NewMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.device, self.sensor, self.measurement)
    }
}

/// Represents a complete measurement record with associated metadata.
///
/// This struct is used when retrieving measurement data from the database.
/// It includes not only the measurement value and timestamp, but also
/// denormalized information about the device and sensor for efficient querying.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Measurement {
    /// When the measurement was recorded
    pub timestamp: DateTime<Utc>,
    /// The measurement value
    pub value: f32,
    /// Unit of measurement from the associated sensor
    pub unit: String,
    /// Name of the device that produced this measurement
    pub device_name: String,
    /// Location of the device that produced this measurement
    pub device_location: String,
    /// Name of the sensor that produced this measurement
    pub sensor_name: String,
}

/// Statistical summary of measurement data for a specific device-sensor combination.
///
/// This struct provides aggregate statistics that are useful for monitoring
/// and analysis purposes, such as detecting anomalies or understanding
/// data distribution patterns.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MeasurementStats {
    /// Minimum recorded value
    min: f32,
    /// Maximum recorded value
    max: f32,
    /// Total number of measurements
    count: i64,
    /// Average (mean) of all measurements
    avg: f64,
    /// Standard deviation of measurements
    stddev: f64,
    /// Variance of measurements
    variance: f64,
}

impl Measurement {
    /// Retrieves the latest measurement for each device-sensor combination.
    ///
    /// This method uses DISTINCT ON to get only the most recent measurement
    /// for each unique device-sensor pair, which is useful for dashboard views
    /// showing current status of all sensors.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of the latest measurements for each device-sensor pair.
    ///
    /// # Errors
    /// 
    /// Returns an error if the database query fails or connection issues occur.
    pub async fn read_all_latest_measurements(pool: &PgPool) -> Result<Vec<Measurement>> {
        let res = sqlx::query_as::<_, Measurement>(
               "SELECT DISTINCT ON (m.device_id, m.sensor_id) m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name
                FROM measurements m
                JOIN devices d ON m.device_id = d.id
                JOIN sensors s ON m.sensor_id = s.id
                ORDER BY m.device_id, m.sensor_id, ts DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    /// Calculates statistical summary for measurements from a specific device-sensor combination.
    ///
    /// This method computes aggregate statistics (min, max, count, average, standard deviation,
    /// and variance) for all measurements from the specified device-sensor pair. Useful for
    /// monitoring data quality and detecting anomalies.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    /// * `device_id` - ID of the device to analyze
    /// * `sensor_id` - ID of the sensor to analyze
    ///
    /// # Returns
    /// 
    /// A Result containing the statistical summary, or an error if the query fails.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device or sensor doesn't exist
    /// - There are no measurements for the specified combination
    /// - The database query fails
    pub async fn read_stats_by_device_id_and_sensor_id(
        pool: &PgPool,
        device_id: i32,
        sensor_id: i32,
    ) -> Result<MeasurementStats> {
        let res = sqlx::query_as::<_, MeasurementStats>(
            "SELECT min(value) as min, max(value) as max, count(value) as count, avg(value) as avg, stddev(value) as stddev, variance(value) as variance FROM measurements WHERE device_id = ($1) AND sensor_id = ($2)",
        )
        .bind(device_id)
        .bind(sensor_id)
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    /// Retrieves all measurements for a specific device-sensor combination.
    ///
    /// This method fetches the complete time series of measurements for the
    /// specified device-sensor pair, ordered by timestamp. Useful for creating
    /// detailed charts and historical analysis.
    ///
    /// # Arguments
    /// 
    /// * `device_id` - ID of the device
    /// * `sensor_id` - ID of the sensor
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of all measurements for the device-sensor pair,
    /// ordered chronologically.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device or sensor doesn't exist
    /// - The database query fails
    /// - There are connection issues
    pub async fn read_by_device_id_and_sensor_id(
        device_id: i32,
        sensor_id: i32,
        pool: &PgPool,
    ) -> Result<Vec<Self>> {
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts",
        )
        .bind(device_id)
        .bind(sensor_id)
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    /// Retrieves the most recent measurement for a specific device-sensor combination.
    ///
    /// This method gets the latest single measurement from the specified device-sensor
    /// pair, which is useful for real-time monitoring and status displays.
    ///
    /// # Arguments
    /// 
    /// * `device_id` - ID of the device
    /// * `sensor_id` - ID of the sensor
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing the most recent measurement for the device-sensor pair.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - No measurements exist for the specified device-sensor combination
    /// - The device or sensor doesn't exist
    /// - The database query fails
    pub async fn read_latest_by_device_id_and_sensor_id(
        device_id: i32,
        sensor_id: i32,
        pool: &PgPool,
    ) -> Result<Self> {
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts desc LIMIT 1",
        )
        .bind(device_id)
        .bind(sensor_id)
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    /// Retrieves all measurements from a specific device across all its sensors.
    ///
    /// This method fetches all measurements reported by the specified device,
    /// regardless of which sensor produced them. Results are ordered chronologically.
    /// Useful for device-level monitoring and analysis.
    ///
    /// # Arguments
    /// 
    /// * `device_id` - ID of the device to get measurements from
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of all measurements from the device,
    /// ordered chronologically.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The device doesn't exist
    /// - The database query fails
    /// - There are connection issues
    pub async fn read_by_device_id(device_id: i32, pool: &PgPool) -> Result<Vec<Self>> {
        let res = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) ORDER BY ts",
        )
        .bind(device_id)
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    /// Retrieves all measurements in the system.
    ///
    /// This method fetches every measurement record from all devices and sensors,
    /// ordered chronologically. This can be a large dataset and should be used
    /// with caution in production environments. Consider using pagination or
    /// filtering for large datasets.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing a vector of all measurements in the system.
    ///
    /// # Errors
    /// 
    /// Returns an error if the database query fails or connection issues occur.
    ///
    /// # Performance Note
    /// 
    /// This operation can be expensive for large datasets. Consider using
    /// more specific query methods when possible.
    pub async fn read_all(pool: &PgPool) -> Result<Vec<Measurement>> {
        let measurements =
            sqlx::query_as::<_, Measurement>("SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts")
                .fetch_all(pool)
                .await?;
        Ok(measurements)
    }

    /// Retrieves the most recent measurement from any device and sensor.
    ///
    /// This method gets the latest single measurement across the entire system,
    /// which can be useful for system health monitoring or "last activity" displays.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing the most recent measurement in the system.
    ///
    /// # Errors
    /// 
    /// Returns an error if:
    /// - No measurements exist in the system
    /// - The database query fails
    /// - There are connection issues
    pub async fn read_latest(pool: &PgPool) -> Result<Self> {
        let measurement = sqlx::query_as::<_, Measurement>(
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts DESC LIMIT 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(measurement)
    }

    /// Gets the total count of measurements in the system.
    ///
    /// This method returns the total number of measurement records across
    /// all devices and sensors. Useful for system statistics and monitoring.
    ///
    /// # Arguments
    /// 
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    /// 
    /// A Result containing the total number of measurements.
    ///
    /// # Errors
    /// 
    /// Returns an error if the database query fails or connection issues occur.
    pub async fn read_total_measurements(pool: &PgPool) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM measurements")
            .fetch_one(pool)
            .await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::measurements::NewMeasurement;
    use crate::sensors::NewSensor;
    use crate::{devices::NewDevice, measurements::Measurement};

    #[sqlx::test]
    async fn should_insert_measurements_without_ts(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();
    }

    #[sqlx::test]
    async fn should_insert_measurements_with_ts(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let ts = chrono::Utc::now();

        let measurement = NewMeasurement::new(Some(ts), 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();
    }

    #[sqlx::test]
    async fn should_read_measurements(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_all(&pool).await.unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_latest_measurements(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();

        let measurement = Measurement::read_latest(&pool).await.unwrap();
        assert_eq!(measurement.value, 1.0);
    }

    #[sqlx::test]
    async fn should_read_measurements_by_device_id(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_by_device_id(1, &pool).await.unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_measurements_by_device_id_and_sensor_id(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_by_device_id_and_sensor_id(1, 1, &pool)
            .await
            .unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_latest_measurements_by_device_id_and_sensor_id(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&pool).await.unwrap();

        let measurement = NewMeasurement::new(None, 1, 1, 1.0);
        measurement.insert(&pool).await.unwrap();

        let measurement = Measurement::read_latest_by_device_id_and_sensor_id(1, 1, &pool)
            .await
            .unwrap();
        assert_eq!(measurement.value, 1.0);
    }
}
