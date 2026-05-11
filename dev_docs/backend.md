# Backend

## Technology

- Rust 2021.
- Axum `0.8` for HTTP routing and extractors.
- SQLx `0.8` for PostgreSQL access and compile-time checked queries.
- Tokio for async runtime, TCP listener, channels, and background tasks.
- Moka future cache for latest measurements.
- `metrics` plus `metrics-exporter-prometheus` for Prometheus output.
- `tracing` and `tower-http` trace middleware for request logging/instrumentation.
- `utoipa` for OpenAPI schema and path generation.
- `quickcheck` and `sqlx::test` for tests.

## Startup

`backend/src/main.rs` does the following:

1. Parses `Opts` from CLI/env with Clap.
2. Configures tracing with the selected log level.
3. Installs a Prometheus recorder.
4. Opens a PostgreSQL pool using `PgPoolOptions`.
5. Builds a Moka cache keyed by `(device_id, sensor_id)` with capacity `128` and TTL `60s`.
6. Creates a Tokio channel for measurement ingestion with buffer `1 << 13`.
7. Builds the Axum router with cloned pool/cache/channel state.
8. Starts concurrent tasks with `tokio::select!`: metrics update loop, insert worker, materialized view refresh loop, and HTTP server.

## Configuration

CLI options are defined in `Opts`:

- `--host` / `-h`: bind address, default `0.0.0.0:65534`.
- `--db-url` / `-d` / `DATABASE_URL`: PostgreSQL URL, default `postgres://postgres:example@localhost:5432/postgres`.
- `--log-level` / `-l`: one of `trace`, `debug`, `info`, `warn`, `error`; default `info`.

## Routing

All active routes are registered in `backend/src/handlers/mod.rs`.

The router is composed from three nested routers:

- `measurements`: gets pool/cache state for reads and channel sender state for writes.
- `devices`: gets pool state for device/sensor list reads and pool/cache state for measurement reads.
- `sensors`: gets pool state.

Global middleware:

- `TraceLayer::new_for_http()` from `tower-http`.
- `profile_endpoint`, which records a `handler` histogram labeled by method and matched path.

## Handler Pattern

Handlers generally follow this pattern:

1. Extract Axum state/path/query/body.
2. Validate simple required fields when creating/updating/deleting devices or sensors.
3. Call an inherent async method on a domain struct.
4. Add `anyhow::Context` around database or channel failures.
5. Return `Json<T>`, plain text `String`, or a built `Response`.

Use `#[instrument]` on handlers to preserve tracing behavior.

## Error Handling

`backend/src/handlers/error.rs` defines `HandlerError`:

- Stores `status: u16` and `message: String`.
- Implements `From<anyhow::Error>` as status `500`.
- Implements `IntoResponse` as a plain text response.

When adding explicit validation errors, use `HandlerError::new(status, message)`. When wrapping fallible operations, prefer `.context("...")?` so logs and responses include useful context.

## Data Model and Access

The backend currently keeps model definitions and data-access methods in the same modules.

### Devices

`NewDevice` fields:

- `name: String`
- `location: String`

`Device` fields:

- `id: i32`
- `name: String`
- `location: String`

Main methods:

- `Device::read`
- `Device::read_by_id`
- `Device::delete`
- `Device::update`
- `Device::refresh_device_sensors_view`
- `NewDevice::insert`

Device insert/update refreshes `device_sensors`.

### Sensors

`NewSensor` fields:

- `name: String`
- `unit: String`

`Sensor` fields:

- `id: i32`
- `name: String`
- `unit: String`

Main methods:

- `Sensor::read`
- `Sensor::read_by_id`
- `Sensor::read_by_device_id`
- `Sensor::delete`
- `Sensor::update`
- `NewSensor::insert`

Sensor insert/update refreshes `device_sensors`. `Sensor::read_by_device_id` reads from the materialized view.

### Measurements

`NewMeasurement` fields:

- `timestamp: Option<DateTime<Utc>>`
- `device: i32`
- `sensor: i32`
- `measurement: f32`

`NewMeasurements` is an untagged enum accepting either a single `NewMeasurement` object or a JSON array.

`Measurement` response fields:

- `timestamp: DateTime<Utc>`
- `value: f32`
- `unit: String`
- `device_name: String`
- `device_location: String`
- `sensor_name: String`

Main methods:

- `NewMeasurement::insert`
- `Measurement::read_all`
- `Measurement::read_latest`
- `Measurement::read_all_latest_measurements`
- `Measurement::read_total_measurements`
- `Measurement::read_by_device_id`
- `Measurement::read_by_device_id_and_sensor_id`
- `Measurement::read_latest_by_device_id_and_sensor_id`
- `Measurement::read_stats_by_device_id_and_sensor_id`
- `Measurement::read_by_date_range`

## Measurement Ingestion

`store_measurements` does not insert into PostgreSQL directly. It sends each `NewMeasurement` through `Sender<NewMeasurement>` and returns `201` once enqueueing succeeds.

`handle_insert_measurement_bg_thread` receives measurements and calls private `insert_measurement`:

- Looks up device and sensor concurrently with `tokio::join!`.
- Builds a denormalized `Measurement` cache entry.
- Inserts cache entry under `(device.id, sensor.id)`.
- Inserts the measurement row into PostgreSQL.
- Records histograms for lookup, cache insert, and database insert durations.
- Increments `new_measurements` on success.

## Caching

The latest-measurement cache is a Moka future cache:

- Key: `(i32, i32)` representing `(device_id, sensor_id)`.
- Value: denormalized `Measurement`.
- Capacity: `128`.
- TTL: `60s`.

Cache use cases:

- The insert worker writes latest values immediately after resolving metadata.
- `fetch_latest_measurement_by_device_id_and_sensor_id` checks cache before querying PostgreSQL.
- `update_metrics` reads cache first and falls back to PostgreSQL.

## Metrics

Prometheus metrics are exposed at `/metrics`.

Recorded metrics include:

- `handler` histogram labeled by method and path.
- `db_insert_duration_seconds` histogram.
- `device_sensor_lookup_duration_seconds` histogram.
- `cache_insert_duration_seconds` histogram.
- `measurement_insert_duration_seconds` histogram.
- `new_measurements` counter.
- `hemrs_pg_pool_size` absolute counter-style metric.
- `hemrs_cache_size` absolute counter-style metric.
- `measurements` gauge labeled by `device_name`, `device_location`, `sensor_name`, and `unit` for values newer than 300 seconds.

## Background View Refresh

`refresh_views` refreshes the `device_sensors` materialized view every 6000 seconds. Device/sensor insert and update paths also refresh the view explicitly.

## Tests

Test styles used in backend code:

- Pure constructor/serialization/display behavior uses `quickcheck_macros::quickcheck`.
- Database behavior uses `#[sqlx::test]`, which provisions a test database using migrations.
- Handler tests call handler functions directly with Axum `State`, `Json`, `Path`, or `Query` extractors.

## SQLx Offline Mode

The backend Dockerfile and CI set `SQLX_OFFLINE=1`. Query changes usually require regenerating SQLx metadata with `cargo sqlx prepare --workspace` after applying migrations.
