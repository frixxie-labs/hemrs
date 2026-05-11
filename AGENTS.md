# AGENTS.md

## Orientation
- Read `dev_docs/agent-guide.md` first for detailed repo context; keep this file compact.
- Service boundaries: `backend/` is Rust/Axum + SQLx/Postgres, `frontend/` is Fresh/Deno/Preact, `plotter/` is FastAPI/Matplotlib SVG rendering.
- Backend entrypoints: `backend/src/main.rs` starts runtime/tasks; `backend/src/handlers/mod.rs` wires all routes/OpenAPI/middleware; domain SQL lives in `backend/src/{devices,sensors,measurements}.rs`.
- Frontend entrypoints: `frontend/main.ts`, `frontend/routes/_app.tsx`, `frontend/routes/_layout.tsx`; backend calls should stay in `frontend/lib/*.ts`.
- Plotter entrypoints: `plotter/app.py`, `plotter/backend_client.py`, `plotter/models.py`.

## Commands
- Backend focused build/test from repo root: `cargo build --workspace`, `cargo test --workspace`.
- Backend full local test path: `just test` starts `docker-compose-test.yaml`, builds, tests, then stops Postgres.
- Regenerate SQLx offline metadata after schema/query changes: `just sqlx_prepare`.
- Avoid relying on `just integration_test` without checking it first; it references `backend/backend.hurl`, which is absent in this tree.
- Frontend from `frontend/`: `deno task check`, `deno task test`, `deno task build`, `deno task dev`.
- Plotter from `plotter/`: `uv sync --frozen`, `uv run pytest`, `uv run main.py`.
- Plotter lint/format checks from repo root: `uv tool run ruff check plotter/`, `uv tool run ruff format --check plotter/`.
- Full local stack from repo root: `docker compose up --build --wait`; backend uses `65534`, plotter uses `8000`, Postgres uses `5432`.

## Environment And Tooling Gotchas
- Backend/CI/Docker use `SQLX_OFFLINE=1`; SQL query changes usually need refreshed `.sqlx` metadata.
- Backend `DATABASE_URL` default differs from Compose; Compose uses `postgresql://postgres:admin@db:5432/postgres`.
- Frontend `HEMRS_URL` is string-concatenated with `api/...`; include a trailing slash, for example `http://localhost:65534/`.
- Frontend plot helper uses `PLOTTER_URL`, default `http://localhost:8000/`.
- Plotter uses `BACKEND_URL`, default `http://localhost:65534`; cache env vars are `PLOT_CACHE_TTL` and `PLOT_CACHE_MAXSIZE`.
- Frontend and plotter Dockerfiles both expose port `8000`; do not run both on the same host port without changing one.

## Change Coupling
- Backend route changes require updating `backend/src/handlers/mod.rs`; public API/schema changes should also update `utoipa` registrations and `dev_docs/api.md`.
- Measurement response shape changes touch Rust `Measurement` SQL queries, frontend `Measurement` interface, plotter Pydantic `Measurement`, and tests.
- Database changes require a new file in `backend/migrations/`, SQLx metadata refresh, and `dev_docs/database.md` updates.
- New plotter charts should use `_cache_key`, `_fig_to_svg`, `_apply_kanagawa`, and `_kanagawa_color`; expose a `frontend/lib/plotter.ts` helper if the UI consumes them.

## Repo-Specific Constraints
- Measurement ingestion is intentionally async: handlers enqueue `NewMeasurement` through a Tokio MPSC channel; the background task inserts into Postgres and updates cache.
- Latest device/sensor reads use a Moka cache keyed by `(device_id, sensor_id)`.
- `device_sensors` is a materialized view; direct measurement inserts can leave device/sensor relationships stale until `REFRESH MATERIALIZED VIEW device_sensors` runs.
- Fresh pages fetch server-side in route handlers and pass data into `define.page`; presentational components should not construct backend URLs.
- UI uses Tailwind v4 theme tokens in `frontend/assets/styles.css`; preserve the Kanagawa dark palette instead of hard-coded one-off colors.
- Several frontend helpers catch errors and return `null` despite non-null Promise types; do not assume strict non-null behavior without checking call sites.
