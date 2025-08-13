use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::devices::{Device, NewDevice};

use super::error::HandlerError;

/// HTTP handler to retrieve all devices.
///
/// This endpoint fetches all registered devices from the database and returns them
/// as JSON. Used by management interfaces to display device listings.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
///
/// # Returns
///
/// JSON array of all devices, or a 500 error if database query fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON array of devices
/// - `500 Internal Server Error` - Database error occurred
#[instrument]
pub async fn fetch_devices(State(pool): State<PgPool>) -> Result<Json<Vec<Device>>, HandlerError> {
    let devices = Device::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;
    Ok(Json(devices))
}

/// HTTP handler to retrieve a specific device by ID.
///
/// This endpoint fetches a single device by its unique identifier.
/// Used when detailed device information is needed.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `device_id` - Device ID extracted from URL path parameter
///
/// # Returns
///
/// JSON representation of the device, or error if not found or database fails.
///
/// # HTTP Response
///
/// - `200 OK` - Returns JSON representation of the device
/// - `500 Internal Server Error` - Database error or device not found
#[instrument]
pub async fn fetch_devices_by_id(
    State(pool): State<PgPool>,
    Path(device_id): Path<i32>,
) -> Result<Json<Device>, HandlerError> {
    let device = Device::read_by_id(&pool, device_id).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;
    Ok(Json(device))
}

/// HTTP handler to create a new device.
///
/// This endpoint accepts device information and creates a new device record
/// in the database. Validates that name and location are not empty before insertion.
/// Also refreshes the device_sensors materialized view after insertion.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `device` - New device data from JSON request body
///
/// # Returns
///
/// Success message or error if validation fails or database insertion fails.
///
/// # HTTP Response
///
/// - `200 OK` - Device created successfully, returns "OK"
/// - `400 Bad Request` - Invalid input (empty name or location)
/// - `500 Internal Server Error` - Database error occurred
///
/// # Validation
///
/// - Device name must not be empty
/// - Device location must not be empty
#[instrument]
pub async fn insert_device(
    State(pool): State<PgPool>,
    Json(device): Json<NewDevice>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.insert(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {e}"))
    })?;
    Ok("OK".to_string())
}

/// HTTP handler to delete a device.
///
/// This endpoint removes a device from the database. The device data is provided
/// in the request body and must pass validation before deletion. This operation
/// may cascade to related measurements depending on database constraints.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `device` - Device data from JSON request body (must include ID)
///
/// # Returns
///
/// Success message or error if validation fails or database deletion fails.
///
/// # HTTP Response
///
/// - `200 OK` - Device deleted successfully, returns "OK"
/// - `400 Bad Request` - Invalid input (empty name or location)
/// - `500 Internal Server Error` - Database error or foreign key constraint violation
///
/// # Validation
///
/// - Device name must not be empty
/// - Device location must not be empty
#[instrument]
pub async fn delete_device(
    State(pool): State<PgPool>,
    Json(device): Json<Device>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.delete(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {e}"))
    })?;
    Ok("OK".to_string())
}

/// HTTP handler to update an existing device.
///
/// This endpoint updates device information in the database. The device data
/// (including ID) is provided in the request body and must pass validation.
/// After updating, refreshes the device_sensors materialized view to maintain consistency.
///
/// # Arguments
///
/// * `pool` - Database connection pool from Axum state
/// * `device` - Complete device data from JSON request body (must include ID)
///
/// # Returns
///
/// Success message or error if validation fails or database update fails.
///
/// # HTTP Response
///
/// - `200 OK` - Device updated successfully, returns "OK"
/// - `400 Bad Request` - Invalid input (empty name or location)
/// - `500 Internal Server Error` - Database error or device not found
///
/// # Validation
///
/// - Device name must not be empty
/// - Device location must not be empty
#[instrument]
pub async fn update_device(
    State(pool): State<PgPool>,
    Json(device): Json<Device>,
) -> Result<String, HandlerError> {
    if device.name.is_empty() || device.location.is_empty() {
        return Err(HandlerError::new(400, "Invalid input".to_string()));
    }
    device.update(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to store data in database: {e}"))
    })?;
    Ok("OK".to_string())
}

#[cfg(test)]

mod tests {
    use super::*;

    #[sqlx::test]
    async fn should_insert_device(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());

        let result = insert_device(State(pool), Json(device)).await;
        assert!(result.is_ok());
    }

    #[sqlx::test]
    async fn should_fetch_devices(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let result = fetch_devices(State(pool)).await;
        assert!(result.is_ok());
        let devices = result.unwrap().0;
        assert!(!devices.is_empty());
        assert_eq!(devices[0].name, "test");
        assert_eq!(devices[0].location, "test");
    }

    #[sqlx::test]
    async fn should_delete_device(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let devices = Device::read(&pool).await.unwrap();
        let result = delete_device(State(pool.clone()), Json(devices[0].clone())).await;
        assert!(result.is_ok());

        let devices_after_delete = Device::read(&pool).await.unwrap();
        assert!(devices_after_delete.is_empty());
    }

    #[sqlx::test]
    async fn should_update_device(pool: PgPool) {
        let device = NewDevice::new("test".to_string(), "test".to_string());
        device.insert(&pool).await.unwrap();

        let devices = Device::read(&pool).await.unwrap();
        let updated_device =
            Device::new(devices[0].id, "updated".to_string(), "updated".to_string());
        let result = update_device(State(pool.clone()), Json(updated_device)).await;
        assert!(result.is_ok());

        let devices_after_update = Device::read(&pool).await.unwrap();
        assert_eq!(devices_after_update[0].name, "updated");
        assert_eq!(devices_after_update[0].location, "updated");
    }
}
