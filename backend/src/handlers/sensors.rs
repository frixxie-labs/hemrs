use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::sensors::{NewSensor, Sensor};

use super::error::HandlerError;

/// HTTP handler to retrieve all sensors.
///
/// This endpoint fetches all registered sensors from the database and returns them
/// as JSON. Used by management interfaces to display sensor type listings.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
///
/// # Returns
///
/// JSON array of all sensors, or a 500 error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of sensors
/// - `500 Internal Server Error` - Database error occurred
#[instrument]
pub async fn fetch_sensors(State(pool): State<PgPool>) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;
    Ok(Json(sensors))
}

/// HTTP handler to retrieve a specific sensor by ID.
///
/// This endpoint fetches a single sensor by its unique identifier.
/// Used when detailed sensor information is needed.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `sensor_id` - Sensor ID extracted from URL path parameter
///
/// # Returns
///
/// JSON representation of the sensor, or error if not found or database fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON representation of the sensor
/// - `500 Internal Server Error` - Database error or sensor not found
#[instrument]
pub async fn fetch_sensor_by_sensor_id(
    State(pool): State<PgPool>,
    Path(sensor_id): Path<i32>,
) -> Result<Json<Sensor>, HandlerError> {
    let sensor = Sensor::read_by_id(&pool, sensor_id).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;
    Ok(Json(sensor))
}

/// HTTP handler to create a new sensor.
///
/// This endpoint accepts sensor information and creates a new sensor record
/// in the database. Validates that name and unit are not empty before insertion.
/// Also refreshes the device_sensors materialized view after insertion.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `sensor` - New sensor data from JSON request body
///
/// # Returns
///
/// Success message or error if validation fails or database insertion fails.
///
/// # HTTP Response
///
/// - `200 OK` - Sensor created successfully, returns "OK"
/// - `400 Bad Request` - Invalid input (empty name or unit)
/// - `500 Internal Server Error` - Database error occurred
///
/// # Validation
///
/// - Sensor name must not be empty
/// - Sensor unit must not be empty
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
        HandlerError::new(500, format!("Failed to store data in database: {e}"))
    })?;
    Ok("OK".to_string())
}

/// HTTP handler to delete a sensor.
///
/// This endpoint removes a sensor from the database. The sensor data is provided
/// in the request body and must pass validation before deletion. This operation
/// may cascade to related measurements depending on database constraints.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `sensor` - Sensor data from JSON request body (must include ID)
///
/// # Returns
///
/// Success message or error if validation fails or database deletion fails.
///
/// # HTTP Response
///
/// - `200 OK` - Sensor deleted successfully, returns "OK"
/// - `400 Bad Request` - Invalid input (empty name or unit)
/// - `500 Internal Server Error` - Database error or foreign key constraint violation
///
/// # Validation
///
/// - Sensor name must not be empty
/// - Sensor unit must not be empty
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
        HandlerError::new(500, format!("Failed to store data in database: {e}"))
    })?;
    Ok("OK".to_string())
}

/// HTTP handler to update an existing sensor.
///
/// This endpoint updates sensor information in the database. The sensor data
/// (including ID) is provided in the request body and must pass validation.
/// After updating, refreshes the device_sensors materialized view to maintain consistency.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `sensor` - Complete sensor data from JSON request body (must include ID)
///
/// # Returns
///
/// Success message or error if validation fails or database update fails.
///
/// # HTTP Response
///
/// - `200 OK` - Sensor updated successfully, returns "OK"
/// - `400 Bad Request` - Invalid input (empty name or unit)
/// - `500 Internal Server Error` - Database error or sensor not found
///
/// # Validation
///
/// - Sensor name must not be empty
/// - Sensor unit must not be empty
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
        HandlerError::new(500, format!("Failed to store data in database: {e}"))
    })?;
    Ok("OK".to_string())
}

/// HTTP handler to retrieve all sensors associated with a specific device.
///
/// This endpoint fetches sensors that have reported measurements from the specified device.
/// Uses the device_sensors materialized view for efficient querying. Results are ordered by sensor ID.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `device_id` - Device ID extracted from URL path parameter
///
/// # Returns
///
/// JSON array of sensors associated with the device, or error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of sensors for the device
/// - `500 Internal Server Error` - Database error or device not found
///
/// # Note
///
/// This method relies on the device_sensors materialized view being up-to-date.
/// If recent sensors are not appearing, the materialized view may need refreshing.
#[instrument]
pub async fn fetch_sensors_by_device_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Vec<Sensor>>, HandlerError> {
    let sensors = Sensor::read_by_device_id(&pool, device_id)
        .await
        .map_err(|e| {
            warn!("Failed with error: {}", e);
            HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
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
