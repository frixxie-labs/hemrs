"""Tests for Pydantic data models."""

from datetime import datetime, timezone

import pytest
from pydantic import ValidationError

from models import Device, Measurement, MeasurementStats, Sensor


# ---------------------------------------------------------------------------
# Device
# ---------------------------------------------------------------------------


class TestDevice:
    def test_valid(self):
        d = Device(id=1, name="living-room", location="Home")
        assert d.id == 1
        assert d.name == "living-room"
        assert d.location == "Home"

    def test_from_dict(self):
        d = Device.model_validate({"id": 2, "name": "kitchen", "location": "Home"})
        assert d.id == 2

    def test_missing_field_raises(self):
        with pytest.raises(ValidationError):
            Device(id=1, name="x")  # missing location

    def test_wrong_id_type_coerced(self):
        """Pydantic coerces compatible types (str '3' -> int 3)."""
        d = Device(id="3", name="x", location="y")
        assert d.id == 3

    def test_serialization_roundtrip(self):
        d = Device(id=1, name="a", location="b")
        assert Device.model_validate(d.model_dump()) == d


# ---------------------------------------------------------------------------
# Sensor
# ---------------------------------------------------------------------------


class TestSensor:
    def test_valid(self):
        s = Sensor(id=1, name="temperature", unit="°C")
        assert s.unit == "°C"

    def test_missing_field_raises(self):
        with pytest.raises(ValidationError):
            Sensor(id=1, name="temperature")  # missing unit

    def test_serialization_roundtrip(self):
        s = Sensor(id=5, name="humidity", unit="%")
        assert Sensor.model_validate(s.model_dump()) == s


# ---------------------------------------------------------------------------
# Measurement
# ---------------------------------------------------------------------------


class TestMeasurement:
    def test_valid(self):
        m = Measurement(
            timestamp=datetime(2025, 6, 1, 12, 0, 0, tzinfo=timezone.utc),
            value=21.5,
            unit="°C",
            device_name="living-room",
            device_location="Home",
            sensor_name="temperature",
        )
        assert m.value == 21.5
        assert m.timestamp.tzinfo == timezone.utc

    def test_parses_iso_timestamp(self):
        m = Measurement.model_validate(
            {
                "timestamp": "2025-06-01T12:00:00Z",
                "value": 21.5,
                "unit": "°C",
                "device_name": "lr",
                "device_location": "Home",
                "sensor_name": "temp",
            }
        )
        assert m.timestamp.year == 2025

    def test_negative_value(self):
        m = Measurement(
            timestamp=datetime(2025, 1, 15, tzinfo=timezone.utc),
            value=-10.3,
            unit="°C",
            device_name="outdoor",
            device_location="Garden",
            sensor_name="temperature",
        )
        assert m.value == -10.3

    def test_missing_value_raises(self):
        with pytest.raises(ValidationError):
            Measurement(
                timestamp=datetime.now(tz=timezone.utc),
                unit="°C",
                device_name="x",
                device_location="y",
                sensor_name="z",
            )

    def test_serialization_roundtrip(self):
        m = Measurement(
            timestamp=datetime(2025, 6, 1, 12, 0, 0, tzinfo=timezone.utc),
            value=21.5,
            unit="°C",
            device_name="lr",
            device_location="Home",
            sensor_name="temp",
        )
        assert Measurement.model_validate(m.model_dump()) == m


# ---------------------------------------------------------------------------
# MeasurementStats
# ---------------------------------------------------------------------------


class TestMeasurementStats:
    def test_valid(self):
        s = MeasurementStats(
            min=18.0, max=25.0, count=100, avg=21.5, stddev=1.5, variance=2.25
        )
        assert s.count == 100
        assert s.avg == 21.5

    def test_zero_count(self):
        s = MeasurementStats(
            min=0.0, max=0.0, count=0, avg=0.0, stddev=0.0, variance=0.0
        )
        assert s.count == 0

    def test_missing_field_raises(self):
        with pytest.raises(ValidationError):
            MeasurementStats(min=0.0, max=1.0, count=5)  # missing avg, stddev, variance

    def test_serialization_roundtrip(self):
        s = MeasurementStats(
            min=1.0, max=10.0, count=50, avg=5.5, stddev=2.0, variance=4.0
        )
        assert MeasurementStats.model_validate(s.model_dump()) == s
