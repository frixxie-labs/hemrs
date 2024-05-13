use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::Pool;

use crate::database::db_connection_pool;

use self::{
    devices::{delete_device, fetch_devices, insert_device, update_device},
    measurements::{fetch_all_measurements, fetch_latest_measurement, store_measurements},
    sensors::{delete_sensor, fetch_sensors, insert_sensor, update_sensor},
};

pub use measurements::InnerMeasurementQuery;

mod devices;
mod error;
mod measurements;
mod sensors;

pub fn create_router(connection: Pool<sqlx::Postgres>) -> Router {
    let pg_pool = db_connection_pool::Postgres::new(connection);

    let measurements = Router::new()
        .route("/measurements", get(fetch_all_measurements))
        .route("/measurements/latest", get(fetch_latest_measurement))
        .route("/measurements", post(store_measurements));

    let devices = Router::new()
        .route("/devices", get(fetch_devices))
        .route("/devices", post(insert_device))
        .route("/devices", delete(delete_device))
        .route("/devices", put(update_device));

    let sensors = Router::new()
        .route("/sensors", get(fetch_sensors))
        .route("/sensors", post(insert_sensor))
        .route("/sensors", delete(delete_sensor))
        .route("/sensors", put(update_sensor));

    Router::new()
        .nest("/api", measurements)
        .nest("/api", devices)
        .nest("/api", sensors)
        .route("/", post(store_measurements))
        .with_state(pg_pool)
}
