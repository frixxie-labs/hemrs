"""Tests for the _fig_to_svg helper and _handle_backend_error."""

import xml.etree.ElementTree as ET
from unittest.mock import MagicMock

import matplotlib
import matplotlib.pyplot as plt
import pytest
from fastapi import HTTPException
from requests.exceptions import ConnectionError, HTTPError

matplotlib.use("svg")

from app import _fig_to_svg, _handle_backend_error


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
        fig, ax = plt.subplots()
        result = _fig_to_svg(fig)
        assert len(result) > 0
        assert b"<svg" in result

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


class TestHandleBackendError:
    def test_connection_error_raises_502(self):
        with pytest.raises(HTTPException) as exc_info:
            _handle_backend_error(ConnectionError("refused"))
        assert exc_info.value.status_code == 502
        assert "Backend unavailable" in exc_info.value.detail

    def test_http_error_preserves_status_code(self):
        mock_response = MagicMock()
        mock_response.status_code = 404
        exc = HTTPError(response=mock_response)
        with pytest.raises(HTTPException) as exc_info:
            _handle_backend_error(exc)
        assert exc_info.value.status_code == 404

    def test_http_500_error(self):
        mock_response = MagicMock()
        mock_response.status_code = 500
        exc = HTTPError(response=mock_response)
        with pytest.raises(HTTPException) as exc_info:
            _handle_backend_error(exc)
        assert exc_info.value.status_code == 500

    def test_unknown_error_re_raised(self):
        with pytest.raises(ValueError):
            _handle_backend_error(ValueError("something unexpected"))
