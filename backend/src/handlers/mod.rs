use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Router,
};
use devices::{delete_device, fetch_devices, insert_device, update_device};
use measurements::{
    fetch_all_latest_measurements, fetch_all_measurements, fetch_latest_measurement,
    fetch_latest_measurement_by_device_id_and_sensor_id, fetch_measurement_by_device_id,
    fetch_measurement_by_device_id_and_sensor_id, fetch_measurements_count,
    fetch_stats_by_device_id_and_sensor_id, store_measurements,
};
use metrics::histogram;
use ping::ping;
use metrics_exporter_prometheus::PrometheusHandle;
use moka::future::Cache;
use sensors::fetch_sensors_by_device_id;
use sensors::{delete_sensor, fetch_sensors, insert_sensor, update_sensor};
use sqlx::Pool;
use tokio::{sync::mpsc::Sender, time::Instant};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

use crate::{
    handlers::{devices::fetch_devices_by_id, sensors::fetch_sensor_by_sensor_id},
    measurements::{Measurement, NewMeasurement},
};

mod devices;
mod error;
mod measurements;
mod ping;
mod sensors;

/// Middleware function that profiles HTTP request handling performance.
///
/// This middleware measures the time taken to handle each HTTP request and records
/// performance metrics using the metrics library. It also logs the start and end
/// of request processing for debugging and monitoring purposes.
///
/// # Arguments
///
/// * `request` - The incoming HTTP request
/// * `next` - The next middleware or handler in the chain
///
/// # Returns
///
/// The HTTP response from the downstream handlers, unchanged.
///
/// # Metrics
///
/// Records a histogram metric named "handler" with labels for HTTP method and URI.
#[instrument]
pub async fn profile_endpoint(request: Request, next: Next) -> Response {
    let method = request.method().clone().to_string();
    let uri = request.uri().clone().to_string();
    info!("Handling {} at {}", method, uri);

    let now = Instant::now();

    let labels = [("method", method.clone()), ("uri", uri.clone())];

    let response = next.run(request).await;

    let elapsed = now.elapsed();

    histogram!("handler", &labels).record(elapsed);

    info!(
        "Finished handling {} at {}, used {} ms",
        method,
        uri,
        elapsed.as_millis()
    );
    response
}

/// Creates and configures the main application router with all API endpoints.
///
/// This function sets up the complete routing structure for the IoT monitoring system,
/// including endpoints for devices, sensors, measurements, and system metrics.
/// It also configures middleware for request profiling and tracing.
///
/// # Arguments
///
/// * `connection` - PostgreSQL database connection pool
/// * `metrics_handler` - Prometheus metrics handler for /metrics endpoint
/// * `cache` - In-memory cache for recent measurements (60s TTL)
/// * `tx` - Channel sender for asynchronous measurement insertion
///
/// # Returns
///
/// A configured Axum Router ready to serve HTTP requests.
///
/// # API Endpoints
///
/// ## Devices
/// - `GET /api/devices` - List all devices
/// - `POST /api/devices` - Create a new device
/// - `PUT /api/devices` - Update an existing device
/// - `DELETE /api/devices` - Delete a device
/// - `GET /api/devices/{id}` - Get device by ID
/// - `GET /api/devices/{id}/sensors` - Get sensors for a device
/// - `GET /api/devices/{id}/measurements` - Get measurements for a device
/// - `GET /api/devices/{id}/sensors/{sensor_id}/measurements` - Get measurements for device-sensor pair
/// - `GET /api/devices/{id}/sensors/{sensor_id}/measurements/latest` - Get latest measurement
/// - `GET /api/devices/{id}/sensors/{sensor_id}/measurements/stats` - Get statistical summary
///
/// ## Sensors
/// - `GET /api/sensors` - List all sensors
/// - `POST /api/sensors` - Create a new sensor
/// - `PUT /api/sensors` - Update an existing sensor
/// - `DELETE /api/sensors` - Delete a sensor
/// - `GET /api/sensors/{id}` - Get sensor by ID
///
/// ## Measurements
/// - `GET /api/measurements` - Get all measurements
/// - `POST /api/measurements` - Store new measurements
/// - `GET /api/measurements/latest` - Get latest measurement from any device
/// - `GET /api/measurements/latest/all` - Get latest measurements from all device-sensor pairs
/// - `GET /api/measurements/count` - Get total measurement count
///
/// ## System
/// - `POST /` - Direct measurement ingestion endpoint
/// - `GET /status/ping` - Health check endpoint
/// - `GET /metrics` - Prometheus metrics endpoint
pub fn create_router(
    connection: Pool<sqlx::Postgres>,
    metrics_handler: PrometheusHandle,
    cache: Cache<(i32, i32), Measurement>,
    tx: Sender<NewMeasurement>,
) -> Router {
    // Measurements router - needs connection+cache for GET routes, tx for POST routes
    let measurements = Router::new()
        .route("/measurements", get(fetch_all_measurements))
        .route("/measurements/latest", get(fetch_latest_measurement))
        .route(
            "/measurements/latest/all",
            get(fetch_all_latest_measurements),
        )
        .route("/measurements/count", get(fetch_measurements_count))
        .with_state((connection.clone(), cache.clone()))
        .route("/measurements", post(store_measurements))
        .with_state(tx.clone());

    // Devices router - needs connection for CRUD, connection+cache for measurement routes
    let devices = Router::new()
        .route("/devices", get(fetch_devices))
        .route("/devices", post(insert_device))
        .route("/devices", delete(delete_device))
        .route("/devices", put(update_device))
        .route("/devices/{device_id}", get(fetch_devices_by_id))
        .route(
            "/devices/{device_id}/sensors",
            get(fetch_sensors_by_device_id),
        )
        .with_state(connection.clone())
        .route(
            "/devices/{device_id}/measurements",
            get(fetch_measurement_by_device_id),
        )
        .route(
            "/devices/{device_id}/sensors/{sensor_id}/measurements",
            get(fetch_measurement_by_device_id_and_sensor_id),
        )
        .route(
            "/devices/{device_id}/sensors/{sensor_id}/measurements/latest",
            get(fetch_latest_measurement_by_device_id_and_sensor_id),
        )
        .route(
            "/devices/{device_id}/sensors/{sensor_id}/measurements/stats",
            get(fetch_stats_by_device_id_and_sensor_id),
        )
        .with_state((connection.clone(), cache.clone()));

    // Sensors router - only needs connection for CRUD operations
    let sensors = Router::new()
        .route("/sensors", get(fetch_sensors))
        .route("/sensors", post(insert_sensor))
        .route("/sensors", delete(delete_sensor))
        .route("/sensors", put(update_sensor))
        .route("/sensors/{sensor_id}", get(fetch_sensor_by_sensor_id))
        .with_state(connection.clone());

    // Main router - each nested router already has its required state
    Router::new()
        .nest("/api", measurements)
        .nest("/api", devices)
        .nest("/api", sensors)
        .route("/", post(store_measurements))
        .with_state(tx)
        .route("/status/ping", get(ping))
        .route("/metrics", get(metrics))
        .with_state(metrics_handler)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(profile_endpoint)),
        )
}

/// HTTP handler that serves Prometheus metrics.
///
/// This endpoint provides system metrics in Prometheus format, including
/// request performance metrics recorded by the profiling middleware and
/// any custom application metrics.
///
/// # Arguments
///
/// * `handle` - Prometheus handle containing the metrics registry
///
/// # Returns
///
/// String containing metrics in Prometheus exposition format.
#[instrument]
async fn metrics(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}
