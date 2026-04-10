from datetime import datetime, timezone
from unittest.mock import MagicMock

import pytest
from fastapi.testclient import TestClient

from models import Device, Measurement, MeasurementStats, Sensor


# ---------------------------------------------------------------------------
# Sample data factories
# ---------------------------------------------------------------------------


def make_device(
    id: int = 1, name: str = "living-room", location: str = "Home"
) -> Device:
    return Device(id=id, name=name, location=location)


def make_sensor(id: int = 1, name: str = "temperature", unit: str = "°C") -> Sensor:
    return Sensor(id=id, name=name, unit=unit)


def make_measurement(
    timestamp: datetime | None = None,
    value: float = 21.5,
    unit: str = "°C",
    device_name: str = "living-room",
    device_location: str = "Home",
    sensor_name: str = "temperature",
) -> Measurement:
    return Measurement(
        timestamp=timestamp or datetime(2025, 6, 1, 12, 0, 0, tzinfo=timezone.utc),
        value=value,
        unit=unit,
        device_name=device_name,
        device_location=device_location,
        sensor_name=sensor_name,
    )


def make_measurements(count: int = 5, **kwargs) -> list[Measurement]:
    """Create a list of measurements with incrementing timestamps and values."""
    base_ts = kwargs.pop(
        "base_timestamp", datetime(2025, 6, 1, 12, 0, 0, tzinfo=timezone.utc)
    )
    base_val = kwargs.pop("base_value", 20.0)
    return [
        make_measurement(
            timestamp=base_ts.replace(hour=12 + i),
            value=base_val + i * 0.5,
            **kwargs,
        )
        for i in range(count)
    ]


def make_stats(
    min: float = 18.0,
    max: float = 25.0,
    count: int = 100,
    avg: float = 21.5,
    stddev: float = 1.5,
    variance: float = 2.25,
) -> MeasurementStats:
    return MeasurementStats(
        min=min, max=max, count=count, avg=avg, stddev=stddev, variance=variance
    )


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture()
def mock_client() -> MagicMock:
    """A MagicMock that stands in for BackendClient."""
    return MagicMock(
        spec_set=[
            "fetch_devices",
            "fetch_device_by_id",
            "fetch_sensors",
            "fetch_sensor_by_id",
            "fetch_sensors_by_device_id",
            "fetch_all_measurements",
            "fetch_latest_measurement",
            "fetch_all_latest_measurements",
            "fetch_measurements_count",
            "fetch_measurements_by_date_range",
            "fetch_measurements_by_device_id",
            "fetch_measurements_by_device_and_sensor",
            "fetch_latest_measurement_by_device_and_sensor",
            "fetch_stats_by_device_and_sensor",
        ]
    )


@pytest.fixture()
def test_app(mock_client: MagicMock) -> TestClient:
    """FastAPI TestClient with the real BackendClient replaced by a mock."""
    import app as app_module

    original_client = app_module.client
    app_module.client = mock_client
    try:
        yield TestClient(app_module.app)
    finally:
        app_module.client = original_client
