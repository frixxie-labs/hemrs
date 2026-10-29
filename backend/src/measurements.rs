use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, de};
use sqlx::{FromRow, PgPool};
use std::fmt;
use utoipa::ToSchema;

use crate::{devices::DeviceId, sensors::SensorId};

pub mod date_range;
pub mod latest;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct NewMeasurement {
    pub timestamp: Option<DateTime<Utc>>,
    pub device: DeviceId,
    pub sensor: SensorId,
    #[serde(deserialize_with = "finite_f32")]
    pub measurement: f32,
}

/// JSON numbers beyond the f32 range would otherwise be stored as infinity.
fn finite_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let value = f32::deserialize(deserializer)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(de::Error::custom(
            "measurement must be a finite 32-bit float",
        ))
    }
}

impl NewMeasurement {
    pub fn resolve_timestamp(self, now: DateTime<Utc>) -> ResolvedMeasurement {
        ResolvedMeasurement {
            timestamp: self.timestamp.unwrap_or(now),
            device: self.device,
            sensor: self.sensor,
            measurement: self.measurement,
        }
    }

    #[cfg(test)]
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        self.resolve_timestamp(Utc::now()).insert(pool).await
    }
}

/// Internal ingestion command: the database row and published event share one timestamp.
pub struct ResolvedMeasurement {
    pub timestamp: DateTime<Utc>,
    pub device: DeviceId,
    pub sensor: SensorId,
    pub measurement: f32,
}

