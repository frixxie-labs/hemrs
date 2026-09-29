"""Regression coverage for oversized, mixed-unit overview charts."""

import xml.etree.ElementTree as ET
from unittest.mock import patch

import app
import pytest

from tests.conftest import make_measurement, make_measurements


@pytest.fixture
def dense_readings():
    return [
        make_measurement(device_name=f"room-{i:02}", value=i - 3) for i in range(14)
    ] + [
        make_measurement(
            device_name="finance", sensor_name=f"ticker-{i}", unit="shares", value=1e9
        )
        for i in range(150)
    ]


@pytest.mark.parametrize(
    "path", ["/plot/measurements/range", "/plot/measurements/latest/all"]
)
def test_dense_overview_has_bounded_dimensions_and_same_unit(
    test_app, mock_client, dense_readings, path
):
    mock_client.fetch_all_latest_measurements.return_value = dense_readings
    mock_client.fetch_measurements_by_date_range.return_value = dense_readings
    with patch.object(app, "_fig_to_svg", wraps=app._fig_to_svg) as render:
        response = test_app.get(path, params={"start": "2025-06-01T00:00:00Z"})
        assert response.status_code == 200
        root = ET.fromstring(response.content)
        assert [float(n) for n in root.attrib["viewBox"].split()] == [0, 0, 576, 345.6]
        ax = render.call_args.args[0].axes[0]
        assert ax.get_title() == "temperature"
        if "range" in path:
            assert len(ax.lines) == 6
            assert ax.get_ylabel() == "°C"
            assert "room-05" in response.text
        else:
            assert len(ax.patches) == 6
            assert ax.get_xlabel() == "°C"
            assert ax.patches[0].get_width() == -3
        assert "ticker-" not in response.text
        assert "room-06" not in response.text


def test_page_selection_and_colors_match_with_missing_history(
    test_app, mock_client, dense_readings
):
    mock_client.fetch_all_latest_measurements.return_value = dense_readings
    # First device on page 2 has no history; it must keep its color/position.
    mock_client.fetch_measurements_by_date_range.return_value = make_measurements(
        3, device_name="room-07"
    )
    params = {"sensor": "temperature", "page": 2, "start": "2025-06-01T00:00:00Z"}
    with patch.object(app, "_fig_to_svg", wraps=app._fig_to_svg) as render:
        history = test_app.get("/plot/measurements/range", params=params)
        history_ax = render.call_args.args[0].axes[0]
        latest = test_app.get("/plot/measurements/latest/all", params=params)
        latest_ax = render.call_args.args[0].axes[0]
    assert history.status_code == latest.status_code == 200
    assert not len(history_ax.lines[0].get_xdata())
    assert len(history_ax.lines[1].get_xdata()) == 3
    assert history_ax.lines[1].get_color() == app._kanagawa_color(1)
    from matplotlib.colors import to_rgba

    assert latest_ax.patches[1].get_facecolor() == to_rgba(app._kanagawa_color(1))
    for response in [history, latest]:
        assert "room-06" in response.text
        assert "room-11" in response.text
        assert "room-05" not in response.text
        assert "room-12" not in response.text


def test_sensor_selection_cache_and_last_page(test_app, mock_client, dense_readings):
    mock_client.fetch_all_latest_measurements.return_value = dense_readings
    path = "/plot/measurements/latest/all"
    first = test_app.get(path, params={"sensor": "temperature"})
    last = test_app.get(path, params={"sensor": "temperature", "page": 999})
    finance = test_app.get(path, params={"sensor": "ticker-42"})
    assert "room-00" in first.text and "room-00" not in last.text
    assert "room-13" in last.text
    assert "shares" in finance.text and "temperature" not in finance.text
    assert test_app.get(path, params={"sensor": "missing"}).status_code == 404
    assert test_app.get(path, params={"page": 0}).status_code == 422


def test_series_at_different_locations_stay_distinct(test_app, mock_client):
    mock_client.fetch_all_latest_measurements.return_value = [
        make_measurement(device_name="room", device_location="Home"),
        make_measurement(device_name="room", device_location="Office"),
    ]
    response = test_app.get("/plot/measurements/latest/all")
    assert "Home" in response.text and "Office" in response.text
