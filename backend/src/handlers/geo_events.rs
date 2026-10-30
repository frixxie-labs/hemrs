use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::instrument;
use utoipa::IntoParams;

use super::error::HandlerError;
use crate::{
    coordinates::Coordinates,
    geo_events::{
        Area, EventQuery,
        earthquakes::{self, Earthquake, NewEarthquake},
        lightning::{self, LightningStrike, NewLightningStrikes},
    },
    measurements::{InsertOutcome, date_range::DateRange},
};

/// Default number of events returned by event list endpoints.
const DEFAULT_EVENTS_LIMIT: i64 = 1_000;
/// Hard cap on event list endpoints to keep response memory bounded.
const MAX_EVENTS_LIMIT: i64 = 10_000;
/// Hard cap on strikes per ingestion request.
const MAX_STRIKES_PER_REQUEST: usize = 10_000;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GeoEventParams {
    /// Start of the time range (required), RFC 3339
    pub start: DateTime<Utc>,
    /// End of the time range (optional, defaults to now), RFC 3339
    pub end: Option<DateTime<Utc>>,
    /// Latitude of the search centre; requires `longitude` and `radius_km`
    pub latitude: Option<f64>,
    /// Longitude of the search centre; requires `latitude` and `radius_km`
    pub longitude: Option<f64>,
    /// Search radius in kilometres; requires `latitude` and `longitude`
    pub radius_km: Option<f64>,
    /// Number of most recent events to return (default 1000, clamped to 1..=10000)
    pub limit: Option<i64>,
}

impl GeoEventParams {
    fn into_query(self, now: DateTime<Utc>) -> Result<EventQuery, &'static str> {
        let range = DateRange::new(self.start, self.end.unwrap_or(now))?;
        let area = match (self.latitude, self.longitude, self.radius_km) {
            (None, None, None) => None,
            (Some(latitude), Some(longitude), Some(radius_km)) => {
                let center = Coordinates::new(latitude, longitude)
                    .map_err(|_| "latitude or longitude is out of range")?;
                Some(Area::new(center, radius_km)?)
            }
            _ => return Err("latitude, longitude and radius_km must be given together"),
        };
        let limit = self
            .limit
            .unwrap_or(DEFAULT_EVENTS_LIMIT)
            .clamp(1, MAX_EVENTS_LIMIT);
        Ok(EventQuery { range, area, limit })
    }
}

/// Inserts reference `devices`, so a foreign key violation means the client named an unknown device.
fn unknown_device_or_internal(error: anyhow::Error) -> HandlerError {
    let unknown_device = error
        .downcast_ref::<sqlx::Error>()
        .and_then(sqlx::Error::as_database_error)
        .is_some_and(|db| db.is_foreign_key_violation());
    if unknown_device {
        HandlerError::InvalidInput("Unknown device")
    } else {
        HandlerError::from(error)
    }
}