impl ResolvedMeasurement {
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES ($1, $2, $3, $4)",
            self.timestamp,
            self.device.0,
            self.sensor.0,
            self.measurement
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum NewMeasurements {
    Measurement(NewMeasurement),
    Measurements(Vec<NewMeasurement>),
}

impl fmt::Display for NewMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.device, self.sensor, self.measurement)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Measurement {
    pub timestamp: DateTime<Utc>,
    pub value: f32,
    pub unit: String,
    pub device_name: String,
    pub device_location: String,
    pub sensor_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MeasurementUpdate {
    pub device_id: DeviceId,
    pub sensor_id: SensorId,
    pub measurement: Measurement,
}

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct MeasurementStats {
    min: Option<f32>,
    max: Option<f32>,
    count: i64,
    avg: Option<f64>,
    stddev: Option<f64>,
    variance: Option<f64>,
}

impl Measurement {
    pub async fn read_all_latest_measurements(pool: &PgPool) -> Result<Vec<Measurement>> {
        let res = sqlx::query_as!(
            Measurement,
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

    pub async fn read_stats_by_device_id_and_sensor_id(
        pool: &PgPool,
        device_id: DeviceId,
        sensor_id: SensorId,
    ) -> Result<MeasurementStats> {
        let res = sqlx::query_as!(
            MeasurementStats,
            "SELECT min(value) as \"min?\", max(value) as \"max?\", count(value) as \"count!\", avg(value) as \"avg?\", stddev(value) as \"stddev?\", variance(value) as \"variance?\" FROM measurements WHERE device_id = ($1) AND sensor_id = ($2)",
            device_id.0,
            sensor_id.0
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    /// Reads the `limit` most recent measurements, returned in chronological order.
    pub async fn read_by_device_id_and_sensor_id(
        pool: &PgPool,
        device_id: DeviceId,
        sensor_id: SensorId,
        limit: i64,
    ) -> Result<Vec<Self>> {
        let res = sqlx::query_as!(
            Measurement,
            "SELECT timestamp AS \"timestamp!\", value AS \"value!\", unit AS \"unit!\", device_name AS \"device_name!\", device_location AS \"device_location!\", sensor_name AS \"sensor_name!\" \
             FROM (SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name \
                   FROM measurements m \
                   JOIN devices d ON d.id = m.device_id \
                   JOIN sensors s ON s.id = m.sensor_id \
                   WHERE m.device_id = $1 AND m.sensor_id = $2 \
                   ORDER BY m.ts DESC \
                   LIMIT $3) recent \
             ORDER BY timestamp",
            device_id.0,
            sensor_id.0,
            limit
        )
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_latest_by_device_id_and_sensor_id(
        pool: &PgPool,
        device_id: DeviceId,
        sensor_id: SensorId,
    ) -> Result<Self> {
        let res = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts desc LIMIT 1",
            device_id.0,
            sensor_id.0
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    /// Reads the `limit` most recent measurements, returned in chronological order.
    pub async fn read_by_device_id(
        pool: &PgPool,
        device_id: DeviceId,
        limit: i64,
    ) -> Result<Vec<Self>> {
        let res = sqlx::query_as!(
            Measurement,
            "SELECT timestamp AS \"timestamp!\", value AS \"value!\", unit AS \"unit!\", device_name AS \"device_name!\", device_location AS \"device_location!\", sensor_name AS \"sensor_name!\" \
             FROM (SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name \
                   FROM measurements m \
                   JOIN devices d ON d.id = m.device_id \
                   JOIN sensors s ON s.id = m.sensor_id \
                   WHERE m.device_id = $1 \
                   ORDER BY m.ts DESC \
                   LIMIT $2) recent \
             ORDER BY timestamp",
            device_id.0,
            limit
        )
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    /// Reads the `limit` most recent measurements, returned in chronological order.
    pub async fn read_all(pool: &PgPool, limit: i64) -> Result<Vec<Self>> {
        let measurements = sqlx::query_as!(
            Measurement,
            "SELECT timestamp AS \"timestamp!\", value AS \"value!\", unit AS \"unit!\", device_name AS \"device_name!\", device_location AS \"device_location!\", sensor_name AS \"sensor_name!\" \
             FROM (SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name \
                   FROM measurements m \
                   JOIN devices d ON d.id = m.device_id \
                   JOIN sensors s ON s.id = m.sensor_id \
                   ORDER BY m.ts DESC \
                   LIMIT $1) recent \
             ORDER BY timestamp",
            limit
        )
        .fetch_all(pool)
        .await?;
        Ok(measurements)
    }

    pub async fn read_latest(pool: &PgPool) -> Result<Self> {
        let measurement = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts DESC LIMIT 1",
        )
        .fetch_one(pool)
        .await?;
        Ok(measurement)
    }

    pub async fn read_total_measurements(pool: &PgPool) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM measurements")
            .fetch_one(pool)
            .await?;
        Ok(count)
    }

    pub async fn read_by_date_range(
        pool: &PgPool,
        range: date_range::DateRange,
    ) -> Result<Vec<Self>> {
        let measurements = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name \
             FROM measurements m \
             JOIN devices d ON d.id = m.device_id \
             JOIN sensors s ON s.id = m.sensor_id \
             WHERE m.ts >= $1 AND m.ts <= $2 \
             ORDER BY m.ts",
            range.start(),
            range.end()
        )
        .fetch_all(pool)
        .await?;
        Ok(measurements)
    }
}

#[cfg(test)]
mod tests {
    use super::date_range::DateRange;
    use crate::{devices::DeviceId, sensors::SensorId};
    use sqlx::PgPool;

    use crate::measurements::NewMeasurement;
    use crate::sensors::NewSensor;
    use crate::{devices::NewDevice, measurements::Measurement};

    async fn setup_device_and_sensor(pool: &PgPool) {
        NewDevice {
            name: "test".to_string(),
            location: "test".to_string(),
        }
        .insert(pool)
        .await
        .unwrap();
        NewSensor {
            name: "test".to_string(),
            unit: "test".to_string(),
        }
        .insert(pool)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn statistics_represent_empty_and_single_sample_datasets(pool: PgPool) {
        let empty = serde_json::to_value(
            Measurement::read_stats_by_device_id_and_sensor_id(&pool, DeviceId(1), SensorId(1))
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(empty["count"], 0);
        for field in ["min", "max", "avg", "stddev", "variance"] {
            assert!(
                empty[field].is_null(),
                "{field} is undefined for an empty dataset"
            );
        }
        setup_device_and_sensor(&pool).await;
        NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 12.5,
        }
        .insert(&pool)
        .await
        .unwrap();
        let single = serde_json::to_value(
            Measurement::read_stats_by_device_id_and_sensor_id(&pool, DeviceId(1), SensorId(1))
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(single["count"], 1);
        assert_eq!(single["min"], 12.5);
        assert_eq!(single["max"], 12.5);
        assert_eq!(single["avg"], 12.5);
        assert!(single["stddev"].is_null());
        assert!(single["variance"].is_null());
    }

    #[test]
    fn timestamp_resolution_preserves_explicit_values_and_defaults_once() {
        let now = chrono::Utc::now();
        let explicit = now - chrono::Duration::hours(1);
        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(2),
            measurement: 3.0,
        };
        assert_eq!(measurement.clone().resolve_timestamp(now).timestamp, now);
        assert_eq!(
            NewMeasurement {
                timestamp: Some(explicit),
                ..measurement
            }
            .resolve_timestamp(now)
            .timestamp,
            explicit
        );
    }

    // ── quickcheck: pure logic, no DB ────────────────────────────────────────

    #[quickcheck_macros::quickcheck]
    fn display_contains_device_sensor_and_value(device: i32, sensor: i32, value: f32) -> bool {
        let m = NewMeasurement {
            timestamp: None,
            device: DeviceId(device),
            sensor: SensorId(sensor),
            measurement: value,
        };
        let s = format!("{m}");
        // Format is "{device},{sensor},{measurement}"
        let parts: Vec<&str> = s.splitn(3, ',').collect();
        parts.len() == 3
            && parts[0].parse::<i32>().ok() == Some(device)
            && parts[1].parse::<i32>().ok() == Some(sensor)
    }

    #[quickcheck_macros::quickcheck]
    fn single_measurement_round_trips_through_json(device: i32, sensor: i32, value: f32) -> bool {
        use crate::measurements::NewMeasurements;

        // NaN and infinity are not valid JSON numbers; skip them.
        if !value.is_finite() {
            return true;
        }

        let original = NewMeasurement {
            timestamp: None,
            device: DeviceId(device),
            sensor: SensorId(sensor),
            measurement: value,
        };
        let json = serde_json::to_string(&original).expect("serialization failed");
        let parsed: Result<NewMeasurements, _> = serde_json::from_str(&json);
        match parsed {
            Ok(NewMeasurements::Measurement(m)) => {
                m.device == DeviceId(device) && m.sensor == SensorId(sensor)
            }
            _ => false,
        }
    }

    #[test]
    fn measurement_values_outside_f32_range_are_rejected() {
        for value in ["1e39", "-1e39"] {
            let json = format!(r#"{{"device":1,"sensor":1,"measurement":{value}}}"#);
            assert!(
                serde_json::from_str::<NewMeasurement>(&json).is_err(),
                "{value} would be stored as infinity"
            );
        }
    }

    #[quickcheck_macros::quickcheck]
    fn vec_of_measurements_round_trips_through_json(items: Vec<(i32, i32)>) -> bool {
        use crate::measurements::NewMeasurements;

        let measurements: Vec<NewMeasurement> = items
            .iter()
            .map(|&(device, sensor)| NewMeasurement {
                timestamp: None,
                device: DeviceId(device),
                sensor: SensorId(sensor),
                measurement: 0.0,
            })
            .collect();

        let json = serde_json::to_string(&measurements).expect("serialization failed");
        let parsed: Result<NewMeasurements, _> = serde_json::from_str(&json);
        match parsed {
            Ok(NewMeasurements::Measurements(ms)) => ms.len() == measurements.len(),
            // An empty Vec deserialises as Measurements([]), but a single-element Vec
            // may also match the untagged Measurement variant — both are acceptable.
            Ok(NewMeasurements::Measurement(_)) => measurements.len() == 1,
            Err(_) => false,
        }
    }

    // ── sqlx integration tests ────────────────────────────────────────────────

    #[sqlx::test]
    async fn should_insert_measurements_without_ts(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();
    }

    #[sqlx::test]
    async fn should_insert_measurements_with_ts(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let ts = chrono::Utc::now();

        let measurement = NewMeasurement {
            timestamp: Some(ts),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();
    }

    #[sqlx::test]
    async fn should_read_measurements(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_all(&pool, 100).await.unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_only_most_recent_measurements_in_order(pool: PgPool) {
        setup_device_and_sensor(&pool).await;
        let base = chrono::Utc::now();
        for i in 0..5 {
            NewMeasurement {
                timestamp: Some(base + chrono::Duration::seconds(i)),
                device: DeviceId(1),
                sensor: SensorId(1),
                measurement: i as f32,
            }
            .insert(&pool)
            .await
            .unwrap();
        }

        let values = |measurements: Vec<Measurement>| -> Vec<f32> {
            measurements.iter().map(|m| m.value).collect()
        };
        let expected = vec![2.0, 3.0, 4.0];
        assert_eq!(
            values(Measurement::read_all(&pool, 3).await.unwrap()),
            expected
        );
        assert_eq!(
            values(
                Measurement::read_by_device_id(&pool, DeviceId(1), 3)
                    .await
                    .unwrap()
            ),
            expected
        );
        assert_eq!(
            values(
                Measurement::read_by_device_id_and_sensor_id(&pool, DeviceId(1), SensorId(1), 3)
                    .await
                    .unwrap()
            ),
            expected
        );
    }

    #[sqlx::test]
    async fn should_read_latest_measurements(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurement = Measurement::read_latest(&pool).await.unwrap();
        assert_eq!(measurement.value, 1.0);
    }

    #[sqlx::test]
    async fn should_read_measurements_by_device_id(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_by_device_id(&pool, DeviceId(1), 10)
            .await
            .unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_measurements_by_device_id_and_sensor_id(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurements =
            Measurement::read_by_device_id_and_sensor_id(&pool, DeviceId(1), SensorId(1), 10)
                .await
                .unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_latest_measurements_by_device_id_and_sensor_id(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurement =
            Measurement::read_latest_by_device_id_and_sensor_id(&pool, DeviceId(1), SensorId(1))
                .await
                .unwrap();
        assert_eq!(measurement.value, 1.0);
    }

    #[sqlx::test]
    async fn should_read_measurements_within_date_range(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let ts = chrono::Utc::now();
        let measurement = NewMeasurement {
            timestamp: Some(ts),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 42.0,
        };
        measurement.insert(&pool).await.unwrap();

        let start = ts - chrono::Duration::seconds(10);
        let end = ts + chrono::Duration::seconds(10);
        let results = Measurement::read_by_date_range(&pool, DateRange::new(start, end).unwrap())
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, 42.0);
    }

    #[sqlx::test]
    async fn should_return_empty_when_no_measurements_in_range(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let ts = chrono::Utc::now();
        let measurement = NewMeasurement {
            timestamp: Some(ts),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 7.0,
        };
        measurement.insert(&pool).await.unwrap();

        // Range is entirely in the future — nothing should match
        let start = ts + chrono::Duration::hours(1);
        let end = ts + chrono::Duration::hours(2);
        let results = Measurement::read_by_date_range(&pool, DateRange::new(start, end).unwrap())
            .await
            .unwrap();

        assert!(results.is_empty());
    }

    #[sqlx::test]
    async fn should_exclude_measurements_outside_range(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let now = chrono::Utc::now();
        // Inside the window
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 10.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        // Before the window
        NewMeasurement {
            timestamp: Some(now - chrono::Duration::hours(2)),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 99.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        // After the window
        NewMeasurement {
            timestamp: Some(now + chrono::Duration::hours(2)),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 99.0,
        }
        .insert(&pool)
        .await
        .unwrap();

        let start = now - chrono::Duration::minutes(5);
        let end = now + chrono::Duration::minutes(5);
        let results = Measurement::read_by_date_range(&pool, DateRange::new(start, end).unwrap())
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, 10.0);
    }

    #[sqlx::test]
    async fn should_read_measurements_up_to_current_time(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let ts = chrono::Utc::now();
        NewMeasurement {
            timestamp: Some(ts),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 5.0,
        }
        .insert(&pool)
        .await
        .unwrap();

        // The HTTP boundary resolves an omitted end to the current time.
        let start = ts - chrono::Duration::seconds(10);
        let results = Measurement::read_by_date_range(
            &pool,
            DateRange::new(start, chrono::Utc::now()).unwrap(),
        )
        .await
        .unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].value, 5.0);
    }

    #[sqlx::test]
    async fn should_return_measurements_ordered_by_timestamp(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let now = chrono::Utc::now();
        // Insert in reverse order
        NewMeasurement {
            timestamp: Some(now + chrono::Duration::seconds(2)),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 3.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        NewMeasurement {
            timestamp: Some(now + chrono::Duration::seconds(1)),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 2.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        NewMeasurement {
            timestamp: Some(now),
            device: DeviceId(1),
            sensor: SensorId(1),
            measurement: 1.0,
        }
        .insert(&pool)
        .await
        .unwrap();

        let start = now - chrono::Duration::seconds(1);
        let end = now + chrono::Duration::seconds(10);
        let results = Measurement::read_by_date_range(&pool, DateRange::new(start, end).unwrap())
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].value, 1.0);
        assert_eq!(results[1].value, 2.0);
        assert_eq!(results[2].value, 3.0);
    }
}
