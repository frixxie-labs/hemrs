from datetime import datetime

import requests

from models import Device, Measurement, MeasurementStats, Sensor

DEFAULT_BASE_URL = "http://localhost:65534"


class BackendClient:
    """HTTP client for the hemrs backend API."""

    def __init__(self, base_url: str = DEFAULT_BASE_URL) -> None:
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()

    def _get(self, path: str, **kwargs) -> requests.Response:
        resp = self.session.get(f"{self.base_url}{path}", **kwargs)
        resp.raise_for_status()
        return resp

    # -- Devices --

    def fetch_devices(self) -> list[Device]:
        resp = self._get("/api/devices")
        return [Device.model_validate(d) for d in resp.json()]

    def fetch_device_by_id(self, device_id: int) -> Device:
        resp = self._get(f"/api/devices/{device_id}")
        return Device.model_validate(resp.json())

    # -- Sensors --

    def fetch_sensors(self) -> list[Sensor]:
        resp = self._get("/api/sensors")
        return [Sensor.model_validate(s) for s in resp.json()]

    def fetch_sensor_by_id(self, sensor_id: int) -> Sensor:
        resp = self._get(f"/api/sensors/{sensor_id}")
        return Sensor.model_validate(resp.json())

    def fetch_sensors_by_device_id(self, device_id: int) -> list[Sensor]:
        resp = self._get(f"/api/devices/{device_id}/sensors")
        return [Sensor.model_validate(s) for s in resp.json()]

    # -- Measurements --

    def fetch_all_measurements(self) -> list[Measurement]:
        resp = self._get("/api/measurements")
        return [Measurement.model_validate(m) for m in resp.json()]

    def fetch_latest_measurement(self) -> Measurement:
        resp = self._get("/api/measurements/latest")
        return Measurement.model_validate(resp.json())

    def fetch_all_latest_measurements(self) -> list[Measurement]:
        resp = self._get("/api/measurements/latest/all")
        return [Measurement.model_validate(m) for m in resp.json()]

    def fetch_measurements_count(self) -> int:
        resp = self._get("/api/measurements/count")
        return resp.json()

    def fetch_measurements_by_date_range(
        self,
        start: datetime,
        end: datetime | None = None,
    ) -> list[Measurement]:
        params: dict[str, str] = {"start": start.isoformat()}
        if end is not None:
            params["end"] = end.isoformat()
        resp = self._get("/api/measurements/range", params=params)
        return [Measurement.model_validate(m) for m in resp.json()]

    def fetch_measurements_by_device_id(self, device_id: int) -> list[Measurement]:
        resp = self._get(f"/api/devices/{device_id}/measurements")
        return [Measurement.model_validate(m) for m in resp.json()]

    def fetch_measurements_by_device_and_sensor(
        self, device_id: int, sensor_id: int
    ) -> list[Measurement]:
        resp = self._get(f"/api/devices/{device_id}/sensors/{sensor_id}/measurements")
        return [Measurement.model_validate(m) for m in resp.json()]

    def fetch_latest_measurement_by_device_and_sensor(
        self, device_id: int, sensor_id: int
    ) -> Measurement:
        resp = self._get(
            f"/api/devices/{device_id}/sensors/{sensor_id}/measurements/latest"
        )
        return Measurement.model_validate(resp.json())

    def fetch_stats_by_device_and_sensor(
        self, device_id: int, sensor_id: int
    ) -> MeasurementStats:
        resp = self._get(
            f"/api/devices/{device_id}/sensors/{sensor_id}/measurements/stats"
        )
        return MeasurementStats.model_validate(resp.json())
