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
use metrics_exporter_prometheus::PrometheusHandle;
use moka::future::Cache;
use sensors::fetch_sensors_by_device_id;
use sensors::{delete_sensor, fetch_sensors, insert_sensor, update_sensor};
use sqlx::Pool;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

use crate::measurements::Measurement;

mod devices;
mod error;
mod measurements;
mod sensors;

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

pub fn create_router(
    connection: Pool<sqlx::Postgres>,
    metrics_handler: PrometheusHandle,
    cache: Cache<(i32, i32), Measurement>,
) -> Router {
    let measurements = Router::new()
        .route("/measurements", get(fetch_all_measurements))
        .route("/measurements/latest", get(fetch_latest_measurement))
        .route(
            "/measurements/latest/all",
            get(fetch_all_latest_measurements),
        )
        .route("/measurements/count", get(fetch_measurements_count))
        .route("/measurements", post(store_measurements))
        .with_state((connection.clone(), cache.clone()));

    let devices = Router::new()
        .route("/devices", get(fetch_devices))
        .route("/devices", post(insert_device))
        .route("/devices", delete(delete_device))
        .route("/devices", put(update_device))
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

    let sensors = Router::new()
        .route("/sensors", get(fetch_sensors))
        .route("/sensors", post(insert_sensor))
        .route("/sensors", delete(delete_sensor))
        .route("/sensors", put(update_sensor));

    Router::new()
        .nest("/api", measurements)
        .nest("/api", devices)
        .nest("/api", sensors)
        .with_state(connection.clone())
        .route("/", post(store_measurements))
        .with_state((connection.clone(), cache.clone()))
        .route("/metrics", get(metrics))
        .with_state(metrics_handler)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(profile_endpoint)),
        )
}

#[instrument]
async fn metrics(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}
