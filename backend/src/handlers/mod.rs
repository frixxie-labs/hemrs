use axum::{
    Router,
    extract::{MatchedPath, Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
};
use devices::{delete_device, fetch_devices, insert_device, update_device};
use geo_events::{
    fetch_earthquakes, fetch_lightning_strikes, store_earthquake, store_lightning_strikes,
};
use measurement_stream::{MeasurementStreamState, stream_measurements};
use measurements::{
    MeasurementState, fetch_all_latest_measurements, fetch_all_measurements,
    fetch_latest_measurement, fetch_latest_measurement_by_device_id_and_sensor_id,
    fetch_measurement_by_device_id, fetch_measurement_by_device_id_and_sensor_id,
    fetch_measurements_by_date_range, fetch_measurements_count,
    fetch_stats_by_device_id_and_sensor_id, store_measurements,
};
use metrics::histogram;
use metrics_exporter_prometheus::PrometheusHandle;
use ping::ping;
use sensors::fetch_sensors_by_device_id;
use sensors::{delete_sensor, fetch_sensors, insert_sensor, update_sensor};
use sqlx::Pool;
use tokio::{
    sync::{broadcast, mpsc::Sender},
    time::Instant,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};
use utoipa::OpenApi;

use crate::{
    handlers::{devices::fetch_devices_by_id, sensors::fetch_sensor_by_sensor_id},
    measurements::{MeasurementUpdate, NewMeasurement, latest::LatestMeasurements},
};

mod devices;
mod error;
mod geo_events;
mod measurement_stream;
mod measurements;
mod ping;
mod sensors;

#[instrument]
pub async fn profile_endpoint(request: Request, next: Next) -> Response {
    let method = request.method().clone().to_string();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    info!("Handling {} at {}", method, path);

    let now = Instant::now();

    let labels = [("method", method.clone()), ("path", path.clone())];

    let response = next.run(request).await;

    let elapsed = now.elapsed();

    histogram!("handler", &labels).record(elapsed);

    info!(
        "Finished handling {} at {}, used {} ms",
        method,
        path,
        elapsed.as_millis()
    );
    response
}

pub fn create_router(
    connection: Pool<sqlx::Postgres>,
    metrics_handler: PrometheusHandle,
    cache: LatestMeasurements,
    tx: Sender<NewMeasurement>,
    measurement_updates: broadcast::Sender<MeasurementUpdate>,
) -> Router {
    let measurement_state = MeasurementState {
        pool: connection.clone(),
        cache: cache.clone(),
    };

    let measurements = Router::new()
        .route("/measurements", get(fetch_all_measurements))
        .route("/measurements/latest", get(fetch_latest_measurement))
        .route(
            "/measurements/latest/all",
            get(fetch_all_latest_measurements),
        )
        .route("/measurements/count", get(fetch_measurements_count))
        .route("/measurements/range", get(fetch_measurements_by_date_range))
        .with_state(measurement_state.clone())
        .route("/measurements", post(store_measurements))
        .with_state(tx.clone());

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
        .with_state(measurement_state.clone());

    let sensors = Router::new()
        .route("/sensors", get(fetch_sensors))
        .route("/sensors", post(insert_sensor))
        .route("/sensors", delete(delete_sensor))
        .route("/sensors", put(update_sensor))
        .route("/sensors/{sensor_id}", get(fetch_sensor_by_sensor_id))
        .with_state(connection.clone());

    let geo_events = Router::new()
        .route("/lightning", get(fetch_lightning_strikes))
        .route("/lightning", post(store_lightning_strikes))
        .route("/earthquakes", get(fetch_earthquakes))
        .route("/earthquakes", post(store_earthquake))
        .with_state(connection.clone());

    let measurement_stream = Router::new()
        .route(
            "/devices/{device_id}/sensors/{sensor_id}/measurements/stream",
            get(stream_measurements),
        )
        .with_state(MeasurementStreamState {
            pool: connection.clone(),
            cache,
            updates: measurement_updates,
        });

    Router::new()
        .nest("/api", measurements)
        .nest("/api", devices)
        .nest("/api", sensors)
        .nest("/api", geo_events)
        .nest("/api", measurement_stream)
        .route("/", post(store_measurements))
        .with_state(tx)
        .route("/status/ping", get(ping))
        .route("/metrics", get(metrics))
        .route("/openapi", get(get_openapi))
        .with_state(metrics_handler)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(profile_endpoint)),
        )
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics in exposition format", body = String),
    )
)]
#[instrument]
async fn metrics(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}

