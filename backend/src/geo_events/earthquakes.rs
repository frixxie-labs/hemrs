use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use super::EventQuery;
use crate::{
    coordinates::Coordinates,
    devices::DeviceId,
    measurements::{InsertOutcome, finite_f32},
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewEarthquake {
    /// When the earthquake happened
    pub timestamp: DateTime<Utc>,
    /// The device that reported the earthquake
    pub device: DeviceId,
    /// Epicentre
    pub coordinates: Coordinates,
    /// Hypocentre depth in kilometres
    #[serde(deserialize_with = "finite_f32")]
    pub depth_km: f32,
    #[serde(deserialize_with = "finite_f32")]
    pub magnitude: f32,
    /// Significance score from the reporting source
    pub significance: i32,
    /// Whether a tsunami was flagged
    pub tsunami: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, ToSchema)]
pub struct Earthquake {
    pub timestamp: DateTime<Utc>,
    pub device_id: DeviceId,
    pub coordinates: Coordinates,
    pub depth_km: f32,
    pub magnitude: f32,
    pub significance: i32,
    pub tsunami: bool,
}

struct EarthquakeRow {
    timestamp: DateTime<Utc>,
    device_id: DeviceId,
    latitude: Option<f64>,
    longitude: Option<f64>,
    depth_km: f32,
    magnitude: f32,
    significance: i32,
    tsunami: bool,
}

impl TryFrom<EarthquakeRow> for Earthquake {
    type Error = anyhow::Error;

    fn try_from(row: EarthquakeRow) -> Result<Self> {
        let coordinates = Coordinates::from_columns(row.latitude, row.longitude)?
            .ok_or_else(|| anyhow::anyhow!("earthquake without a position"))?;
        Ok(Self {
            timestamp: row.timestamp,
            device_id: row.device_id,
            coordinates,
            depth_km: row.depth_km,
            magnitude: row.magnitude,
            significance: row.significance,
            tsunami: row.tsunami,
        })
    }
}

impl NewEarthquake {
    /// A device reports a given earthquake once; resends are ignored.
    pub async fn insert(self, pool: &PgPool) -> Result<InsertOutcome> {
        let result = sqlx::query!(
            "INSERT INTO earthquakes (ts, device_id, geom, depth_km, magnitude, significance, tsunami) \
             VALUES ($1, $2, ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography, $5, $6, $7, $8) \
             ON CONFLICT DO NOTHING",
            self.timestamp,
            self.device.0,
            self.coordinates.longitude(),
            self.coordinates.latitude(),
            self.depth_km,
            self.magnitude,
            self.significance,
            self.tsunami
        )
        .execute(pool)
        .await?;
        Ok(match result.rows_affected() {
            0 => InsertOutcome::Duplicate,
            _ => InsertOutcome::Inserted,
        })
    }
}

/// The `limit` most recent earthquakes matching `query`, in chronological order.
pub async fn read(pool: &PgPool, query: EventQuery) -> Result<Vec<Earthquake>> {
    sqlx::query_as!(
        EarthquakeRow,
        "SELECT timestamp AS \"timestamp!\", device_id AS \"device_id!: DeviceId\", latitude, longitude, \
                depth_km AS \"depth_km!\", magnitude AS \"magnitude!\", significance AS \"significance!\", tsunami AS \"tsunami!\" \
         FROM (SELECT ts AS timestamp, device_id, ST_Y(geom::geometry) AS latitude, ST_X(geom::geometry) AS longitude, \
                      depth_km, magnitude, significance, tsunami \
               FROM earthquakes \
               WHERE ts >= $1 AND ts <= $2 \
                 AND ($5::float8 IS NULL \
                      OR ST_DWithin(geom, ST_SetSRID(ST_MakePoint($4, $3), 4326)::geography, $5)) \
               ORDER BY ts DESC \
               LIMIT $6) recent \
         ORDER BY timestamp",
        query.range.start(),
        query.range.end(),
        query.latitude(),
        query.longitude(),
        query.radius_m(),
        query.limit
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(Earthquake::try_from)
    .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        devices::NewDevice,
        geo_events::{Area, EventQuery},
        measurements::date_range::DateRange,
    };

    fn quake(timestamp: DateTime<Utc>, magnitude: f32) -> NewEarthquake {
        NewEarthquake {
            timestamp,
            device: DeviceId(1),
            coordinates: Coordinates::new(44.0156, 147.4942).unwrap(),
            depth_km: 66.149,
            magnitude,
            significance: 340,
            tsunami: false,
        }
    }

    #[sqlx::test]
    async fn resent_earthquake_is_stored_once(pool: PgPool) {
        NewDevice {
            name: "wictk_earthquake".to_string(),
            location: "Japan".to_string(),
            coordinates: None,
        }
        .insert(&pool)
        .await
        .unwrap();
        let now = Utc::now();

        let outcomes = (
            quake(now, 4.7).insert(&pool).await.unwrap(),
            quake(now, 4.7).insert(&pool).await.unwrap(),
        );

        assert_eq!(
            outcomes,
            (InsertOutcome::Inserted, InsertOutcome::Duplicate)
        );
        let tokyo = Coordinates::new(35.6762, 139.6503).unwrap();
        let query = EventQuery {
            range: DateRange::new(now - Duration::hours(1), now).unwrap(),
            area: Some(Area::new(tokyo, 1_500.0).unwrap()),
            limit: 10,
        };
        let stored = read(&pool, query).await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].magnitude, 4.7);
        assert_eq!(
            stored[0].coordinates,
            Coordinates::new(44.0156, 147.4942).unwrap()
        );
    }

    #[test]
    fn non_finite_magnitude_is_rejected() {
        let json = r#"{"timestamp":"2026-08-10T10:36:42Z","device":1,
            "coordinates":{"latitude":44.0,"longitude":147.5},
            "depth_km":66.1,"magnitude":1e39,"significance":340,"tsunami":false}"#;
        assert!(serde_json::from_str::<NewEarthquake>(json).is_err());
    }
}
