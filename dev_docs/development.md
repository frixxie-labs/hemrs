# Development

## Prerequisites

- Rust and Cargo.
- Docker and Docker Compose.
- Deno for frontend development.
- `uv` for plotter development.
- PostgreSQL client tooling and `sqlx-cli` when updating migrations/query metadata.
- `just` for backend automation.

## Local Services

Start backend, plotter, and PostgreSQL:

```bash
docker compose up --build --wait
```

Start only the test PostgreSQL service:

```bash
docker compose -f docker-compose-test.yaml up --wait
```

Stop test PostgreSQL:

```bash
docker compose -f docker-compose-test.yaml down
```

## Backend Commands

From repo root:

```bash
cargo build --workspace
cargo test --workspace
```

Using `just`:

```bash
just check
just build
just test
just sqlx_prepare
```

`just test` builds, starts test PostgreSQL, runs `cargo test`, and stops PostgreSQL.

`just integration_test` starts the full Docker Compose stack, installs `sqlx-cli` and `hurl`, runs migrations, runs `backend/backend.hurl`, and tears the stack down. The referenced Hurl file is not present in the current tree, so verify this target before relying on it.

Run backend manually:

```bash
cargo run -p backend -- --host 0.0.0.0:65534 --db-url postgresql://postgres:admin@localhost:5432/postgres
```

## Frontend Commands

From `frontend/`:

```bash
deno task dev
deno task check
deno task test
deno task build
deno task start
```

Important frontend env:

```bash
HEMRS_URL=http://localhost:65534/
PLOTTER_URL=http://localhost:8000/
```

## Plotter Commands

From `plotter/`:

```bash
uv sync --frozen
uv run main.py
uv run pytest
```

From repo root:

```bash
uv tool run ruff check plotter/
uv tool run ruff format --check plotter/
```

Important plotter env:

```bash
BACKEND_URL=http://localhost:65534
PLOT_CACHE_TTL=60
PLOT_CACHE_MAXSIZE=128
```

## Load Testing

`load_test/locustfile.py` defines Locust tasks for:

- `GET api/devices`
- `GET api/sensors`

Latest-measurement tasks are present but commented out.

## Docker Images

Backend root `Dockerfile`:

- Uses `rust:latest` builder.
- Sets `SQLX_OFFLINE=1`.
- Copies backend and `.sqlx` metadata.
- Installs backend binary with `cargo install --path backend`.
- Runs on `rust:slim` with `CMD ["backend"]`.

Frontend `frontend/Dockerfile`:

- Uses `denoland/deno:latest`.
- Copies app.
- Runs `deno task build`.
- Exposes `8000`.
- Starts with `deno task start` through `CMD ["task", "start"]`.

Plotter `plotter/Dockerfile`:

- Uses `ghcr.io/astral-sh/uv:python3.14-bookworm-slim`.
- Installs dependencies from `pyproject.toml` and `uv.lock`.
- Exposes `8000`.
- Starts with `uv run main.py`.

## CI/CD

`.gitlab-ci.yml` stages:

- `lint`
- `build`
- `test`
- `deploy`

Plotter jobs:

- Ruff check and format check.
- `uv sync --frozen` build validation.
- `uv run pytest`.
- Buildah image build/push for tags.

Backend jobs:

- `cargo build --workspace --verbose` with `SQLX_OFFLINE=1`.
- `cargo test --workspace --verbose` with Postgres service and `SQLX_OFFLINE=1`.
- Buildah image build/push for tags.

Frontend jobs:

- Deno format and lint.
- `deno task check` and `deno task build`.
- `deno task test`.
- Buildah image build/push for tags.

Changelog job:

- Runs `git-cliff` on tags and publishes `CHANGELOG.md` as an artifact.

## Kubernetes Release Manifests

`release/` contains Kubernetes manifests:

- `deployment.yaml`: backend deployment with `replicas: 2`, image policy annotation comment, and `DATABASE_URL` from ConfigMap.
- `service.yaml` and `service_loadbalancer.yaml`: service exposure.
- `configmap.yaml`: `DATABASE_URL` value.
- `kustomization.yaml`: groups release resources under namespace `default`.

The manifests contain Kompose annotations and appear to originate from `docker-compose.yaml` conversion. Validate them before production changes.
