use axum::{routing::{get, post}, Router};
use sqlx::Pool;


use crate::database::db_connection_pool;

use self::{devices::fetch_devices, measurements::{fetch_all_measurements, fetch_latest_measurement, store_measurements}, sensors::fetch_sensors};

mod measurements;
mod sensors;
mod devices;
mod error;

pub fn create_router(connection: Pool<sqlx::Postgres>) -> Router {
    let pg_pool = db_connection_pool::Postgres::new(connection);

    let measurements = Router::new()
        .route("/measurements", get(fetch_all_measurements))
        .route("/measurements/latest", get(fetch_latest_measurement));

    let devices = Router::new().route("/devices", get(fetch_devices));

    let sensors = Router::new().route("/sensors", get(fetch_sensors));

    let app = Router::new()
        .nest("/api", measurements)
        .nest("/api", devices)
        .nest("/api", sensors)
        .route("/", post(store_measurements))
        .with_state(pg_pool);
    app
}