#[derive(OpenApi)]
#[openapi(
    paths(
        // Metrics and health
        metrics,
        ping::ping,

        // Devices
        devices::fetch_devices,
        devices::fetch_devices_by_id,
        devices::insert_device,
        devices::update_device,
        devices::delete_device,

        // Sensors
        sensors::fetch_sensors,
        sensors::fetch_sensor_by_sensor_id,
        sensors::insert_sensor,
        sensors::update_sensor,
        sensors::delete_sensor,
        sensors::fetch_sensors_by_device_id,

        // Measurements
        measurements::store_measurements,
        measurements::fetch_latest_measurement,
        measurements::fetch_measurements_count,
        measurements::fetch_all_measurements,
        measurements::fetch_measurement_by_device_id,
        measurements::fetch_latest_measurement_by_device_id_and_sensor_id,
        measurements::fetch_measurement_by_device_id_and_sensor_id,
        measurements::fetch_all_latest_measurements,
        measurements::fetch_stats_by_device_id_and_sensor_id,
        measurements::fetch_measurements_by_date_range,
        measurement_stream::stream_measurements,

        // Geo events
        geo_events::store_lightning_strikes,
        geo_events::fetch_lightning_strikes,
        geo_events::store_earthquake,
        geo_events::fetch_earthquakes,
    ),
    components(
        schemas(
            crate::devices::Device,
            crate::devices::NewDevice,
            crate::coordinates::Coordinates,
            crate::sensors::Sensor,
            crate::sensors::NewSensor,
            crate::measurements::Measurement,
            crate::measurements::NewMeasurement,
            crate::measurements::NewMeasurements,
            crate::measurements::MeasurementStats,
            crate::measurements::MeasurementUpdate,
            crate::geo_events::lightning::NewLightningStrike,
            crate::geo_events::lightning::NewLightningStrikes,
            crate::geo_events::lightning::LightningStrike,
            crate::geo_events::earthquakes::NewEarthquake,
            crate::geo_events::earthquakes::Earthquake,
            ping::PingResponse,
        )
    ),
    tags(
        (name = "devices", description = "Device management endpoints"),
        (name = "sensors", description = "Sensor management endpoints"),
        (name = "measurements", description = "Measurement data endpoints"),
        (name = "geo_events", description = "Lightning strike and earthquake endpoints"),
        (name = "system", description = "System health and metrics endpoints"),
    )
)]
pub struct ApiDoc;

pub async fn get_openapi() -> String {
    let doc = ApiDoc::openapi();
    serde_json::to_string_pretty(&doc).unwrap()
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn path_parameters_appear_in_their_path_template() {
        let doc: serde_json::Value = serde_json::from_str(&super::get_openapi().await).unwrap();
        let misplaced: Vec<String> = doc["paths"]
            .as_object()
            .unwrap()
            .iter()
            .flat_map(|(path, operations)| {
                operations
                    .as_object()
                    .unwrap()
                    .values()
                    .flat_map(|op| op["parameters"].as_array().cloned().unwrap_or_default())
                    .filter(move |p| {
                        p["in"] == "path"
                            && !path.contains(&format!("{{{}}}", p["name"].as_str().unwrap()))
                    })
                    .map(move |p| format!("{path}: {}", p["name"]))
            })
            .collect();
        assert!(misplaced.is_empty(), "{misplaced:?}");
    }

    /// Run `just openapi` to regenerate the committed spec after API changes.
    #[tokio::test]
    async fn committed_openapi_spec_matches_the_code() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../dev_docs/openapi.json");
        let generated = super::get_openapi().await + "\n";
        if std::env::var_os("UPDATE_OPENAPI").is_some() {
            std::fs::write(path, &generated).unwrap();
        }
        let committed = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            committed == generated,
            "dev_docs/openapi.json is out of date; run `just openapi`"
        );
    }
}
