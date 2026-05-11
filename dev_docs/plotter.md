# Plotter Service

## Purpose

The plotter is a Python FastAPI service that renders SVG charts from backend measurement data. It is separate from the backend so chart rendering and Matplotlib dependencies stay isolated.

## Technology

- FastAPI for HTTP routes.
- Uvicorn for serving.
- Requests for backend HTTP calls.
- Pydantic models for backend response validation.
- Matplotlib with SVG output.
- NumPy for polynomial regression on device/sensor plots.
- `cachetools.TTLCache` for in-memory SVG response caching.
- Pytest, HTTPX, respx, and Ruff for tests/linting.

## Files

- `plotter/main.py`: starts Uvicorn on `0.0.0.0:8000`.
- `plotter/app.py`: app definition, endpoints, plotting, theme, cache, and error translation.
- `plotter/backend_client.py`: typed backend client.
- `plotter/models.py`: Pydantic models matching backend JSON.
- `plotter/tests/`: unit tests.

## Configuration

- `BACKEND_URL`: backend base URL, default `http://localhost:65534`.
- `PLOT_CACHE_TTL`: cache TTL in seconds, default `60`.
- `PLOT_CACHE_MAXSIZE`: maximum cached plot count, default `128`.

## Backend Client

`BackendClient` owns a `requests.Session` and raises for non-success HTTP statuses.

Available methods cover:

- Devices: fetch all and by ID.
- Sensors: fetch all, by ID, and by device ID.
- Measurements: all, latest, latest all, count, date range, by device, by device/sensor, latest by device/sensor, and stats by device/sensor.

When backend JSON changes, update `plotter/models.py` and the relevant client methods.

## Caching

Each plot route uses `_cache_key(request)` to cache SVG bytes by path plus query string.

Use the existing pattern for new plot endpoints:

1. Compute `cache_key`.
2. Return cached SVG if present.
3. Fetch backend data.
4. Generate Matplotlib figure.
5. Render with `_fig_to_svg(fig)`, which closes the figure.
6. Store bytes in `_plot_cache`.
7. Return `Response(content=svg, media_type="image/svg+xml")`.

## Theme

The plotter uses the same Kanagawa-inspired dark palette as the frontend:

- `_apply_kanagawa(fig, ax)` styles figure background, axes, labels, ticks, spines, grid, and legend.
- `_kanagawa_color(index)` cycles through the palette.

Use these helpers for all charts so SVGs visually fit into the dashboard.

## Endpoints

See `api.md` for the endpoint table. Plot endpoints return SVG bytes, not JSON.

## Error Handling

`_handle_backend_error` translates backend failures:

- `requests.exceptions.ConnectionError` becomes `502 Backend unavailable`.
- `requests.exceptions.HTTPError` becomes the backend status code with error detail.
- Other exceptions are re-raised.

Routes return `404` when there is no data to plot.

## Chart Semantics

- `/plot/measurements`: groups all measurements by `device_name / sensor_name (unit)`.
- `/plot/devices/{device_id}/measurements`: groups one device's measurements by `sensor_name (unit)`.
- `/plot/devices/{device_id}/sensors/{sensor_id}/measurements`: filters optional `start`/`end` in the plotter after fetching the device/sensor series, then draws a line and a second-degree polynomial regression when there are at least two timestamps.
- `/plot/measurements/range`: delegates range filtering to the backend and groups results by `device_name / sensor_name (unit)`.
- `/plot/measurements/latest/all`: renders latest values as bars, one per device/sensor pair.

## Development Commands

From `plotter/`:

- `uv sync --frozen`: install dependencies from lockfile.
- `uv run main.py`: run service.
- `uv run pytest`: run tests.

From repo root:

- `uv tool run ruff check plotter/`: lint.
- `uv tool run ruff format --check plotter/`: check formatting.
