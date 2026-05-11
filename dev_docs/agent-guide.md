# Agent Guide

Use this file as the first stop before modifying HEMRS.

## Project Map

- `backend/`: Rust 2021 Axum API server and domain/data-access modules.
- `frontend/`: Fresh 2 / Deno / Preact frontend.
- `plotter/`: Python FastAPI service that renders SVG plots from backend data.
- `backend/migrations/`: SQLx migrations for PostgreSQL.
- `.sqlx/` and `backend/.sqlx/`: SQLx offline query metadata used by CI/builds.
- `sqls/`: Ad-hoc SQL query examples.
- `load_test/`: Locust load-test tasks.
- `release/`: Kubernetes manifests generated from Docker Compose via Kompose and maintained for deployment.
- `.gitlab-ci.yml`: CI pipeline for backend, frontend, plotter, deployments, and changelog.

## High-Value Entry Points

- Backend router: `backend/src/handlers/mod.rs`
- Backend startup: `backend/src/main.rs`
- Backend background tasks: `backend/src/background_tasks.rs`
- Backend domain/data modules: `backend/src/devices.rs`, `backend/src/sensors.rs`, `backend/src/measurements.rs`
- Frontend app setup: `frontend/main.ts`, `frontend/routes/_app.tsx`, `frontend/routes/_layout.tsx`
- Frontend backend clients: `frontend/lib/*.ts`
- Frontend reusable components: `frontend/components/*.tsx`
- Plotter app: `plotter/app.py`
- Plotter backend client: `plotter/backend_client.py`
- Plotter models: `plotter/models.py`

## Common Workflows

- Add a backend endpoint: update the handler module, route registration in `backend/src/handlers/mod.rs`, OpenAPI paths/components if relevant, domain query functions, tests, and frontend/plotter clients if they consume it.
- Change measurement response shape: update Rust `Measurement`, SQL queries, OpenAPI schema, frontend `Measurement` interface, plotter `Measurement` Pydantic model, and tests.
- Add a frontend page: add a route under `frontend/routes`, fetch data through `frontend/lib`, compose existing components where possible, and use existing Tailwind theme tokens.
- Add a chart: add/extend a `BackendClient` method, add a plotter route in `plotter/app.py`, use `_cache_key`, `_fig_to_svg`, `_apply_kanagawa`, and expose a frontend helper in `frontend/lib/plotter.ts` if the UI needs it.
- Change database schema: add a migration, update SQLx query code, refresh SQLx offline metadata, and update `dev_docs/database.md`.

## Things To Preserve

- Backend data access is implemented as inherent methods on domain structs rather than a repository trait layer.
- Backend handlers are thin: validate request data, call domain methods, map errors through `HandlerError`, and return Axum responses.
- Measurement ingestion should remain non-blocking at the HTTP boundary: handlers enqueue work through `Sender<NewMeasurement>`.
- Latest device/sensor measurements use the Moka cache keyed by `(device_id, sensor_id)`.
- Fresh pages fetch server-side in route handlers and pass typed data into page components.
- The plotter returns SVG bytes directly and caches by path plus query string.

## Verification Shortcuts

- Backend: `cargo test --workspace` from the repository root. SQLx tests require PostgreSQL and migrations.
- Backend local full path: `just test` starts `docker-compose-test.yaml`, builds, tests, and stops Postgres.
- Frontend: `deno task check` and `deno task test` from `frontend/`.
- Plotter: `uv run pytest` from `plotter/`; lint/format checks use `uv tool run ruff check plotter/` and `uv tool run ruff format --check plotter/` from repo root.
- Docker Compose: `docker compose up --build --wait` from repo root starts backend, plotter, and Postgres.

## Environment Variables

- Backend: `DATABASE_URL` or `--db-url`; default is `postgres://postgres:example@localhost:5432/postgres`.
- Frontend: `HEMRS_URL` should point at the backend base URL and include the trailing slash, for example `http://localhost:65534/`.
- Frontend plotter client: `PLOTTER_URL`, default `http://localhost:8000/`.
- Plotter: `BACKEND_URL`, default `http://localhost:65534`.
- Plotter cache: `PLOT_CACHE_TTL`, default `60`; `PLOT_CACHE_MAXSIZE`, default `128`.

## Known Sharp Edges

- `MeasurementStats` fields in Rust are private but serialized; preserve names unless updating all consumers.
- Several frontend helpers return `null` in catch paths despite non-null Promise return types. Be careful when relying on strict type guarantees.
- `HEMRS_URL` is interpolated directly with `api/...`; missing trailing slash creates invalid URLs.
- `device_sensors` is a materialized view; after direct DB writes, refresh it with `Device::refresh_device_sensors_view` or wait for the background refresh.
- SQLx offline mode is used in Docker and CI. Query changes usually require regenerating `.sqlx` metadata.
