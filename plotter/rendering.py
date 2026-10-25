"""SVG rendering: all Matplotlib mutation is confined to this module."""

import io
from datetime import datetime
from textwrap import shorten, wrap

import matplotlib

matplotlib.use("svg")

import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
from chart_data import OVERVIEW_PAGE_SIZE, SeriesKey, series_key
from models import Measurement

_KANAGAWA_BG = "#1F1F28"
_KANAGAWA_FG = "#DCD7BA"
_KANAGAWA_GRID = "#54546D"
_KANAGAWA_PALETTE = [
    "#7E9CD8",
    "#E46876",
    "#98BB6C",
    "#E6C384",
    "#957FB8",
    "#7FB4CA",
    "#D27E99",
    "#FF9E3B",
    "#7AA89F",
    "#FFA066",
]


def _kanagawa_color(index: int) -> str:
    return _KANAGAWA_PALETTE[index % len(_KANAGAWA_PALETTE)]


def _apply_kanagawa(fig: plt.Figure, ax: plt.Axes):
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


def _fig_to_svg(fig: plt.Figure, *, tight: bool = True) -> bytes:
    try:
        with io.BytesIO() as buf:
            fig.savefig(buf, format="svg", bbox_inches="tight" if tight else None)
            return buf.getvalue()
    finally:
        plt.close(fig)


def render_time_series(
    groups: list[list[Measurement]], title: str, *, include_device: bool
) -> bytes:
    fig, ax = plt.subplots(figsize=(12, 6))
    for i, items in enumerate(groups):
        m = items[0]
        label = f"{m.sensor_name} ({m.unit})"
        if include_device:
            label = f"{m.device_name} / {m.device_location} / {label}"
        ax.plot(
            [m.timestamp for m in items],
            [m.value for m in items],
            label=label,
            marker=".",
            markersize=3,
            color=_kanagawa_color(i),
        )
    ax.set_xlabel("Time")
    ax.set_ylabel("Value")
    ax.set_title(title)
    ax.legend(fontsize="small", loc="best")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    _apply_kanagawa(fig, ax)
    return _fig_to_svg(fig)


def render_sensor_history(measurements: list[Measurement]) -> bytes:
    timestamps = [m.timestamp for m in measurements]
    values = [m.value for m in measurements]
    first = measurements[0]
    label = f"{first.sensor_name} ({first.unit})"
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.plot(timestamps, values, marker=".", markersize=3, color=_kanagawa_color(0))
    # A quadratic fit requires at least three distinct timestamps. Centering avoids
    # poor conditioning from fitting large Unix timestamp values directly.
    if len(set(timestamps)) >= 3:
        ts_numeric = np.array([(t - timestamps[0]).total_seconds() for t in timestamps])
        coeffs = np.polyfit(ts_numeric, np.array(values), 2)
        ax.plot(
            timestamps,
            np.polyval(coeffs, ts_numeric),
            linestyle=":",
            color=_kanagawa_color(1),
            label="Polynomial regression",
        )
        ax.legend(fontsize="small", loc="best")
    ax.set_xlabel("Time")
    ax.set_ylabel(label)
    ax.set_title(f"{first.device_name} — {label}")
    ax.xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d %H:%M"))
    fig.autofmt_xdate()
    _apply_kanagawa(fig, ax)
    return _fig_to_svg(fig)


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


def render_comparison_history(
    selected: list[Measurement],
    history: list[list[Measurement]],
    start: datetime,
    end: datetime,
) -> bytes:
    groups: dict[SeriesKey, list[Measurement]] = {
        series_key(items[0]): items for items in history
    }
    fig, ax = _comparison_axes(selected[0])
    fig.subplots_adjust(left=0.13, right=0.96, bottom=0.35, top=0.9)
    for i, m in enumerate(selected):
        items = groups.get(series_key(m), [])
        ax.plot(
            [item.timestamp for item in items],
            [item.value for item in items],
            label=_device_label(m),
            marker="." if len(items) < 50 else None,
            markersize=4,
            linewidth=1.8,
            color=_kanagawa_color(i),
        )
    if not any(groups.get(series_key(m)) for m in selected):
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
    ax.set_xlim(start, end)
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
    return _fig_to_svg(fig, tight=False)


def render_latest_comparison(selected: list[Measurement]) -> bytes:
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
    return _fig_to_svg(fig, tight=False)
