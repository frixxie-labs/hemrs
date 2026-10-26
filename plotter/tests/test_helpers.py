"""Tests for the _fig_to_svg helper and _backend_http_error."""

import warnings
import xml.etree.ElementTree as ET
from unittest.mock import MagicMock

import matplotlib
import matplotlib.pyplot as plt
import pytest
from requests.exceptions import ConnectionError, HTTPError, Timeout

matplotlib.use("svg")

from app import _backend_http_error
from rendering import _fig_to_svg, render_sensor_history

from tests.conftest import make_measurements


class TestFigToSvg:
    def test_returns_bytes(self):
        fig, ax = plt.subplots()
        ax.plot([1, 2, 3], [1, 4, 9])
        result = _fig_to_svg(fig)
        assert isinstance(result, bytes)

    def test_output_is_valid_svg(self):
        fig, ax = plt.subplots()
        ax.plot([0, 1], [0, 1])
        svg_bytes = _fig_to_svg(fig)
        root = ET.fromstring(svg_bytes)
        assert root.tag.endswith("svg")

    def test_figure_is_closed(self):
        fig, ax = plt.subplots()
        ax.plot([0], [0])
        fig_number = fig.number
        _fig_to_svg(fig)
        # After _fig_to_svg, the figure should be closed
        assert fig_number not in plt.get_fignums()

    def test_empty_plot(self):
        fig, _ = plt.subplots()
        result = _fig_to_svg(fig)
        assert len(result) > 0
        assert b"<svg" in result

    def test_sensor_regression_requires_three_distinct_samples(self):
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            two = render_sensor_history(make_measurements(2))
            three = render_sensor_history(make_measurements(3))
        assert b"Polynomial regression" not in two
        assert b"Polynomial regression" in three

    def test_figure_is_closed_when_rendering_fails(self, monkeypatch):
        fig, _ = plt.subplots()

        def fail(*args, **kwargs):
            raise OSError("Rendering failed")

        monkeypatch.setattr(fig, "savefig", fail)
        with pytest.raises(OSError, match="Rendering failed"):
            _fig_to_svg(fig)
        assert fig.number not in plt.get_fignums()

    def test_multiple_subplots(self):
        fig, (ax1, ax2) = plt.subplots(1, 2)
        ax1.plot([1, 2], [3, 4])
        ax2.bar([0, 1], [5, 6])
        result = _fig_to_svg(fig)
        assert b"<svg" in result

    def test_large_dataset(self):
        fig, ax = plt.subplots()
        ax.plot(range(1000), range(1000))
        result = _fig_to_svg(fig)
        assert b"<svg" in result


class TestBackendHttpError:
    def test_timeout_maps_to_504(self):
        assert _backend_http_error(Timeout("Backend timed out")).status_code == 504

    def test_connection_error_maps_to_502(self):
        error = _backend_http_error(ConnectionError("refused"))
        assert error.status_code == 502
        assert "Backend unavailable" in error.detail

    @pytest.mark.parametrize("status_code", [404, 500])
    def test_http_error_preserves_status_code(self, status_code):
        mock_response = MagicMock()
        mock_response.status_code = status_code
        error = _backend_http_error(HTTPError(response=mock_response))
        assert error.status_code == status_code
