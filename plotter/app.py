import io
import os
from datetime import datetime
from textwrap import shorten, wrap
from typing import Annotated

import matplotlib
import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
from backend_client import BackendClient
from cachetools import TTLCache
from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import Response
from models import Measurement
from requests.exceptions import ConnectionError, HTTPError

matplotlib.use("svg")

# -- Kanagawa theme (inspired by "The Great Wave off Kanagawa") --
_KANAGAWA_BG = "#1F1F28"  # sumiInk0 – dark indigo background
_KANAGAWA_FG = "#DCD7BA"  # fujiWhite – warm off-white foreground
_KANAGAWA_GRID = "#54546D"  # sumiInk4
_KANAGAWA_PALETTE = [
    "#7E9CD8",  # crystalBlue
    "#E46876",  # waveRed
    "#98BB6C",  # springGreen
    "#E6C384",  # carpYellow
    "#957FB8",  # oniViolet
    "#7FB4CA",  # springBlue
    "#D27E99",  # sakuraPink
    "#FF9E3B",  # surimiOrange
    "#7AA89F",  # waveAqua2
    "#FFA066",  # peachRed
]


def _apply_kanagawa(fig: plt.Figure, ax: plt.Axes):
    """Apply Kanagawa colour theme to a figure and axes."""
    fig.patch.set_facecolor(_KANAGAWA_BG)
    ax.set_facecolor(_KANAGAWA_BG)
    ax.title.set_color(_KANAGAWA_FG)
    ax.xaxis.label.set_color(_KANAGAWA_FG)
    ax.yaxis.label.set_color(_KANAGAWA_FG)
    ax.tick_params(colors=_KANAGAWA_FG)
    for spine in ax.spines.values():
        spine.set_color(_KANAGAWA_GRID)
    ax.grid(True, color=_KANAGAWA_GRID, alpha=0.4)
    legend = ax.get_legend()
    if legend:
        legend.get_frame().set_facecolor(_KANAGAWA_BG)
        legend.get_frame().set_edgecolor(_KANAGAWA_GRID)
        for text in legend.get_texts():
            text.set_color(_KANAGAWA_FG)


def _kanagawa_color(index: int) -> str:
    return _KANAGAWA_PALETTE[index % len(_KANAGAWA_PALETTE)]


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


def _fig_to_svg(fig: plt.Figure, *, tight: bool = True) -> bytes:
    """Render a matplotlib figure to SVG bytes and close it."""
    buf = io.BytesIO()
    fig.savefig(buf, format="svg", bbox_inches="tight" if tight else None)
    plt.close(fig)
    buf.seek(0)
    return buf.read()


def _handle_backend_error(exc: Exception):
    if isinstance(exc, ConnectionError):
        raise HTTPException(status_code=502, detail="Backend unavailable")
    if isinstance(exc, HTTPError):
        raise HTTPException(status_code=exc.response.status_code, detail=str(exc))
    raise exc


# Keep the overview legible at card width, regardless of sensor/device count.
OVERVIEW_PAGE_SIZE = 6


def _comparison_page(
    measurements: list[Measurement], sensor: str | None, page: int
) -> list[Measurement]:
    """Choose one sensor/unit and a stable, bounded page of devices."""
    groups: dict[tuple[str, str], dict[tuple[str, str], Measurement]] = {}
    for m in measurements:
        if sensor is None or m.sensor_name == sensor:
            devices = groups.setdefault((m.sensor_name, m.unit), {})
            key = (m.device_name, m.device_location)
            if key not in devices or devices[key].timestamp < m.timestamp:
                devices[key] = m
    if not groups:
        raise HTTPException(status_code=404, detail="No measurements for this sensor")
    # Default to the sensor shared by the most devices, with a stable tie-break.
    key = min(groups, key=lambda key: (-len(groups[key]), key))
    devices = groups[key]
    last_page = (len(devices) - 1) // OVERVIEW_PAGE_SIZE + 1
    offset = (min(page, last_page) - 1) * OVERVIEW_PAGE_SIZE
    return [
        devices[key] for key in sorted(devices)[offset : offset + OVERVIEW_PAGE_SIZE]
    ]


def _series_key(m: Measurement) -> tuple[str, str, str, str]:
    return m.device_name, m.device_location, m.sensor_name, m.unit


def _device_label(m: Measurement) -> str:
    label = (
        f"{m.device_name} / {m.device_location}" if m.device_location else m.device_name
    )
    return "\n".join(wrap(label, width=28, max_lines=2, placeholder="…"))


def _comparison_axes(measurement: Measurement) -> tuple[plt.Figure, plt.Axes]:
    fig, ax = plt.subplots(figsize=(8, 4.8))
    _apply_kanagawa(fig, ax)
    ax.set_title(
        shorten(measurement.sensor_name, width=52, placeholder="…"), fontsize=12, pad=14
    )
    ax.tick_params(labelsize=10)
    ax.spines[["top", "right"]].set_visible(False)
    ax.set_axisbelow(True)
    return fig, ax


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
    for i, (label, items) in enumerate(groups.items()):
        items.sort(key=lambda m: m.timestamp)
        timestamps = [m.timestamp for m in items]
        values = [m.value for m in items]
        ax.plot(
            timestamps,
            values,
            label=label,
            marker=".",
            markersize=3,
            color=_kanagawa_color(i),
        )

    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    ax.set_title("All Measurements")
    ax.legend(fontsize="small", loc="best")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    _apply_kanagawa(fig, ax)

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
    for i, (label, items) in enumerate(groups.items()):
        items.sort(key=lambda m: m.timestamp)
        timestamps = [m.timestamp for m in items]
        values = [m.value for m in items]
        ax.plot(
            timestamps,
            values,
            label=label,
            marker=".",
            markersize=3,
            color=_kanagawa_color(i),
        )

    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    ax.set_title(f"Measurements — {device_label}")
    ax.legend(fontsize="small", loc="best")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    _apply_kanagawa(fig, ax)

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
    start: Annotated[datetime | None, Query()] = None,
    end: Annotated[datetime | None, Query()] = None,
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
    ax.plot(timestamps, values, marker=".", markersize=3, color=_kanagawa_color(0))

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
            color=_kanagawa_color(1),
            label="Polynomial regression",
        )
        ax.legend(fontsize="small", loc="best")

    ax.set_xlabel("Time")
    ax.set_ylabel(f"{measurements[0].sensor_name} ({measurements[0].unit})")
    ax.set_title(f"{measurements[0].device_name} — {label}")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    _apply_kanagawa(fig, ax)

    svg = _fig_to_svg(fig)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get("/plot/measurements/range", response_class=Response)
