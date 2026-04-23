import io
import os
from datetime import datetime

import matplotlib
import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
from cachetools import TTLCache
from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import Response
from requests.exceptions import ConnectionError, HTTPError

from backend_client import BackendClient

matplotlib.use("svg")

BACKEND_URL = os.environ.get("BACKEND_URL", "http://localhost:65534")
CACHE_TTL = int(os.environ.get("PLOT_CACHE_TTL", "60"))
CACHE_MAXSIZE = int(os.environ.get("PLOT_CACHE_MAXSIZE", "128"))

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
    """Build a cache key from the request path and query string."""
    url = request.url
    return f"{url.path}?{url.query}" if url.query else url.path


SVG_MEDIA_TYPE = "image/svg+xml"


@app.get("/status/health")
def health():
    return {"status": "healthy"}


@app.get("/status/ping")
def ping():
    return {"ping": "pong"}


def _fig_to_svg(fig: plt.Figure) -> bytes:
    """Render a matplotlib figure to SVG bytes and close it."""
    buf = io.BytesIO()
    fig.savefig(buf, format="svg", bbox_inches="tight")
    plt.close(fig)
    buf.seek(0)
    return buf.read()


def _handle_backend_error(exc: Exception):
    if isinstance(exc, ConnectionError):
        raise HTTPException(status_code=502, detail="Backend unavailable")
    if isinstance(exc, HTTPError):
        raise HTTPException(status_code=exc.response.status_code, detail=str(exc))
    raise exc


