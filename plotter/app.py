import os
from collections.abc import Callable
from datetime import datetime
from typing import Annotated, TypeVar

from backend_client import BackendClient
from cachetools import TTLCache
from chart_data import comparison_page, filter_measurements, group_series
from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import Response
from models import Measurement
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
BACKEND_ERRORS = (ConnectionError, HTTPError, Timeout)

T = TypeVar("T")

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


def _backend_http_error(exc: ConnectionError | HTTPError | Timeout) -> HTTPException:
    if isinstance(exc, Timeout):
        return HTTPException(status_code=504, detail="Backend timed out")
    if isinstance(exc, ConnectionError):
        return HTTPException(status_code=502, detail="Backend unavailable")
    return HTTPException(status_code=exc.response.status_code, detail=str(exc))


def _cached_svg(
    request: Request, fetch: Callable[[], T], render: Callable[[T], bytes]
) -> Response:
    """Serve a cached SVG, or fetch backend data, render it, and cache the result."""
    key = _cache_key(request)
    if key not in _plot_cache:
        try:
            data = fetch()
        except BACKEND_ERRORS as exc:
            raise _backend_http_error(exc) from exc
        _plot_cache[key] = render(data)
    return Response(content=_plot_cache[key], media_type=SVG_MEDIA_TYPE)


def _not_found(detail: str) -> HTTPException:
    return HTTPException(status_code=404, detail=detail)


@app.get("/status/health")
def health():
    return {"status": "healthy"}


@app.get("/status/ping")
def ping():
    return {"ping": "pong"}


@app.get("/plot/measurements", response_class=Response)
def plot_all_measurements(request: Request):
    def render(measurements: list[Measurement]) -> bytes:
        if not measurements:
            raise _not_found("No measurements found")
        return render_time_series(
            group_series(measurements), "All Measurements", include_device=True
        )

    return _cached_svg(request, client.fetch_all_measurements, render)


@app.get("/plot/devices/{device_id}/measurements", response_class=Response)
def plot_measurements_by_device(device_id: int, request: Request):
    def render(measurements: list[Measurement]) -> bytes:
        if not measurements:
            raise _not_found(f"No measurements found for device {device_id}")
        return render_time_series(
            group_series(measurements),
            f"Measurements — {measurements[0].device_name}",
            include_device=False,
        )

    return _cached_svg(
        request, lambda: client.fetch_measurements_by_device_id(device_id), render
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
    def render(measurements: list[Measurement]) -> bytes:
        selected = filter_measurements(measurements, start, end)
        if not selected:
            raise _not_found(
                f"No measurements for device {device_id} / sensor {sensor_id}"
            )
        return render_sensor_history(selected)

    return _cached_svg(
        request,
        lambda: client.fetch_measurements_by_device_and_sensor(device_id, sensor_id),
        render,
    )


@app.get("/plot/measurements/range", response_class=Response)
def plot_measurements_by_range(
    request: Request,
    start: Annotated[datetime, Query()],
    end: Annotated[datetime | None, Query()] = None,
    sensor: Annotated[str | None, Query()] = None,
    page: Annotated[int, Query(ge=1)] = 1,
):
    if end is not None and end <= start:
        raise HTTPException(status_code=422, detail="End must be after start")

    def fetch() -> tuple[list[Measurement], list[Measurement]]:
        return (
            client.fetch_measurements_by_date_range(start, end),
            client.fetch_all_latest_measurements(),
        )

    def render(data: tuple[list[Measurement], list[Measurement]]) -> bytes:
        measurements, latest = data
        if not measurements:
            raise _not_found("No measurements found in the given range")
        selected = comparison_page(latest or measurements, sensor, page)
        if not selected:
            raise _not_found("No measurements for this sensor")
        return render_comparison_history(
            selected,
            group_series(measurements),
            start,
            end or datetime.now(start.tzinfo),
        )

    return _cached_svg(request, fetch, render)


@app.get("/plot/measurements/latest/all", response_class=Response)
def plot_all_latest_measurements(
    request: Request,
    sensor: Annotated[str | None, Query()] = None,
    page: Annotated[int, Query(ge=1)] = 1,
):
    def render(measurements: list[Measurement]) -> bytes:
        if not measurements:
            raise _not_found("No measurements found")
        selected = comparison_page(measurements, sensor, page)
        if not selected:
            raise _not_found("No measurements for this sensor")
        return render_latest_comparison(selected)

    return _cached_svg(request, client.fetch_all_latest_measurements, render)
