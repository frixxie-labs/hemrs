use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;
use tracing::{instrument, warn};

use crate::devices::{Device, NewDevice};

use super::error::HandlerError;

#[utoipa::path(
    get,
    path = "api/devices",
    responses(
        (status = 200, description = "List of devices", body = [Device]),
        (status = 500, description = "Internal server error"),
    )
)]
#[instrument]
pub async fn fetch_devices(State(pool): State<PgPool>) -> Result<Json<Vec<Device>>, HandlerError> {
    let devices = Device::read(&pool).await.map_err(|e| {
        warn!("Failed with error: {}", e);
        HandlerError::new(500, format!("Failed to fetch data from database: {e}"))
    })?;
    Ok(Json(devices))
}

#[utoipa::path(
    get,
    path = "api/devices/{device_id}",
    params(
        ("device_id" = i32, Path, description = "Device ID")
    ),
    responses(
        (status = 200, description = "Device found", body = Device),
        (status = 500, description = "Internal server error"),
    )
)]
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

#[utoipa::path(
    post,
    path = "api/devices",
    request_body = NewDevice,
    responses(
        (status = 200, description = "Device created successfully", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
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

#[utoipa::path(
    delete,
    path = "api/devices",
    request_body = Device,
    responses(
        (status = 200, description = "Device deleted successfully", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
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

#[utoipa::path(
    put,
    path = "api/devices",
    request_body = Device,
    responses(
        (status = 200, description = "Device updated successfully", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error"),
    )
)]
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