@app.get("/plot/measurements", response_class=Response)
def plot_all_measurements(request: Request):
    """Plot all measurements as a time-series SVG, grouped by sensor."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    try:
        measurements = client.fetch_all_measurements()
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if not measurements:
        raise HTTPException(status_code=404, detail="No measurements found")

    groups: dict[str, list] = {}
    for m in measurements:
        key = f"{m.device_name} / {m.sensor_name} ({m.unit})"
        groups.setdefault(key, []).append(m)

    fig, ax = plt.subplots(figsize=(12, 6))
    for label, items in groups.items():
        items.sort(key=lambda m: m.timestamp)
        timestamps = [m.timestamp for m in items]
        values = [m.value for m in items]
        ax.plot(timestamps, values, label=label, marker=".", markersize=3)

    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    ax.set_title("All Measurements")
    ax.legend(fontsize="small", loc="best")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    ax.grid(True, alpha=0.3)

    svg = _fig_to_svg(fig)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get("/plot/devices/{device_id}/measurements", response_class=Response)
def plot_measurements_by_device(device_id: int, request: Request):
    """Plot all measurements for a device as a time-series SVG."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    try:
        measurements = client.fetch_measurements_by_device_id(device_id)
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if not measurements:
        raise HTTPException(
            status_code=404,
            detail=f"No measurements found for device {device_id}",
        )

    groups: dict[str, list] = {}
    for m in measurements:
        key = f"{m.sensor_name} ({m.unit})"
        groups.setdefault(key, []).append(m)

    device_label = measurements[0].device_name

    fig, ax = plt.subplots(figsize=(12, 6))
    for label, items in groups.items():
        items.sort(key=lambda m: m.timestamp)
        timestamps = [m.timestamp for m in items]
        values = [m.value for m in items]
        ax.plot(timestamps, values, label=label, marker=".", markersize=3)

    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    ax.set_title(f"Measurements — {device_label}")
    ax.legend(fontsize="small", loc="best")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    ax.grid(True, alpha=0.3)

    svg = _fig_to_svg(fig)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get(
    "/plot/devices/{device_id}/sensors/{sensor_id}/measurements",
    response_class=Response,
)
def plot_measurements_by_device_and_sensor(
    device_id: int,
    sensor_id: int,
    request: Request,
    start: datetime | None = Query(default=None),
    end: datetime | None = Query(default=None),
):
    """Plot measurements for a specific device/sensor pair as a time-series SVG."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    try:
        measurements = client.fetch_measurements_by_device_and_sensor(
            device_id, sensor_id
        )
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if measurements and start is not None:
        measurements = [m for m in measurements if m.timestamp >= start]
    if measurements and end is not None:
        measurements = [m for m in measurements if m.timestamp <= end]

    if not measurements:
        raise HTTPException(
            status_code=404,
            detail=f"No measurements for device {device_id} / sensor {sensor_id}",
        )

    measurements.sort(key=lambda m: m.timestamp)
    timestamps = [m.timestamp for m in measurements]
    values = [m.value for m in measurements]
    label = f"{measurements[0].sensor_name} ({measurements[0].unit})"

    fig, ax = plt.subplots(figsize=(12, 6))
    ax.plot(timestamps, values, marker=".", markersize=3)

    # Dotted regression line
    if len(timestamps) >= 2:
        ts_numeric = np.array([t.timestamp() for t in timestamps])
        vals = np.array(values)
        coeffs = np.polyfit(ts_numeric, vals, 2)
        reg_values = np.polyval(coeffs, ts_numeric)
        ax.plot(
            timestamps,
            reg_values,
            linestyle=":",
            color="red",
            label="Polynomial regression",
        )
        ax.legend(fontsize="small", loc="best")

    ax.set_xlabel("Time")
    ax.set_ylabel(f"{measurements[0].sensor_name} ({measurements[0].unit})")
    ax.set_title(f"{measurements[0].device_name} — {label}")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    ax.grid(True, alpha=0.3)

    svg = _fig_to_svg(fig)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get("/plot/measurements/range", response_class=Response)
def plot_measurements_by_range(
    request: Request,
    start: datetime = Query(...),
    end: datetime | None = Query(default=None),
):
    """Plot measurements within a date range as a time-series SVG."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    try:
        measurements = client.fetch_measurements_by_date_range(start, end)
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if not measurements:
        raise HTTPException(
            status_code=404,
            detail="No measurements found in the given range",
        )

    groups: dict[str, list] = {}
    for m in measurements:
        key = f"{m.device_name} / {m.sensor_name} ({m.unit})"
        groups.setdefault(key, []).append(m)

    fig, ax = plt.subplots(figsize=(12, 6))
    for label, items in groups.items():
        items.sort(key=lambda m: m.timestamp)
        timestamps = [m.timestamp for m in items]
        values = [m.value for m in items]
        ax.plot(timestamps, values, label=label, marker=".", markersize=3)

    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    title = f"Measurements from {start.strftime('%Y-%m-%d %H:%M')}"
    if end:
        title += f" to {end.strftime('%Y-%m-%d %H:%M')}"
    ax.set_title(title)
    ax.legend(fontsize="small", loc="best")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    ax.grid(True, alpha=0.3)

    svg = _fig_to_svg(fig)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get("/plot/measurements/latest/all", response_class=Response)
def plot_all_latest_measurements(request: Request):
    """Bar chart of the latest measurement per device/sensor pair as SVG."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    try:
        measurements = client.fetch_all_latest_measurements()
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if not measurements:
        raise HTTPException(status_code=404, detail="No measurements found")

    labels = [f"{m.device_name}\n{m.sensor_name}\n({m.unit})" for m in measurements]
    values = [m.value for m in measurements]

    fig, ax = plt.subplots(figsize=(max(6, len(labels) * 1.2), 6))
    bars = ax.bar(range(len(labels)), values)
    ax.set_xticks(range(len(labels)))
    ax.set_xticklabels(labels, fontsize="small")
    ax.set_ylabel("Value")
    ax.set_title("Latest Measurements (per device/sensor)")
    ax.grid(True, axis="y", alpha=0.3)

    for bar, val in zip(bars, values):
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            bar.get_height(),
            f"{val:.2f}",
            ha="center",
            va="bottom",
            fontsize="small",
        )

    fig.tight_layout()

    svg = _fig_to_svg(fig)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)
