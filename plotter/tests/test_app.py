"""Tests for FastAPI endpoints in app.py."""

from unittest.mock import MagicMock

from requests.exceptions import ConnectionError, HTTPError

from tests.conftest import make_measurement, make_measurements


# ---------------------------------------------------------------------------
# GET /plot/measurements
# ---------------------------------------------------------------------------


class TestPlotAllMeasurements:
    def test_returns_svg(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_measurements.return_value = make_measurements(5)
        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 200
        assert resp.headers["content-type"] == "image/svg+xml"
        assert resp.content.startswith(b"<?xml") or b"<svg" in resp.content

    def test_groups_by_device_sensor(self, test_app, mock_client: MagicMock):
        """Multiple sensors should each produce a series in the SVG."""
        data = make_measurements(
            3, device_name="d1", sensor_name="temp", unit="°C"
        ) + make_measurements(3, device_name="d1", sensor_name="humidity", unit="%")
        mock_client.fetch_all_measurements.return_value = data
        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 200
        svg = resp.text
        # Legend entries should appear in the SVG text
        assert "temp" in svg
        assert "humidity" in svg

    def test_empty_returns_404(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_measurements.return_value = []
        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 404

    def test_backend_connection_error_returns_502(
        self, test_app, mock_client: MagicMock
    ):
        mock_client.fetch_all_measurements.side_effect = ConnectionError("down")
        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 502

    def test_backend_http_error_returns_upstream_code(
        self, test_app, mock_client: MagicMock
    ):
        exc = HTTPError(response=MagicMock(status_code=500))
        mock_client.fetch_all_measurements.side_effect = exc
        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 500


# ---------------------------------------------------------------------------
# GET /plot/devices/{device_id}/measurements
# ---------------------------------------------------------------------------


class TestPlotMeasurementsByDevice:
    def test_returns_svg(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_id.return_value = make_measurements(3)
        resp = test_app.get("/plot/devices/1/measurements")
        assert resp.status_code == 200
        assert "image/svg+xml" in resp.headers["content-type"]

    def test_device_name_in_title(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_id.return_value = make_measurements(
            3, device_name="kitchen"
        )
        resp = test_app.get("/plot/devices/1/measurements")
        assert "kitchen" in resp.text

    def test_empty_returns_404(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_id.return_value = []
        resp = test_app.get("/plot/devices/1/measurements")
        assert resp.status_code == 404

    def test_backend_down_returns_502(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_id.side_effect = ConnectionError()
        resp = test_app.get("/plot/devices/1/measurements")
        assert resp.status_code == 502


# ---------------------------------------------------------------------------
# GET /plot/devices/{device_id}/sensors/{sensor_id}/measurements
# ---------------------------------------------------------------------------


class TestPlotMeasurementsByDeviceAndSensor:
    def test_returns_svg(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_and_sensor.return_value = (
            make_measurements(3)
        )
        resp = test_app.get("/plot/devices/1/sensors/2/measurements")
        assert resp.status_code == 200
        assert b"<svg" in resp.content

    def test_sensor_name_in_svg(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_and_sensor.return_value = (
            make_measurements(3, sensor_name="pressure", unit="hPa")
        )
        resp = test_app.get("/plot/devices/1/sensors/2/measurements")
        assert "pressure" in resp.text
        assert "hPa" in resp.text

    def test_empty_returns_404(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_and_sensor.return_value = []
        resp = test_app.get("/plot/devices/1/sensors/2/measurements")
        assert resp.status_code == 404

    def test_backend_http_error(self, test_app, mock_client: MagicMock):
        exc = HTTPError(response=MagicMock(status_code=503))
        mock_client.fetch_measurements_by_device_and_sensor.side_effect = exc
        resp = test_app.get("/plot/devices/1/sensors/2/measurements")
        assert resp.status_code == 503


# ---------------------------------------------------------------------------
# GET /plot/measurements/range
# ---------------------------------------------------------------------------


class TestPlotMeasurementsByRange:
    def test_returns_svg(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_date_range.return_value = make_measurements(4)
        resp = test_app.get(
            "/plot/measurements/range",
            params={"start": "2025-06-01T00:00:00Z"},
        )
        assert resp.status_code == 200
        assert "image/svg+xml" in resp.headers["content-type"]

    def test_with_end_param(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_date_range.return_value = make_measurements(2)
        resp = test_app.get(
            "/plot/measurements/range",
            params={
                "start": "2025-06-01T00:00:00Z",
                "end": "2025-06-30T23:59:59Z",
            },
        )
        assert resp.status_code == 200
        # End date should appear in title
        assert "2025-06-30" in resp.text

    def test_empty_returns_404(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_date_range.return_value = []
        resp = test_app.get(
            "/plot/measurements/range",
            params={"start": "2025-06-01T00:00:00Z"},
        )
        assert resp.status_code == 404

    def test_missing_start_returns_422(self, test_app, mock_client: MagicMock):
        resp = test_app.get("/plot/measurements/range")
        assert resp.status_code == 422

    def test_backend_down_returns_502(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_date_range.side_effect = ConnectionError()
        resp = test_app.get(
            "/plot/measurements/range",
            params={"start": "2025-06-01T00:00:00Z"},
        )
        assert resp.status_code == 502


# ---------------------------------------------------------------------------
# GET /plot/measurements/latest/all
# ---------------------------------------------------------------------------


class TestPlotAllLatestMeasurements:
    def test_returns_svg_bar_chart(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_latest_measurements.return_value = [
            make_measurement(value=21.5, device_name="lr", sensor_name="temp"),
            make_measurement(
                value=55.0, device_name="lr", sensor_name="humidity", unit="%"
            ),
        ]
        resp = test_app.get("/plot/measurements/latest/all")
        assert resp.status_code == 200
        assert "image/svg+xml" in resp.headers["content-type"]

    def test_value_labels_in_svg(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_latest_measurements.return_value = [
            make_measurement(value=21.50, device_name="lr", sensor_name="temp"),
        ]
        resp = test_app.get("/plot/measurements/latest/all")
        # The formatted value "21.50" should appear in the SVG
        assert "21.50" in resp.text

    def test_empty_returns_404(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_latest_measurements.return_value = []
        resp = test_app.get("/plot/measurements/latest/all")
        assert resp.status_code == 404

    def test_backend_down_returns_502(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_latest_measurements.side_effect = ConnectionError()
        resp = test_app.get("/plot/measurements/latest/all")
        assert resp.status_code == 502

    def test_single_measurement(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_latest_measurements.return_value = [
            make_measurement(value=99.9),
        ]
        resp = test_app.get("/plot/measurements/latest/all")
        assert resp.status_code == 200


# ---------------------------------------------------------------------------
# SVG validity checks
# ---------------------------------------------------------------------------


class TestSvgOutput:
    """Cross-cutting concerns for all SVG-returning endpoints."""

    def test_svg_is_valid_xml(self, test_app, mock_client: MagicMock):
        """Every SVG response should be well-formed XML."""
        import xml.etree.ElementTree as ET

        mock_client.fetch_all_measurements.return_value = make_measurements(3)
        resp = test_app.get("/plot/measurements")
        # Should parse without error
        root = ET.fromstring(resp.content)
        assert root.tag.endswith("svg")

    def test_svg_contains_expected_elements(self, test_app, mock_client: MagicMock):
        """SVG should contain basic plot structure."""
        mock_client.fetch_all_measurements.return_value = make_measurements(3)
        resp = test_app.get("/plot/measurements")
        svg = resp.text
        # matplotlib SVGs include these
        assert "viewBox" in svg or "width" in svg
