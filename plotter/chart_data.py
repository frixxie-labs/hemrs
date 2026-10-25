"""Deterministic chart selection and preparation, without HTTP or plotting effects."""

from datetime import datetime

from models import Measurement

OVERVIEW_PAGE_SIZE = 6
SeriesKey = tuple[str, str, str, str]


def series_key(m: Measurement) -> SeriesKey:
    return m.device_name, m.device_location, m.sensor_name, m.unit


def group_series(measurements: list[Measurement]) -> list[list[Measurement]]:
    groups: dict[SeriesKey, list[Measurement]] = {}
    for measurement in measurements:
        groups.setdefault(series_key(measurement), []).append(measurement)
    return [sorted(items, key=lambda m: m.timestamp) for items in groups.values()]


def filter_measurements(
    measurements: list[Measurement], start: datetime | None, end: datetime | None
) -> list[Measurement]:
    return sorted(
        (
            m
            for m in measurements
            if (start is None or m.timestamp >= start)
            and (end is None or m.timestamp <= end)
        ),
        key=lambda m: m.timestamp,
    )


def comparison_page(
    measurements: list[Measurement], sensor: str | None, page: int
) -> list[Measurement]:
    """Choose one sensor/unit and a bounded device page; absence is an empty list."""
    groups: dict[tuple[str, str], dict[tuple[str, str], Measurement]] = {}
    for m in measurements:
        if sensor is None or m.sensor_name == sensor:
            devices = groups.setdefault((m.sensor_name, m.unit), {})
            key = (m.device_name, m.device_location)
            if key not in devices or devices[key].timestamp < m.timestamp:
                devices[key] = m
    if not groups:
        return []
    key = min(groups, key=lambda key: (-len(groups[key]), key))
    devices = groups[key]
    last_page = (len(devices) - 1) // OVERVIEW_PAGE_SIZE + 1
    offset = (max(1, min(page, last_page)) - 1) * OVERVIEW_PAGE_SIZE
    return [
        devices[key] for key in sorted(devices)[offset : offset + OVERVIEW_PAGE_SIZE]
    ]
