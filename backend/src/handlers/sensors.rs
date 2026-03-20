use anyhow::Context;
use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::sensors::{NewSensor, Sensor};

use super::error::HandlerError;

#[utoipa::path(
    get,
    path = "api/sensors",
    responses(
        (status = 200, description = "List of sensors", body = [Sensor]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_sensors(State(pool): State<PgPool>) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read(&pool)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(sensors))
}

#[utoipa::path(
    get,
    path = "api/sensors/{sensor_id}",
    params(
        ("sensor_id" = i32, Path, description = "Sensor ID")
    ),
    responses(
        (status = 200, description = "Sensor found", body = Sensor),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_sensor_by_sensor_id(
    State(pool): State<PgPool>,
    Path(sensor_id): Path<i32>,
) -> Result<Json<Sensor>, HandlerError> {
    let sensor = Sensor::read_by_id(&pool, sensor_id)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(sensor))
}

#[utoipa::path(
    post,
    path = "api/sensors",
    request_body = NewSensor,
    responses(
        (status = 200, description = "Sensor created successfully", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn insert_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<NewSensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor
        .insert(&pool)
        .await
        .context("Failed to store data in database")?;
    Ok("OK".to_string())
}

#[utoipa::path(
    delete,
    path = "api/sensors",
    request_body = Sensor,
    responses(
        (status = 200, description = "Sensor deleted successfully", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn delete_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor
        .delete(&pool)
        .await
        .context("Failed to store data in database")?;
    Ok("OK".to_string())
}

#[utoipa::path(
    put,
    path = "api/sensors",
    request_body = Sensor,
    responses(
        (status = 200, description = "Sensor updated successfully", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn update_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor
        .update(&pool)
        .await
        .context("Failed to store data in database")?;
    Ok("OK".to_string())
}

#[utoipa::path(
    get,
    path = "api/sensors/device/{device_id}",
    params(
        ("device_id" = i32, Path, description = "Device ID")
    ),
    responses(
        (status = 200, description = "List of sensors for device", body = [Sensor]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_sensors_by_device_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read_by_device_id(&pool, device_id)
        .await
        .context("Failed to fetch data from database")?;
    Ok(Json(sensors))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn should_insert_sensor(pool: PgPool) {
        let sensor = NewSensor {
            name: "Temperature".to_string(),
            unit: "Celsius".to_string(),
        };

        let result = insert_sensor(State(pool), Json(sensor)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK".to_string());
    }

    #[sqlx::test]
    async fn should_fetch_sensors(pool: PgPool) {
        let sensor = NewSensor {
            name: "Humidity".to_string(),
            unit: "Percent".to_string(),
        };
        sensor.insert(&pool).await.unwrap();

        let result = fetch_sensors(State(pool)).await;
        assert!(result.is_ok());
        let sensors = result.unwrap().0;
        assert!(!sensors.is_empty());
        assert_eq!(sensors[0].name, "Humidity");
        assert_eq!(sensors[0].unit, "Percent");
    }

    #[sqlx::test]
    async fn should_delete_sensor(pool: PgPool) {
        let sensor = NewSensor {
            name: "Pressure".to_string(),
            unit: "Pascal".to_string(),
        };
        sensor.insert(&pool).await.unwrap();

        let sensors = Sensor::read(&pool).await.unwrap();
        assert!(!sensors.is_empty());

        let result = delete_sensor(State(pool), Json(sensors[0].clone())).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK".to_string());
    }

    #[sqlx::test]
    async fn should_update_sensor(pool: PgPool) {
        let sensor = NewSensor {
            name: "Light".to_string(),
            unit: "Lux".to_string(),
        };
        sensor.insert(&pool).await.unwrap();

        let sensors = Sensor::read(&pool).await.unwrap();
        assert!(!sensors.is_empty());

        let updated_sensor = Sensor::new(
            sensors[0].id,
            "Updated Light".to_string(),
            "Updated Lux".to_string(),
        );
        let result = update_sensor(State(pool.clone()), Json(updated_sensor)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK".to_string());

        let sensors_after_update = Sensor::read(&pool).await.unwrap();
        assert_eq!(sensors_after_update[0].name, "Updated Light");
    }
}
