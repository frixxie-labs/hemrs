use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct NewMeasurement {
    pub timestamp: Option<DateTime<Utc>>,
    pub device: i32,
    pub sensor: i32,
    pub measurement: f32,
}

impl NewMeasurement {
    pub async fn insert(self, pool: &PgPool) -> Result<()> {
        match self.timestamp {
            Some(t) => {
                sqlx::query!(
                    "INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES ($1, $2, $3, $4)",
                    t,
                    self.device,
                    self.sensor,
                    self.measurement
                )
                .execute(pool)
                .await?;
                Ok(())
            }
            None => {
                sqlx::query!(
                    "INSERT INTO measurements (ts, device_id, sensor_id, value) VALUES (CURRENT_TIMESTAMP, $1, $2, $3)",
                    self.device,
                    self.sensor,
                    self.measurement
                )
                .execute(pool)
                .await?;
                Ok(())
            }
        }
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
    pub device_id: i32,
    pub sensor_id: i32,
    pub measurement: Measurement,
}

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct MeasurementStats {
    min: f32,
    max: f32,
    count: i64,
    avg: f64,
    stddev: f64,
    variance: f64,
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
        device_id: i32,
        sensor_id: i32,
    ) -> Result<MeasurementStats> {
        let res = sqlx::query_as!(
            MeasurementStats,
            "SELECT min(value) as \"min!\", max(value) as \"max!\", count(value) as \"count!\", avg(value) as \"avg!\", stddev(value) as \"stddev!\", variance(value) as \"variance!\" FROM measurements WHERE device_id = ($1) AND sensor_id = ($2)",
            device_id,
            sensor_id
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_by_device_id_and_sensor_id(
        device_id: i32,
        sensor_id: i32,
        pool: &PgPool,
    ) -> Result<Vec<Self>> {
        let res = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts",
            device_id,
            sensor_id
        )
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_latest_by_device_id_and_sensor_id(
        device_id: i32,
        sensor_id: i32,
        pool: &PgPool,
    ) -> Result<Self> {
        let res = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) AND m.sensor_id = ($2) ORDER BY ts desc LIMIT 1",
            device_id,
            sensor_id
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_by_device_id(device_id: i32, pool: &PgPool) -> Result<Vec<Self>> {
        let res = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id where m.device_id = ($1) ORDER BY ts",
            device_id
        )
        .fetch_all(pool)
        .await?;
        Ok(res)
    }

    pub async fn read_all(pool: &PgPool) -> Result<Vec<Self>> {
        let measurements = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name FROM measurements m JOIN devices d ON d.id = m.device_id JOIN sensors s ON s.id = m.sensor_id ORDER BY ts"
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
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<Self>> {
        let end = end.unwrap_or_else(Utc::now);
        let measurements = sqlx::query_as!(
            Measurement,
            "SELECT m.ts AS timestamp, m.value, s.unit, d.name AS device_name, d.location AS device_location, s.name AS sensor_name \
             FROM measurements m \
             JOIN devices d ON d.id = m.device_id \
             JOIN sensors s ON s.id = m.sensor_id \
             WHERE m.ts >= $1 AND m.ts <= $2 \
             ORDER BY m.ts",
            start,
            end
        )
        .fetch_all(pool)
        .await?;
        Ok(measurements)
    }
}

#[cfg(test)]
mod tests {
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

    // ── quickcheck: pure logic, no DB ────────────────────────────────────────

    #[quickcheck_macros::quickcheck]
    fn display_contains_device_sensor_and_value(device: i32, sensor: i32, value: f32) -> bool {
        let m = NewMeasurement {
            timestamp: None,
            device,
            sensor,
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
            device,
            sensor,
            measurement: value,
        };
        let json = serde_json::to_string(&original).expect("serialization failed");
        let parsed: Result<NewMeasurements, _> = serde_json::from_str(&json);
        match parsed {
            Ok(NewMeasurements::Measurement(m)) => m.device == device && m.sensor == sensor,
            _ => false,
        }
    }

    #[quickcheck_macros::quickcheck]
    fn vec_of_measurements_round_trips_through_json(items: Vec<(i32, i32)>) -> bool {
        use crate::measurements::NewMeasurements;

        let measurements: Vec<NewMeasurement> = items
            .iter()
            .map(|&(device, sensor)| NewMeasurement {
                timestamp: None,
                device,
                sensor,
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
            device: 1,
            sensor: 1,
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
            device: 1,
            sensor: 1,
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();
    }

    #[sqlx::test]
    async fn should_read_measurements(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: 1,
            sensor: 1,
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_all(&pool).await.unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_latest_measurements(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: 1,
            sensor: 1,
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
            device: 1,
            sensor: 1,
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_by_device_id(1, &pool).await.unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_measurements_by_device_id_and_sensor_id(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: 1,
            sensor: 1,
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurements = Measurement::read_by_device_id_and_sensor_id(1, 1, &pool)
            .await
            .unwrap();
        assert!(!measurements.is_empty());
    }

    #[sqlx::test]
    async fn should_read_latest_measurements_by_device_id_and_sensor_id(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let measurement = NewMeasurement {
            timestamp: None,
            device: 1,
            sensor: 1,
            measurement: 1.0,
        };
        measurement.insert(&pool).await.unwrap();

        let measurement = Measurement::read_latest_by_device_id_and_sensor_id(1, 1, &pool)
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
            device: 1,
            sensor: 1,
            measurement: 42.0,
        };
        measurement.insert(&pool).await.unwrap();

        let start = ts - chrono::Duration::seconds(10);
        let end = ts + chrono::Duration::seconds(10);
        let results = Measurement::read_by_date_range(&pool, start, Some(end))
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
            device: 1,
            sensor: 1,
            measurement: 7.0,
        };
        measurement.insert(&pool).await.unwrap();

        // Range is entirely in the future — nothing should match
        let start = ts + chrono::Duration::hours(1);
        let end = ts + chrono::Duration::hours(2);
        let results = Measurement::read_by_date_range(&pool, start, Some(end))
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
            device: 1,
            sensor: 1,
            measurement: 10.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        // Before the window
        NewMeasurement {
            timestamp: Some(now - chrono::Duration::hours(2)),
            device: 1,
            sensor: 1,
            measurement: 99.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        // After the window
        NewMeasurement {
            timestamp: Some(now + chrono::Duration::hours(2)),
            device: 1,
            sensor: 1,
            measurement: 99.0,
        }
        .insert(&pool)
        .await
        .unwrap();

        let start = now - chrono::Duration::minutes(5);
        let end = now + chrono::Duration::minutes(5);
        let results = Measurement::read_by_date_range(&pool, start, Some(end))
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, 10.0);
    }

    #[sqlx::test]
    async fn should_read_measurements_without_explicit_end(pool: PgPool) {
        setup_device_and_sensor(&pool).await;

        let ts = chrono::Utc::now();
        NewMeasurement {
            timestamp: Some(ts),
            device: 1,
            sensor: 1,
            measurement: 5.0,
        }
        .insert(&pool)
        .await
        .unwrap();

        // No end — should default to now and return the measurement
        let start = ts - chrono::Duration::seconds(10);
        let results = Measurement::read_by_date_range(&pool, start, None)
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
            device: 1,
            sensor: 1,
            measurement: 3.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        NewMeasurement {
            timestamp: Some(now + chrono::Duration::seconds(1)),
            device: 1,
            sensor: 1,
            measurement: 2.0,
        }
        .insert(&pool)
        .await
        .unwrap();
        NewMeasurement {
            timestamp: Some(now),
            device: 1,
            sensor: 1,
            measurement: 1.0,
        }
        .insert(&pool)
        .await
        .unwrap();

        let start = now - chrono::Duration::seconds(1);
        let end = now + chrono::Duration::seconds(10);
        let results = Measurement::read_by_date_range(&pool, start, Some(end))
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].value, 1.0);
        assert_eq!(results[1].value, 2.0);
        assert_eq!(results[2].value, 3.0);
    }
}
