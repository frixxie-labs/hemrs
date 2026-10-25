import os
from datetime import datetime
from typing import Annotated

from backend_client import BackendClient
from cachetools import TTLCache
from chart_data import comparison_page, filter_measurements, group_series
from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import Response
from rendering import (
    render_comparison_history,
    render_latest_comparison,
    render_sensor_history,
    render_time_series,
)
from requests.exceptions import ConnectionError, HTTPError, Timeout

BACKEND_URL = os.environ.get("BACKEND_URL", "http://localhost:65534")
CACHE_TTL = int(os.environ.get("PLOT_CACHE_TTL", "60"))
CACHE_MAXSIZE = int(os.environ.get("PLOT_CACHE_MAXSIZE", "128"))
SVG_MEDIA_TYPE = "image/svg+xml"

app = FastAPI(
    title="hemrs plotter",
    version="0.1.0",
    docs_url=None,
    redoc_url=None,
    openapi_url=None,
)
client = BackendClient(base_url=BACKEND_URL)
_plot_cache: TTLCache[str, bytes] = TTLCache(maxsize=CACHE_MAXSIZE, ttl=CACHE_TTL)


def _cache_key(request: Request) -> str:
    url = request.url
    return f"{url.path}?{url.query}" if url.query else url.path


def _handle_backend_error(exc: Exception):
    if isinstance(exc, Timeout):
        raise HTTPException(status_code=504, detail="Backend timed out") from exc
    if isinstance(exc, ConnectionError):
        raise HTTPException(status_code=502, detail="Backend unavailable") from exc
    if isinstance(exc, HTTPError):
        raise HTTPException(
            status_code=exc.response.status_code, detail=str(exc)
        ) from exc
    raise exc


def _svg_response(key: str, svg: bytes) -> Response:
    _plot_cache[key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get("/status/health")
def health():
    return {"status": "healthy"}


@app.get("/status/ping")
def ping():
    return {"ping": "pong"}


@app.get("/plot/measurements", response_class=Response)
def plot_all_measurements(request: Request):
    key = _cache_key(request)
    if key in _plot_cache:
        return Response(content=_plot_cache[key], media_type=SVG_MEDIA_TYPE)
    try:
        measurements = client.fetch_all_measurements()
    except (ConnectionError, HTTPError, Timeout) as exc:
        _handle_backend_error(exc)
    if not measurements:
        raise HTTPException(status_code=404, detail="No measurements found")
    return _svg_response(
        key,
        render_time_series(
            group_series(measurements), "All Measurements", include_device=True
        ),
    )


@app.get("/plot/devices/{device_id}/measurements", response_class=Response)
def plot_measurements_by_device(device_id: int, request: Request):
    key = _cache_key(request)
    if key in _plot_cache:
        return Response(content=_plot_cache[key], media_type=SVG_MEDIA_TYPE)
    try:
        measurements = client.fetch_measurements_by_device_id(device_id)
    except (ConnectionError, HTTPError, Timeout) as exc:
        _handle_backend_error(exc)
    if not measurements:
        raise HTTPException(
            status_code=404, detail=f"No measurements found for device {device_id}"
        )
    return _svg_response(
        key,
        render_time_series(
            group_series(measurements),
            f"Measurements — {measurements[0].device_name}",
            include_device=False,
        ),
    )


@app.get(
    "/plot/devices/{device_id}/sensors/{sensor_id}/measurements",
    response_class=Response,
)
def plot_measurements_by_device_and_sensor(
    device_id: int,
    sensor_id: int,
    request: Request,
    start: Annotated[datetime | None, Query()] = None,
    end: Annotated[datetime | None, Query()] = None,
):
    key = _cache_key(request)
    if key in _plot_cache:
        return Response(content=_plot_cache[key], media_type=SVG_MEDIA_TYPE)
    try:
        measurements = client.fetch_measurements_by_device_and_sensor(
            device_id, sensor_id
        )
    except (ConnectionError, HTTPError, Timeout) as exc:
        _handle_backend_error(exc)
    measurements = filter_measurements(measurements, start, end)
    if not measurements:
        raise HTTPException(
            status_code=404,
            detail=f"No measurements for device {device_id} / sensor {sensor_id}",
        )
    return _svg_response(key, render_sensor_history(measurements))


@app.get("/plot/measurements/range", response_class=Response)
def plot_measurements_by_range(
    request: Request,
    start: Annotated[datetime, Query()],
    end: Annotated[datetime | None, Query()] = None,
    sensor: Annotated[str | None, Query()] = None,
    page: Annotated[int, Query(ge=1)] = 1,
):
    key = _cache_key(request)
    if key in _plot_cache:
        return Response(content=_plot_cache[key], media_type=SVG_MEDIA_TYPE)
    if end is not None and end <= start:
        raise HTTPException(status_code=422, detail="End must be after start")
    try:
        measurements = client.fetch_measurements_by_date_range(start, end)
        latest = client.fetch_all_latest_measurements()
    except (ConnectionError, HTTPError, Timeout) as exc:
        _handle_backend_error(exc)
    if not measurements:
        raise HTTPException(
            status_code=404, detail="No measurements found in the given range"
        )
    selected = comparison_page(latest or measurements, sensor, page)
    if not selected:
        raise HTTPException(status_code=404, detail="No measurements for this sensor")
    return _svg_response(
        key,
        render_comparison_history(
            selected,
            group_series(measurements),
            start,
            end or datetime.now(start.tzinfo),
        ),
    )


@app.get("/plot/measurements/latest/all", response_class=Response)
def plot_all_latest_measurements(
    request: Request,
    sensor: Annotated[str | None, Query()] = None,
    page: Annotated[int, Query(ge=1)] = 1,
):
    key = _cache_key(request)
    if key in _plot_cache:
        return Response(content=_plot_cache[key], media_type=SVG_MEDIA_TYPE)
    try:
        measurements = client.fetch_all_latest_measurements()
    except (ConnectionError, HTTPError, Timeout) as exc:
        _handle_backend_error(exc)
    if not measurements:
        raise HTTPException(status_code=404, detail="No measurements found")
    selected = comparison_page(measurements, sensor, page)
    if not selected:
        raise HTTPException(status_code=404, detail="No measurements for this sensor")
    return _svg_response(key, render_latest_comparison(selected))
