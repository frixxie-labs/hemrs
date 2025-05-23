use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use futures::future::join_all;
use metrics::counter;
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::measurements::{Measurement, MeasurementStats, NewMeasurements};

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

#[instrument]
pub async fn fetch_stats_by_device_id_and_sensor_id(
    State(pool): State<PgPool>,
    Path((device_id, sensor_id)): Path<(i32, i32)>,
) -> Result<Json<MeasurementStats>, HandlerError> {
    let stats = Measurement::read_stats_by_device_id_and_sensor_id(&pool, device_id, sensor_id)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
    Ok(Json(stats))
}

#[cfg(test)]

mod tests {
    use crate::{devices::NewDevice, measurements::NewMeasurement, sensors::NewSensor};

    use super::*;

    #[sqlx::test]
    async fn should_store_single_measurement_without_ts(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurement = NewMeasurement::new(None, 1, 1, 1.0);
        let result = store_measurements(
            State(db),
            Json(NewMeasurements::Measurement(new_measurement)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }

    #[sqlx::test]
    async fn should_store_single_measurement_with_ts(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurement = NewMeasurement::new(Some(chrono::Utc::now()), 1, 1, 1.0);
        let result = store_measurements(
            State(db),
            Json(NewMeasurements::Measurement(new_measurement)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }

    #[sqlx::test]
    async fn should_store_multiple_measurements(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurements = vec![
            NewMeasurement::new(None, 1, 1, 1.0),
            NewMeasurement::new(None, 1, 1, 2.0),
        ];
        let result = store_measurements(
            State(db),
            Json(NewMeasurements::Measurements(new_measurements)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }

    #[sqlx::test]
    async fn should_store_multiple_measurements_with_and_without_ts(db: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&db).await.unwrap();
        let sensor = NewSensor::new("test".to_string(), "test".to_string());
        sensor.insert(&db).await.unwrap();
        let new_measurements = vec![
            NewMeasurement::new(None, 1, 1, 1.0),
            NewMeasurement::new(Some(chrono::Utc::now()), 1, 1, 2.0),
        ];
        let result = store_measurements(
            State(db),
            Json(NewMeasurements::Measurements(new_measurements)),
        )
        .await
        .unwrap();
        assert_eq!(result.status(), 201);
    }
}
