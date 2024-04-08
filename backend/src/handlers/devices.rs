use axum::{extract::State, Json};
use log::{info, warn};

use crate::{
    database::{db_connection_pool::Postgres, read::Read},
    devices::Device,
};

use super::error::HandlerError;

pub async fn fetch_devices(
    State(pg_pool): State<Postgres>,
) -> Result<Json<Vec<Device>>, HandlerError> {
    info!("GET api/devices");
    let devices = Vec::<Device>::read(pg_pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(devices))
}
