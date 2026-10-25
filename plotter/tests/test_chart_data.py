from datetime import timedelta

from chart_data import comparison_page, filter_measurements, group_series

from tests.conftest import make_measurement


def test_comparison_selection_is_stable_and_keeps_newest_per_location():
    old = make_measurement(device_name="room", device_location="Home", value=1)
    newer = old.model_copy(
        update={"timestamp": old.timestamp + timedelta(hours=1), "value": 2}
    )
    office = make_measurement(device_name="room", device_location="Office")
    readings = [office, newer, old]
    assert comparison_page(readings, "temperature", 1) == [newer, office]
    assert comparison_page(list(reversed(readings)), "temperature", 1) == [
        newer,
        office,
    ]
    assert comparison_page(readings, "missing", 1) == []
    assert comparison_page([], None, 1) == []


def test_comparison_never_combines_units_or_exceeds_page_size():
    readings = [make_measurement(device_name=f"room-{i:02}") for i in range(8)]
    readings += [make_measurement(unit="F", device_name="other")]
    assert len(comparison_page(readings, None, 1)) == 6
    last = comparison_page(readings, None, 999)
    assert [m.device_name for m in last] == ["room-06", "room-07"]
    assert all(m.unit == "°C" for m in last)


def test_grouping_and_filtering_do_not_mutate_input_or_merge_locations():
    first = make_measurement(value=1)
    last = first.model_copy(
        update={"timestamp": first.timestamp + timedelta(hours=1), "value": 2}
    )
    other = make_measurement(device_location="Office")
    readings = [last, first, other]
    assert group_series(readings) == [[first, last], [other]]
    assert filter_measurements(readings, first.timestamp, first.timestamp) == [
        first,
        other,
    ]
    assert filter_measurements(readings, None, None) == [first, other, last]
    assert readings == [last, first, other]
