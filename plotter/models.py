from datetime import datetime

from pydantic import BaseModel


class Device(BaseModel):
    id: int
    name: str
    location: str


class Sensor(BaseModel):
    id: int
    name: str
    unit: str


class Measurement(BaseModel):
    timestamp: datetime
    value: float
    unit: str
    device_name: str
    device_location: str
    sensor_name: str


class MeasurementStats(BaseModel):
    min: float | None
    max: float | None
    count: int
    avg: float | None
    stddev: float | None
    variance: float | None
