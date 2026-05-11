# Architecture

## System Overview

HEMRS monitors environmental measurements from distributed devices. The system is split into independent services:

- Rust backend: REST API, PostgreSQL access, background measurement insertion, metrics, and OpenAPI JSON.
- PostgreSQL: durable storage for devices, sensors, measurements, and the materialized device/sensor relationship view.
- Fresh frontend: server-rendered dashboard and CRUD forms for devices, sensors, and measurements.
- Python plotter: FastAPI service that calls the backend and returns SVG charts.
- Deployment assets: Dockerfiles, Docker Compose, GitLab CI, and Kubernetes manifests.

## Runtime Data Flow

1. A client or sensor posts one measurement or a batch to `/api/measurements` or `/`.
2. The backend handler deserializes the body as `NewMeasurements`.
3. The handler sends each `NewMeasurement` to a Tokio MPSC channel and returns HTTP `201`.
4. `handle_insert_measurement_bg_thread` receives measurements from the channel.
5. The background task resolves device and sensor metadata, updates the Moka latest-measurement cache, and inserts the row into PostgreSQL.
6. Query endpoints read from PostgreSQL, with latest device/sensor reads checking the cache first.
7. The metrics background task periodically emits gauges for recent measurements and counters for pool/cache size.
8. The plotter fetches measurements from the backend, renders SVG charts, and caches them by request path/query.
9. The frontend renders pages by calling the backend and plotter from server-side route handlers.

## Backend Layers

- `backend/src/main.rs`: parses CLI/env config, initializes tracing, metrics, Postgres pool, cache, background tasks, router, and HTTP listener.
- `backend/src/handlers/mod.rs`: owns Axum routing, middleware, metrics endpoint, OpenAPI document registration, and route state wiring.
- `backend/src/handlers/*.rs`: HTTP handlers for devices, sensors, measurements, ping, and errors.
- `backend/src/devices.rs`, `backend/src/sensors.rs`, `backend/src/measurements.rs`: serializable models plus SQLx data-access methods.
- `backend/src/background_tasks.rs`: metrics loop, asynchronous insert worker, and materialized-view refresh loop.

## Frontend Layers

- `frontend/main.ts`: creates the Fresh app, installs static files and logging middleware, and enables file-system routes.
- `frontend/routes/_app.tsx`: HTML shell and global stylesheet link.
- `frontend/routes/_layout.tsx`: dashboard header, navigation, and page container.
- `frontend/routes/**`: server-side route handlers and page components.
- `frontend/lib/*.ts`: backend and plotter client helpers.
- `frontend/components/*.tsx`: reusable presentational UI components.
- `frontend/assets/styles.css`: Tailwind v4 import and custom theme tokens.

## Plotter Layers

- `plotter/main.py`: Uvicorn entry point.
- `plotter/app.py`: FastAPI app, route handlers, Matplotlib rendering, theme helpers, cache, and error translation.
- `plotter/backend_client.py`: typed HTTP client for backend endpoints.
- `plotter/models.py`: Pydantic models mirroring backend JSON responses.

## Repository Layout

```text
.
├── backend/              Rust Axum backend
├── frontend/             Fresh/Deno frontend
├── plotter/              FastAPI SVG plotting service
├── load_test/            Locust load-test tasks
├── sqls/                 Ad-hoc SQL query examples
├── release/              Kubernetes manifests
├── dev_docs/             Developer and AI-agent documentation
├── docker-compose.yaml   Local multi-service runtime
├── docker-compose-test.yaml
├── Dockerfile            Backend container image
├── Cargo.toml            Rust workspace manifest
├── justfile              Backend-oriented automation
└── .gitlab-ci.yml        CI/CD pipeline
```

## Service Ports

- Backend: `65534`
- Frontend: `8000` in its Dockerfile/runtime command. When using `vite`, the dev server uses Vite defaults unless configured elsewhere.
- Plotter: `8000`
- PostgreSQL: `5432`

Avoid running frontend and plotter on the same host port at the same time unless one is moved.