def plot_measurements_by_range(
    request: Request,
    start: Annotated[datetime, Query()],
    end: Annotated[datetime | None, Query()] = None,
    sensor: Annotated[str | None, Query()] = None,
    page: Annotated[int, Query(ge=1)] = 1,
):
    """Compare a page of devices for one sensor within a date range."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    if end is not None and end <= start:
        raise HTTPException(status_code=422, detail="End must be after start")

    try:
        measurements = client.fetch_measurements_by_date_range(start, end)
        # Use the same device selection as the latest chart, even when a device
        # has no history in this window. Fall back for historical-only data.
        latest = client.fetch_all_latest_measurements()
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if not measurements:
        raise HTTPException(
            status_code=404,
            detail="No measurements found in the given range",
        )

    selected = _comparison_page(latest or measurements, sensor, page)
    groups = {_series_key(m): [] for m in selected}
    for m in measurements:
        if _series_key(m) in groups:
            groups[_series_key(m)].append(m)

    fig, ax = _comparison_axes(selected[0])
    fig.subplots_adjust(left=0.13, right=0.96, bottom=0.35, top=0.9)
    for i, m in enumerate(selected):
        items = sorted(groups[_series_key(m)], key=lambda item: item.timestamp)
        ax.plot(
            [item.timestamp for item in items],
            [item.value for item in items],
            label=_device_label(m),
            marker="." if len(items) < 50 else None,
            markersize=4,
            linewidth=1.8,
            color=_kanagawa_color(i),
        )

    if not any(groups.values()):
        ax.text(
            0.5,
            0.5,
            "No readings in this time window",
            transform=ax.transAxes,
            ha="center",
            color=_KANAGAWA_FG,
        )
        ax.set_yticks([])

    ax.set_ylabel(selected[0].unit)
    ax.set_xlabel("Time (UTC)", fontsize=10)
    ax.set_xlim(start, end or datetime.now(start.tzinfo))
    locator = mdates.AutoDateLocator(minticks=3, maxticks=5)
    ax.xaxis.set_major_locator(locator)
    ax.xaxis.set_major_formatter(mdates.ConciseDateFormatter(locator))
    fig.legend(
        *ax.get_legend_handles_labels(),
        loc="lower center",
        bbox_to_anchor=(0.5, 0.005),
        ncol=2,
        fontsize=9,
        frameon=False,
        labelcolor=_KANAGAWA_FG,
    )

    svg = _fig_to_svg(fig, tight=False)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)


@app.get("/plot/measurements/latest/all", response_class=Response)
def plot_all_latest_measurements(
    request: Request,
    sensor: Annotated[str | None, Query()] = None,
    page: Annotated[int, Query(ge=1)] = 1,
):
    """Horizontal comparison of up to six devices measuring the same sensor/unit."""
    cache_key = _cache_key(request)
    if cache_key in _plot_cache:
        return Response(content=_plot_cache[cache_key], media_type=SVG_MEDIA_TYPE)

    try:
        measurements = client.fetch_all_latest_measurements()
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

    if not measurements:
        raise HTTPException(status_code=404, detail="No measurements found")

    selected = _comparison_page(measurements, sensor, page)
    values = [m.value for m in selected]
    fig, ax = _comparison_axes(selected[0])
    fig.subplots_adjust(left=0.32, right=0.83, bottom=0.16, top=0.9)
    ax.barh(
        range(len(selected)),
        values,
        height=0.5,
        color=[_kanagawa_color(i) for i in range(len(selected))],
    )
    ax.set_yticks(
        range(len(selected)), [_device_label(m) for m in selected], fontsize=9
    )
    ax.set_ylim(OVERVIEW_PAGE_SIZE - 0.5, -0.5)
    ax.set_xlabel(selected[0].unit)
    ax.yaxis.grid(False)
    ax.axvline(0, color=_KANAGAWA_GRID, linewidth=0.8)
    ax.margins(x=0.12)
    ax.xaxis.set_major_locator(matplotlib.ticker.MaxNLocator(4))

    for i, val in enumerate(values):
        label = (
            f"{val:.3g}" if abs(val) >= 1e6 or 0 < abs(val) < 0.01 else f"{val:,.2f}"
        )
        ax.text(
            1.04,
            i,
            label,
            transform=ax.get_yaxis_transform(),
            ha="left",
            va="center",
            fontsize=10,
            color=_KANAGAWA_FG,
        )

    svg = _fig_to_svg(fig, tight=False)
    _plot_cache[cache_key] = svg
    return Response(content=svg, media_type=SVG_MEDIA_TYPE)
