use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use futures::future::join_all;
use metrics::counter;
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::measurements::{Measurement, NewMeasurements};

use super::error::HandlerError;

#[instrument]
pub async fn store_measurements(
    State(pool): State<PgPool>,
    Json(measurement): Json<NewMeasurements>,
) -> Result<Response, HandlerError>
where
    Response: IntoResponse,
{
    match measurement {
        NewMeasurements::Measurement(new_measurement) => {
            new_measurement.insert(&pool).await.map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to insert data into database: {}", e))
            })?;
            counter!("new_measurements").increment(1);
        }
        NewMeasurements::Measurements(new_measurements) => {
            let handles = new_measurements
                .into_iter()
                .map(async |m| {
                    m.insert(&pool).await.map_err(|e| {
                        warn!("Failed with error: {}", e);
                        HandlerError::new(
                            500,
                            format!("Failed to insert data into database: {}", e),
                        )
                    })
                })
                .collect::<Vec<_>>();
            let res = join_all(handles).await;
            for r in res {
                if let Err(e) = r {
                    warn!("Failed with error: {}", e);
                    return Err(HandlerError::new(
                        500,
                        format!("Failed to insert data into database: {}", e),
                    ));
                } else {
                    counter!("new_measurements").increment(1);
                }
            }
        }
    };
    let resp = Response::builder()
        .status(201)
        .body("Measurement(s) inserted successfully".into())
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to build response: {}", e))
        })?;
    Ok(resp)
}

#[instrument]
pub async fn fetch_latest_measurement(
    State(pool): State<PgPool>,
) -> Result<Json<Measurement>, HandlerError> {
    let entry = Measurement::read_latest(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(entry))
}

#[instrument]
pub async fn fetch_measurements_count(
    State(pool): State<PgPool>,
) -> Result<Json<usize>, HandlerError> {
    let entry = Measurement::read_all(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entry.len()))
}

#[instrument]
pub async fn fetch_all_measurements(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let entries = Measurement::read_all(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;

    Ok(Json(entries))
}

#[instrument]
pub async fn fetch_measurement_by_device_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let measurements = Measurement::read_by_device_id(device_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(measurements))
}

#[instrument]
pub async fn fetch_latest_measurement_by_device_id_and_sensor_id(
    State(pool): State<PgPool>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Measurement>, HandlerError> {
    let measurement =
        Measurement::read_latest_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
            .await
            .map_err(|e| {
                warn!("Failed with error: {}", e);
                HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
            })?;
    Ok(Json(measurement))
}

#[instrument]
pub async fn fetch_measurement_by_device_id_and_sensor_id(
    State(pool): State<PgPool>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let measurements = Measurement::read_by_device_id_and_sensor_id(device_id, sensor_id, &pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(measurements))
}

#[instrument]
pub async fn fetch_all_latest_measurements(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Measurement>>, HandlerError> {
    let measurements = Measurement::read_all_latest_measurements(&pool)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(measurements))
}
