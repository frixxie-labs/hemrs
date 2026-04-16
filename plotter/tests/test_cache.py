"""Tests for the plot caching layer."""

from unittest.mock import MagicMock

from tests.conftest import make_measurements


class TestPlotCache:
    """Verify that plot endpoints use the in-memory TTL cache."""

    def test_second_request_serves_cache(self, test_app, mock_client: MagicMock):
        """Second identical request should not call the backend again."""
        mock_client.fetch_all_measurements.return_value = make_measurements(3)

        resp1 = test_app.get("/plot/measurements")
        resp2 = test_app.get("/plot/measurements")

        assert resp1.status_code == 200
        assert resp2.status_code == 200
        mock_client.fetch_all_measurements.assert_called_once()

    def test_different_paths_not_shared(self, test_app, mock_client: MagicMock):
        """Different endpoints should have independent cache entries."""
        mock_client.fetch_measurements_by_device_id.return_value = make_measurements(3)

        test_app.get("/plot/devices/1/measurements")
        test_app.get("/plot/devices/2/measurements")

        assert mock_client.fetch_measurements_by_device_id.call_count == 2

    def test_different_query_params_not_shared(self, test_app, mock_client: MagicMock):
        """Different query strings should produce separate cache entries."""
        mock_client.fetch_measurements_by_date_range.return_value = make_measurements(3)

        test_app.get("/plot/measurements/range?start=2025-01-01T00:00:00")
        test_app.get("/plot/measurements/range?start=2025-06-01T00:00:00")

        assert mock_client.fetch_measurements_by_date_range.call_count == 2

    def test_cache_cleared_between_fixtures(self, test_app, mock_client: MagicMock):
        """Ensure the fixture clears the cache (no leakage from other tests)."""
        import app as app_module

        assert len(app_module._plot_cache) == 0

    def test_device_sensor_endpoint_cached(self, test_app, mock_client: MagicMock):
        mock_client.fetch_measurements_by_device_and_sensor.return_value = (
            make_measurements(3)
        )

        test_app.get("/plot/devices/1/sensors/2/measurements")
        test_app.get("/plot/devices/1/sensors/2/measurements")

        mock_client.fetch_measurements_by_device_and_sensor.assert_called_once()

    def test_latest_all_endpoint_cached(self, test_app, mock_client: MagicMock):
        mock_client.fetch_all_latest_measurements.return_value = make_measurements(3)

        test_app.get("/plot/measurements/latest/all")
        test_app.get("/plot/measurements/latest/all")

        mock_client.fetch_all_latest_measurements.assert_called_once()

    def test_error_not_cached(self, test_app, mock_client: MagicMock):
        """A failed request should not populate the cache."""
        mock_client.fetch_all_measurements.return_value = []

        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 404

        # Now return data — should not be blocked by a cached 404
        mock_client.fetch_all_measurements.return_value = make_measurements(3)
        resp = test_app.get("/plot/measurements")
        assert resp.status_code == 200
