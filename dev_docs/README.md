# HEMRS Developer Documentation

This directory is the working documentation set for humans and agentic AI working on HEMRS.

HEMRS is a home environment monitoring system with a Rust backend, a Fresh/Deno frontend, a Python FastAPI plotter service, PostgreSQL storage, and Docker/Kubernetes deployment assets.

## Start Here

- `agent-guide.md`: Practical orientation for AI agents and new contributors.
- `architecture.md`: System shape, data flow, runtime services, and repository layout.
- `api.md`: Backend and plotter API contracts.
- `backend.md`: Rust backend internals, database access, caching, metrics, and tests.
- `frontend.md`: Fresh frontend routes, components, styling, and data access conventions.
- `plotter.md`: FastAPI plotter service, caching, plotting, and backend client.
- `database.md`: PostgreSQL schema, migrations, views, and SQLx notes.
- `development.md`: Local commands, tests, CI, Docker, and deployment notes.
- `conventions.md`: Cross-project coding conventions and change guidance.

## Quick Mental Model

- The backend is the source of truth and serves JSON APIs under `/api`, metrics at `/metrics`, OpenAPI JSON at `/openapi`, and health at `/status/ping`.
- Measurements are accepted synchronously over HTTP but inserted asynchronously through a Tokio channel and background task.
- PostgreSQL stores devices, sensors, and measurements. A materialized `device_sensors` view drives device-to-sensor relationships.
- The frontend is server-rendered Fresh/Preact and talks to the backend through helper functions in `frontend/lib`.
- The plotter is a separate FastAPI service that calls the backend, renders SVG charts with Matplotlib, caches SVG bytes, and is consumed by the frontend.
- The UI uses Tailwind v4 theme tokens in `frontend/assets/styles.css` with a Kanagawa-inspired dark palette.

## When Updating Docs

Update this documentation when changing any of these contracts:

- HTTP routes, payloads, response shapes, or status codes.
- Database schema, migrations, materialized views, or query semantics.
- Frontend route structure, shared component behavior, or environment variables.
- Plotter endpoints, cache behavior, chart semantics, or backend API usage.
- Development commands, CI jobs, Dockerfiles, or deployment manifests.
