use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::sensors::{NewSensor, Sensor};

use super::error::HandlerError;

#[instrument]
pub async fn fetch_sensors(State(pool): State<PgPool>) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensors))
}

#[instrument]
pub async fn fetch_sensor_by_sensor_id(
    State(pool): State<PgPool>,
    Path(sensor_id): Path<i32>,
) -> Result<Json<Sensor>, HandlerError> {
    let sensor = Sensor::read_by_id(&pool, sensor_id).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
    })?;
    Ok(Json(sensor))
}

#[instrument]
pub async fn insert_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<NewSensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.insert(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn delete_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.delete(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn update_sensor(
    State(pool): State<PgPool>,
    Json(sensor): Json<Sensor>,
) -> Result<String, HandlerError> {
    if sensor.name.is_empty() || sensor.unit.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    sensor.update(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {}", e))
    })?;
    Ok("OK".to_string())
}

#[instrument]
pub async fn fetch_sensors_by_device_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read_by_device_id(&pool, device_id)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {}", e))
        })?;
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

        let sensors = Sensors::read(&pool).await.unwrap();
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

        let sensors = Sensors::read(&pool).await.unwrap();
        assert!(!sensors.is_empty());

        let updated_sensor = Sensors::new(
            sensors[0].id,
            "Updated Light".to_string(),
            "Updated Lux".to_string(),
        );
        let result = update_sensor(State(pool.clone()), Json(updated_sensor)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK".to_string());

        let sensors_after_update = Sensors::read(&pool).await.unwrap();
        assert_eq!(sensors_after_update[0].name, "Updated Light");
    }
}