#[utoipa::path(
    post,
    path = "api/lightning",
    request_body = NewLightningStrikes,
    responses(
        (status = 201, description = "Strikes stored"),
        (status = 400, description = "Invalid payload or unknown device"),
        (status = 413, description = "Too many strikes in one request"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument(skip(pool, strikes))]
pub async fn store_lightning_strikes(
    State(pool): State<PgPool>,
    Json(strikes): Json<NewLightningStrikes>,
) -> Result<StatusCode, HandlerError> {
    let strikes = strikes.into_vec();
    if strikes.len() > MAX_STRIKES_PER_REQUEST {
        return Ok(StatusCode::PAYLOAD_TOO_LARGE);
    }
    lightning::insert_all(&pool, &strikes, Utc::now())
        .await
        .map_err(unknown_device_or_internal)?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "api/lightning",
    params(GeoEventParams),
    responses(
        (status = 200, description = "Most recent strikes in chronological order, bounded by `limit`", body = [LightningStrike]),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument(skip(pool))]
pub async fn fetch_lightning_strikes(
    State(pool): State<PgPool>,
    Query(params): Query<GeoEventParams>,
) -> Result<Json<Vec<LightningStrike>>, HandlerError> {
    let query = params
        .into_query(Utc::now())
        .map_err(HandlerError::InvalidInput)?;
    let strikes = lightning::read(&pool, query)
        .await
        .context("Failed to fetch lightning strikes")?;
    Ok(Json(strikes))
}

#[utoipa::path(
    post,
    path = "api/earthquakes",
    request_body = NewEarthquake,
    responses(
        (status = 201, description = "Earthquake stored"),
        (status = 200, description = "Earthquake already stored for this device and timestamp; ignored"),
        (status = 400, description = "Invalid payload or unknown device"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument(skip(pool))]
pub async fn store_earthquake(
    State(pool): State<PgPool>,
    Json(earthquake): Json<NewEarthquake>,
) -> Result<StatusCode, HandlerError> {
    let outcome = earthquake
        .insert(&pool)
        .await
        .map_err(unknown_device_or_internal)?;
    Ok(match outcome {
        InsertOutcome::Inserted => StatusCode::CREATED,
        InsertOutcome::Duplicate => StatusCode::OK,
    })
}

#[utoipa::path(
    get,
    path = "api/earthquakes",
    params(GeoEventParams),
    responses(
        (status = 200, description = "Most recent earthquakes in chronological order, bounded by `limit`", body = [Earthquake]),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument(skip(pool))]
pub async fn fetch_earthquakes(
    State(pool): State<PgPool>,
    Query(params): Query<GeoEventParams>,
) -> Result<Json<Vec<Earthquake>>, HandlerError> {
    let query = params
        .into_query(Utc::now())
        .map_err(HandlerError::InvalidInput)?;
    let earthquakes = earthquakes::read(&pool, query)
        .await
        .context("Failed to fetch earthquakes")?;
    Ok(Json(earthquakes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unknown_device_strike() -> NewLightningStrikes {
        serde_json::from_str(r#"{"device":999,"coordinates":{"latitude":60,"longitude":10}}"#)
            .unwrap()
    }

    fn unknown_device_earthquake() -> NewEarthquake {
        serde_json::from_str(
            r#"{"timestamp":"2030-01-01T00:00:00Z","device":999,
                "coordinates":{"latitude":35,"longitude":139},
                "depth_km":10,"magnitude":5,"significance":400,"tsunami":false}"#,
        )
        .unwrap()
    }

    #[sqlx::test]
    async fn events_for_unknown_devices_are_rejected_as_invalid_input(pool: PgPool) {
        let strike =
            store_lightning_strikes(State(pool.clone()), Json(unknown_device_strike())).await;
        let quake = store_earthquake(State(pool), Json(unknown_device_earthquake())).await;

        assert!(
            matches!(strike, Err(HandlerError::InvalidInput(_))),
            "{strike:?}"
        );
        assert!(
            matches!(quake, Err(HandlerError::InvalidInput(_))),
            "{quake:?}"
        );
    }

    fn params(
        latitude: Option<f64>,
        longitude: Option<f64>,
        radius_km: Option<f64>,
    ) -> GeoEventParams {
        GeoEventParams {
            start: Utc::now() - chrono::Duration::hours(1),
            end: None,
            latitude,
            longitude,
            radius_km,
            limit: None,
        }
    }

    #[quickcheck_macros::quickcheck]
    fn area_is_used_only_when_all_three_parts_are_given(
        latitude: Option<u8>,
        longitude: Option<u8>,
        radius_km: Option<u8>,
    ) -> bool {
        let parts = [latitude, longitude, radius_km];
        let given = parts.iter().filter(|p| p.is_some()).count();
        let result = params(
            latitude.map(|v| f64::from(v % 90)),
            longitude.map(|v| f64::from(v % 180)),
            radius_km.map(|v| f64::from(v) + 1.0),
        )
        .into_query(Utc::now());
        match given {
            0 => matches!(result, Ok(EventQuery { area: None, .. })),
            3 => matches!(result, Ok(EventQuery { area: Some(_), .. })),
            _ => result.is_err(),
        }
    }

    #[test]
    fn out_of_range_centre_and_non_positive_radius_are_rejected() {
        let now = Utc::now();
        assert!(
            params(Some(91.0), Some(0.0), Some(10.0))
                .into_query(now)
                .is_err()
        );
        assert!(
            params(Some(63.0), Some(10.0), Some(0.0))
                .into_query(now)
                .is_err()
        );
    }

    #[quickcheck_macros::quickcheck]
    fn limit_is_clamped_to_supported_range(limit: Option<i64>) -> bool {
        let query = GeoEventParams {
            limit,
            ..params(None, None, None)
        }
        .into_query(Utc::now())
        .unwrap();
        (1..=MAX_EVENTS_LIMIT).contains(&query.limit)
            && limit.is_none_or(|l| query.limit == l.clamp(1, MAX_EVENTS_LIMIT))
    }
}
