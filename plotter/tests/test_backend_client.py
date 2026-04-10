"""Tests for BackendClient with mocked HTTP responses."""

from datetime import datetime, timezone
from unittest.mock import MagicMock, patch

import pytest
import requests

from backend_client import BackendClient
from models import Device, Measurement, MeasurementStats, Sensor


@pytest.fixture()
def client() -> BackendClient:
    return BackendClient(base_url="http://test-backend:65534")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _mock_response(json_data, status_code: int = 200) -> MagicMock:
    """Create a mock requests.Response."""
    resp = MagicMock(spec=requests.Response)
    resp.status_code = status_code
    resp.json.return_value = json_data
    resp.raise_for_status.return_value = None
    return resp


def _mock_error_response(status_code: int) -> MagicMock:
    resp = MagicMock(spec=requests.Response)
    resp.status_code = status_code
    resp.raise_for_status.side_effect = requests.exceptions.HTTPError(response=resp)
    return resp


# ---------------------------------------------------------------------------
# Device methods
# ---------------------------------------------------------------------------


class TestFetchDevices:
    def test_returns_list_of_devices(self, client: BackendClient):
        data = [
            {"id": 1, "name": "lr", "location": "Home"},
            {"id": 2, "name": "kitchen", "location": "Home"},
        ]
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            devices = client.fetch_devices()
        assert len(devices) == 2
        assert all(isinstance(d, Device) for d in devices)
        assert devices[0].name == "lr"

    def test_empty_list(self, client: BackendClient):
        with patch.object(client.session, "get", return_value=_mock_response([])):
            devices = client.fetch_devices()
        assert devices == []


class TestFetchDeviceById:
    def test_returns_device(self, client: BackendClient):
        data = {"id": 1, "name": "lr", "location": "Home"}
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            device = client.fetch_device_by_id(1)
        assert isinstance(device, Device)
        assert device.id == 1

    def test_http_error_propagates(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_error_response(404)
        ):
            with pytest.raises(requests.exceptions.HTTPError):
                client.fetch_device_by_id(999)


# ---------------------------------------------------------------------------
# Sensor methods
# ---------------------------------------------------------------------------


class TestFetchSensors:
    def test_returns_list(self, client: BackendClient):
        data = [{"id": 1, "name": "temperature", "unit": "°C"}]
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            sensors = client.fetch_sensors()
        assert len(sensors) == 1
        assert isinstance(sensors[0], Sensor)


class TestFetchSensorById:
    def test_returns_sensor(self, client: BackendClient):
        data = {"id": 1, "name": "temperature", "unit": "°C"}
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            sensor = client.fetch_sensor_by_id(1)
        assert sensor.name == "temperature"


class TestFetchSensorsByDeviceId:
    def test_returns_sensors_for_device(self, client: BackendClient):
        data = [
            {"id": 1, "name": "temperature", "unit": "°C"},
            {"id": 2, "name": "humidity", "unit": "%"},
        ]
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            sensors = client.fetch_sensors_by_device_id(1)
        assert len(sensors) == 2


# ---------------------------------------------------------------------------
# Measurement methods
# ---------------------------------------------------------------------------

SAMPLE_MEASUREMENT = {
    "timestamp": "2025-06-01T12:00:00Z",
    "value": 21.5,
    "unit": "°C",
    "device_name": "lr",
    "device_location": "Home",
    "sensor_name": "temperature",
}


class TestFetchAllMeasurements:
    def test_returns_measurements(self, client: BackendClient):
        data = [SAMPLE_MEASUREMENT]
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            result = client.fetch_all_measurements()
        assert len(result) == 1
        assert isinstance(result[0], Measurement)
        assert result[0].value == 21.5


class TestFetchLatestMeasurement:
    def test_returns_single(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response(SAMPLE_MEASUREMENT)
        ):
            result = client.fetch_latest_measurement()
        assert isinstance(result, Measurement)


class TestFetchAllLatestMeasurements:
    def test_returns_list(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response([SAMPLE_MEASUREMENT])
        ):
            result = client.fetch_all_latest_measurements()
        assert len(result) == 1


class TestFetchMeasurementsCount:
    def test_returns_int(self, client: BackendClient):
        with patch.object(client.session, "get", return_value=_mock_response(42)):
            count = client.fetch_measurements_count()
        assert count == 42


class TestFetchMeasurementsByDateRange:
    def test_with_start_only(self, client: BackendClient):
        mock_resp = _mock_response([SAMPLE_MEASUREMENT])
        with patch.object(client.session, "get", return_value=mock_resp) as mock_get:
            result = client.fetch_measurements_by_date_range(
                start=datetime(2025, 6, 1, tzinfo=timezone.utc)
            )
        assert len(result) == 1
        # Verify params passed correctly
        call_kwargs = mock_get.call_args
        params = call_kwargs.kwargs.get("params") or call_kwargs[1].get("params", {})
        assert "start" in params
        assert "end" not in params

    def test_with_start_and_end(self, client: BackendClient):
        mock_resp = _mock_response([SAMPLE_MEASUREMENT])
        with patch.object(client.session, "get", return_value=mock_resp) as mock_get:
            client.fetch_measurements_by_date_range(
                start=datetime(2025, 6, 1, tzinfo=timezone.utc),
                end=datetime(2025, 6, 30, tzinfo=timezone.utc),
            )
        call_kwargs = mock_get.call_args
        params = call_kwargs.kwargs.get("params") or call_kwargs[1].get("params", {})
        assert "start" in params
        assert "end" in params


class TestFetchMeasurementsByDeviceId:
    def test_returns_measurements(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response([SAMPLE_MEASUREMENT])
        ):
            result = client.fetch_measurements_by_device_id(1)
        assert len(result) == 1


class TestFetchMeasurementsByDeviceAndSensor:
    def test_returns_measurements(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response([SAMPLE_MEASUREMENT])
        ):
            result = client.fetch_measurements_by_device_and_sensor(1, 1)
        assert len(result) == 1


class TestFetchLatestMeasurementByDeviceAndSensor:
    def test_returns_single(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response(SAMPLE_MEASUREMENT)
        ):
            result = client.fetch_latest_measurement_by_device_and_sensor(1, 1)
        assert isinstance(result, Measurement)


class TestFetchStatsByDeviceAndSensor:
    def test_returns_stats(self, client: BackendClient):
        data = {
            "min": 18.0,
            "max": 25.0,
            "count": 100,
            "avg": 21.5,
            "stddev": 1.5,
            "variance": 2.25,
        }
        with patch.object(client.session, "get", return_value=_mock_response(data)):
            result = client.fetch_stats_by_device_and_sensor(1, 1)
        assert isinstance(result, MeasurementStats)
        assert result.avg == 21.5


# ---------------------------------------------------------------------------
# URL construction
# ---------------------------------------------------------------------------


class TestUrlConstruction:
    def test_trailing_slash_stripped(self):
        c = BackendClient(base_url="http://example.com/")
        assert c.base_url == "http://example.com"

    def test_correct_url_for_devices(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response([])
        ) as mock_get:
            client.fetch_devices()
        mock_get.assert_called_once_with("http://test-backend:65534/api/devices")

    def test_correct_url_for_device_sensor_measurements(self, client: BackendClient):
        with patch.object(
            client.session, "get", return_value=_mock_response([])
        ) as mock_get:
            client.fetch_measurements_by_device_and_sensor(3, 7)
        mock_get.assert_called_once_with(
            "http://test-backend:65534/api/devices/3/sensors/7/measurements"
        )
