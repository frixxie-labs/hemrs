import io
import os
from datetime import datetime

import matplotlib
import matplotlib.dates as mdates
import matplotlib.pyplot as plt
from fastapi import FastAPI, HTTPException, Query
from fastapi.responses import Response
from requests.exceptions import ConnectionError, HTTPError

from backend_client import BackendClient

matplotlib.use("svg")

BACKEND_URL = os.environ.get("BACKEND_URL", "http://localhost:65534")

app = FastAPI(
    title="hemrs plotter",
    version="0.1.0",
    docs_url=None,
    redoc_url=None,
    openapi_url=None,
)
client = BackendClient(base_url=BACKEND_URL)

SVG_MEDIA_TYPE = "image/svg+xml"


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
def plot_all_measurements():
    """Plot all measurements as a time-series SVG, grouped by sensor."""
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

    return Response(content=_fig_to_svg(fig), media_type=SVG_MEDIA_TYPE)


@app.get("/plot/devices/{device_id}/measurements", response_class=Response)
def plot_measurements_by_device(device_id: int):
    """Plot all measurements for a device as a time-series SVG."""
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

    return Response(content=_fig_to_svg(fig), media_type=SVG_MEDIA_TYPE)


@app.get(
    "/plot/devices/{device_id}/sensors/{sensor_id}/measurements",
    response_class=Response,
)
def plot_measurements_by_device_and_sensor(device_id: int, sensor_id: int):
    """Plot measurements for a specific device/sensor pair as a time-series SVG."""
    try:
        measurements = client.fetch_measurements_by_device_and_sensor(
            device_id, sensor_id
        )
    except (ConnectionError, HTTPError) as exc:
        _handle_backend_error(exc)

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

    ax.set_xlabel("Time")
    ax.set_ylabel(f"{measurements[0].sensor_name} ({measurements[0].unit})")
    ax.set_title(f"{measurements[0].device_name} — {label}")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    ax.grid(True, alpha=0.3)

    return Response(content=_fig_to_svg(fig), media_type=SVG_MEDIA_TYPE)


@app.get("/plot/measurements/range", response_class=Response)
def plot_measurements_by_range(
    start: datetime = Query(...),
    end: datetime | None = Query(default=None),
):
    """Plot measurements within a date range as a time-series SVG."""
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

    return Response(content=_fig_to_svg(fig), media_type=SVG_MEDIA_TYPE)


@app.get("/plot/measurements/latest/all", response_class=Response)
def plot_all_latest_measurements():
    """Bar chart of the latest measurement per device/sensor pair as SVG."""
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

    return Response(content=_fig_to_svg(fig), media_type=SVG_MEDIA_TYPE)
