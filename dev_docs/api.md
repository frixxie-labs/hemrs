# API Documentation

## Backend Base

The backend binds to `0.0.0.0:65534` by default. Most routes are under `/api`.

OpenAPI JSON is available at `/openapi` and is generated with `utoipa` from route annotations and schema derives.

## Backend JSON Shapes

### Device

```json
{
  "id": 1,
  "name": "Living Room Sensor",
  "location": "Living Room"
}
```

### NewDevice

```json
{
  "name": "Living Room Sensor",
  "location": "Living Room"
}
```

### Sensor

```json
{
  "id": 1,
  "name": "Temperature",
  "unit": "Celsius"
}
```

### NewSensor

```json
{
  "name": "Temperature",
  "unit": "Celsius"
}
```

### NewMeasurement

`timestamp` is optional. If omitted or `null`, the backend inserts `CURRENT_TIMESTAMP` in the database and uses `Utc::now()` for cache metadata.

```json
{
  "timestamp": "2026-05-11T12:00:00Z",
  "device": 1,
  "sensor": 1,
  "measurement": 21.5
}
```

Batch ingestion uses an untagged array of `NewMeasurement` objects:

```json
[
  { "device": 1, "sensor": 1, "measurement": 21.5 },
  { "device": 1, "sensor": 2, "measurement": 40.0 }
]
```

### Measurement

Measurement responses are denormalized and include names/units instead of IDs.

```json
{
  "timestamp": "2026-05-11T12:00:00Z",
  "value": 21.5,
  "unit": "Celsius",
  "device_name": "Living Room Sensor",
  "device_location": "Living Room",
  "sensor_name": "Temperature"
}
```

### MeasurementStats

```json
{
  "min": 18.2,
  "max": 24.8,
  "count": 128,
  "avg": 21.4,
  "stddev": 1.2,
  "variance": 1.44
}
```

## Backend Routes

### System

| Method | Path | Purpose | Notes |
| --- | --- | --- | --- |
| `GET` | `/status/ping` | Health check | Returns ping response from `handlers/ping.rs`. |
| `GET` | `/metrics` | Prometheus metrics | Returns text exposition from `PrometheusHandle::render()`. |
| `GET` | `/openapi` | OpenAPI JSON | Pretty-printed JSON string. |

### Devices

| Method | Path | Purpose | Body | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/devices` | List devices | None | `Device[]` |
| `GET` | `/api/devices/{device_id}` | Fetch one device | None | `Device` |
| `POST` | `/api/devices` | Create device | `NewDevice` | Text `OK` |
| `PUT` | `/api/devices` | Update device | `Device` | Text `OK` |
| `DELETE` | `/api/devices` | Delete device | `Device` | Text `OK` |
| `GET` | `/api/devices/{device_id}/sensors` | Sensors observed for a device | None | `Sensor[]` |

Device create/update/delete validate that `name` and `location` are non-empty. Invalid input returns `400` with a plain text message.

### Sensors

| Method | Path | Purpose | Body | Success |
| --- | --- | --- | --- | --- |
| `GET` | `/api/sensors` | List sensors | None | `Sensor[]` |
| `GET` | `/api/sensors/{sensor_id}` | Fetch one sensor | None | `Sensor` |
| `POST` | `/api/sensors` | Create sensor | `NewSensor` | Text `OK` |
| `PUT` | `/api/sensors` | Update sensor | `Sensor` | Text `OK` |
| `DELETE` | `/api/sensors` | Delete sensor | `Sensor` | Text `OK` |

Sensor create/update/delete validate that `name` and `unit` are non-empty. Invalid input returns `400` with a plain text message.

There is also an annotated handler path `api/sensors/device/{device_id}`, but the active router exposes device sensors as `/api/devices/{device_id}/sensors`.

### Measurements

| Method | Path | Purpose | Body/Query | Success |
| --- | --- | --- | --- | --- |
| `POST` | `/` | Direct measurement ingestion | `NewMeasurement` or `NewMeasurement[]` | `201` text |
| `POST` | `/api/measurements` | Measurement ingestion | `NewMeasurement` or `NewMeasurement[]` | `201` text |
| `GET` | `/api/measurements` | All measurements | None | `Measurement[]` |
| `GET` | `/api/measurements/latest` | Latest measurement globally | None | `Measurement` |
| `GET` | `/api/measurements/latest/all` | Latest measurement per device/sensor pair | None | `Measurement[]` |
| `GET` | `/api/measurements/count` | Total measurement count | None | JSON number |
| `GET` | `/api/measurements/range` | Measurements in a date range | `start` required, `end` optional | `Measurement[]` |
| `GET` | `/api/devices/{device_id}/measurements` | All measurements for a device | None | `Measurement[]` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements` | Measurements for a device/sensor pair | None | `Measurement[]` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements/latest` | Latest measurement for a device/sensor pair | None | `Measurement` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements/stats` | Aggregate stats for a device/sensor pair | None | `MeasurementStats` |

`/api/measurements/range` expects RFC 3339 / ISO 8601 timestamps, for example:

```text
/api/measurements/range?start=2026-05-11T00:00:00Z&end=2026-05-12T00:00:00Z
```

If `end` is absent, the backend defaults it to current UTC time.

## Error Behavior

- Domain/data-access errors are wrapped with `anyhow::Context` and converted to `HandlerError` with status `500`.
- `HandlerError` responses are plain text, not JSON.
- Explicit validation failures use `HandlerError::new(400, ...)`.
- Fetching missing rows through SQLx `fetch_one` currently becomes a `500`, not a `404`, unless a frontend route maps a `null` client response to `HttpError(404)`.

## Plotter API

The plotter binds to `0.0.0.0:8000` by default and returns SVG bytes for chart routes.

### System

| Method | Path | Purpose | Response |
| --- | --- | --- | --- |
| `GET` | `/status/health` | Health check | JSON `{ "status": "healthy" }` |
| `GET` | `/status/ping` | Ping check | JSON `{ "ping": "pong" }` |

### Charts

| Method | Path | Purpose | Query | Response |
| --- | --- | --- | --- | --- |
| `GET` | `/plot/measurements` | Time-series for all measurements grouped by device/sensor/unit | None | `image/svg+xml` |
| `GET` | `/plot/devices/{device_id}/measurements` | Time-series for one device grouped by sensor/unit | None | `image/svg+xml` |
| `GET` | `/plot/devices/{device_id}/sensors/{sensor_id}/measurements` | Time-series for one device/sensor pair with polynomial regression when possible | Optional `start`, `end` | `image/svg+xml` |
| `GET` | `/plot/measurements/range` | Time-series for measurements in a backend-filtered range | Required `start`, optional `end` | `image/svg+xml` |
| `GET` | `/plot/measurements/latest/all` | Bar chart of latest values per device/sensor pair | None | `image/svg+xml` |

Plotter backend failures are translated as follows:

- Backend connection failure: `502 Backend unavailable`.
- Backend HTTP error: same status code as the backend response.
- No measurements for a chart: `404`.
