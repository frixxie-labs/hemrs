use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use super::EventQuery;
use crate::{coordinates::Coordinates, devices::DeviceId};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewLightningStrike {
    /// When the strike happened; defaults to the time of ingestion
    pub timestamp: Option<DateTime<Utc>>,
    /// The device that reported the strike
    pub device: DeviceId,
    /// Where the strike hit
    pub coordinates: Coordinates,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum NewLightningStrikes {
    Strike(NewLightningStrike),
    Strikes(Vec<NewLightningStrike>),
}

impl NewLightningStrikes {
    pub fn into_vec(self) -> Vec<NewLightningStrike> {
        match self {
            Self::Strike(strike) => vec![strike],
            Self::Strikes(strikes) => strikes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, ToSchema)]
pub struct LightningStrike {
    pub timestamp: DateTime<Utc>,
    pub device_id: DeviceId,
    pub coordinates: Coordinates,
}

struct LightningRow {
    timestamp: DateTime<Utc>,
    device_id: DeviceId,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

impl TryFrom<LightningRow> for LightningStrike {
    type Error = anyhow::Error;

    fn try_from(row: LightningRow) -> Result<Self> {
        let coordinates = Coordinates::from_columns(row.latitude, row.longitude)?
            .ok_or_else(|| anyhow::anyhow!("lightning strike without a position"))?;
        Ok(Self {
            timestamp: row.timestamp,
            device_id: row.device_id,
            coordinates,
        })
    }
}

/// Stores every strike, including several sharing a timestamp; returns how many were stored.
pub async fn insert_all(
    pool: &PgPool,
    strikes: &[NewLightningStrike],
    now: DateTime<Utc>,
) -> Result<u64> {
    let timestamps: Vec<DateTime<Utc>> =
        strikes.iter().map(|s| s.timestamp.unwrap_or(now)).collect();
    let devices: Vec<i32> = strikes.iter().map(|s| s.device.0).collect();
    let latitudes: Vec<f64> = strikes.iter().map(|s| s.coordinates.latitude()).collect();
    let longitudes: Vec<f64> = strikes.iter().map(|s| s.coordinates.longitude()).collect();
    let result = sqlx::query!(
        "INSERT INTO lightning_strikes (ts, device_id, geom) \
         SELECT ts, device_id, ST_SetSRID(ST_MakePoint(lon, lat), 4326)::geography \
         FROM UNNEST($1::timestamptz[], $2::int4[], $3::float8[], $4::float8[]) AS s(ts, device_id, lat, lon)",
        &timestamps,
        &devices,
        &latitudes,
        &longitudes
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// The `limit` most recent strikes matching `query`, in chronological order.
pub async fn read(pool: &PgPool, query: EventQuery) -> Result<Vec<LightningStrike>> {
    sqlx::query_as!(
        LightningRow,
        "SELECT timestamp AS \"timestamp!\", device_id AS \"device_id!: DeviceId\", latitude, longitude \
         FROM (SELECT ts AS timestamp, device_id, ST_Y(geom::geometry) AS latitude, ST_X(geom::geometry) AS longitude \
               FROM lightning_strikes \
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
    .map(LightningStrike::try_from)
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

    async fn setup_device(pool: &PgPool) {
        NewDevice {
            name: "wictk_lightning".to_string(),
            location: "Mobile".to_string(),
            coordinates: None,
        }
        .insert(pool)
        .await
        .unwrap();
    }

    fn strike(timestamp: DateTime<Utc>, latitude: f64, longitude: f64) -> NewLightningStrike {
        NewLightningStrike {
            timestamp: Some(timestamp),
            device: DeviceId(1),
            coordinates: Coordinates::new(latitude, longitude).unwrap(),
        }
    }

    fn around(now: DateTime<Utc>, area: Option<Area>, limit: i64) -> EventQuery {
        EventQuery {
            range: DateRange::new(now - Duration::hours(1), now + Duration::hours(1)).unwrap(),
            area,
            limit,
        }
    }

    #[sqlx::test]
    async fn strikes_sharing_a_timestamp_are_all_stored(pool: PgPool) {
        setup_device(&pool).await;
        let now = Utc::now();
        let strikes = [
            strike(now, 63.2893, 25.1497),
            strike(now, 63.294, 25.1426),
            strike(now, 63.296, 25.1497),
        ];

        let stored = insert_all(&pool, &strikes, now).await.unwrap();

        assert_eq!(stored, 3);
        let positions: Vec<_> = read(&pool, around(now, None, 10))
            .await
            .unwrap()
            .into_iter()
            .map(|s| (s.coordinates.latitude(), s.coordinates.longitude()))
            .collect();
        assert_eq!(positions.len(), 3);
        assert!(positions.contains(&(63.294, 25.1426)));
    }

    #[sqlx::test]
    async fn area_filter_keeps_only_strikes_within_radius(pool: PgPool) {
        setup_device(&pool).await;
        let now = Utc::now();
        let trondheim = Coordinates::new(63.4305, 10.3951).unwrap();
        insert_all(
            &pool,
            &[strike(now, 63.45, 10.45), strike(now, 59.9139, 10.7522)],
            now,
        )
        .await
        .unwrap();

        let near = read(
            &pool,
            around(now, Some(Area::new(trondheim, 50.0).unwrap()), 10),
        )
        .await
        .unwrap();

        assert_eq!(near.len(), 1);
        assert_eq!(near[0].coordinates.latitude(), 63.45);
    }

    #[sqlx::test]
    async fn read_returns_most_recent_strikes_in_chronological_order(pool: PgPool) {
        setup_device(&pool).await;
        let now = Utc::now();
        let strikes: Vec<_> = (0..5)
            .map(|i| strike(now - Duration::seconds(i), 60.0 + i as f64, 10.0))
            .collect();
        insert_all(&pool, &strikes, now).await.unwrap();

        let latitudes: Vec<f64> = read(&pool, around(now, None, 3))
            .await
            .unwrap()
            .into_iter()
            .map(|s| s.coordinates.latitude())
            .collect();

        assert_eq!(latitudes, vec![62.0, 61.0, 60.0]);
    }

    #[test]
    fn single_strike_and_batch_payloads_are_both_accepted() {
        let one = r#"{"device":1,"coordinates":{"latitude":63.2,"longitude":25.1}}"#;
        let many = format!("[{one},{one}]");
        assert_eq!(
            serde_json::from_str::<NewLightningStrikes>(one)
                .unwrap()
                .into_vec()
                .len(),
            1
        );
        assert_eq!(
            serde_json::from_str::<NewLightningStrikes>(&many)
                .unwrap()
                .into_vec()
                .len(),
            2
        );
    }
}
