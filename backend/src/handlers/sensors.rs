use axum::{extract::State, Json};
use log::{info, warn};

use crate::{
    database::{db_connection_pool::Postgres, read::Read},
    sensors::Sensor,
};

use super::error::HandlerError;

pub async fn fetch_sensors(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    info!("GET api/sensors");
    let sensors = Vec::<Sensor>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}
