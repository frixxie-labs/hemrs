# API Documentation

## Backend Base

The backend binds to `0.0.0.0:65534` by default. Most routes are under `/api`.

OpenAPI JSON is available at `/openapi` and is generated with `utoipa` from route annotations and schema derives. A copy is committed as `dev_docs/openapi.json` for client integrations (such as wictk); a backend test fails when it drifts from the code, and `just openapi` regenerates it.

## Backend JSON Shapes

### Device

```json
{
  "id": 1,
  "name": "Living Room Sensor",
  "location": "Living Room",
  "coordinates": { "latitude": 59.3293, "longitude": 18.0686 }
}
```

`coordinates` is optional (`null` or omitted when unknown). It is stored as a PostGIS `geography(Point, 4326)`; latitude must be within `-90..=90` and longitude within `-180..=180`, otherwise the request body is rejected. `PUT /api/devices` replaces the coordinates, so omitting them clears the stored position.

### NewDevice

```json
{
  "name": "Living Room Sensor",
  "location": "Living Room",
  "coordinates": { "latitude": 59.3293, "longitude": 18.0686 }
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

`timestamp` is optional. If omitted or `null`, the ingestion worker resolves it once to current UTC time, using the same timestamp for the database row and published event.

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

`count` is always a number. `min`, `max`, and `avg` are `null` for an empty dataset;
sample `stddev` and `variance` are `null` with fewer than two samples.

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
| `GET` | `/api/measurements` | Most recent measurements, chronological order | `limit` optional (default `1000`, clamped to `1..=10000`) | `Measurement[]` |
| `GET` | `/api/measurements/latest` | Latest measurement globally | None | `Measurement` |
| `GET` | `/api/measurements/latest/all` | Latest measurement per device/sensor pair | None | `Measurement[]` |
| `GET` | `/api/measurements/count` | Total measurement count | None | JSON number |
| `GET` | `/api/measurements/range` | Measurements in a date range | `start` required, `end` optional | `Measurement[]` |
| `GET` | `/api/devices/{device_id}/measurements` | Most recent measurements for a device, chronological order | `limit` optional (default `1000`, clamped to `1..=10000`) | `Measurement[]` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements` | Most recent measurements for a device/sensor pair, chronological order | `limit` optional (default `1000`, clamped to `1..=10000`) | `Measurement[]` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements/latest` | Latest measurement for a device/sensor pair | None | `Measurement` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements/stats` | Aggregate stats for a device/sensor pair | None | `MeasurementStats` |
| `GET` | `/api/devices/{device_id}/sensors/{sensor_id}/measurements/stream` | Live updates plus a database refresh of the latest value every 15 seconds for a device/sensor pair | None | `text/event-stream` |

A reading is identified by `(device, sensor, timestamp)`. Ingesting the same triple again is accepted but ignored: the first stored value wins, and the duplicate is neither cached nor published to streams.

### Geo Events

Lightning strikes and earthquakes are stored as PostGIS points in their own tables, not as `lat`/`lon` measurements. Sending positions as measurements no longer works: several strikes share a timestamp, so all but the first would be dropped as duplicates.

| Method | Path | Purpose | Body/Query | Success |
| --- | --- | --- | --- | --- |
| `POST` | `/api/lightning` | Store strikes | `NewLightningStrike` or `NewLightningStrike[]` (max 10,000) | `201` |
| `GET` | `/api/lightning` | Most recent strikes, chronological order | `GeoEventParams` | `LightningStrike[]` |
| `POST` | `/api/earthquakes` | Store an earthquake | `NewEarthquake` | `201`, or `200` if already stored |
| `GET` | `/api/earthquakes` | Most recent earthquakes, chronological order | `GeoEventParams` | `Earthquake[]` |

`GeoEventParams`:
- `start` is required and `end` defaults to now (RFC 3339, inclusive).
- `limit` defaults to `1000`, clamped to `1..=10000`.
- `latitude`, `longitude` and `radius_km` are optional but must be given together; they keep only events within that many kilometres of the point.

Invalid combinations return `400`. Posting a strike or earthquake for a device that does not exist also returns `400` with `Unknown device`.

```text
/api/lightning?start=2026-10-29T00:00:00Z&latitude=63.4305&longitude=10.3951&radius_km=50
```

```json
// NewLightningStrike (timestamp optional, defaults to ingestion time) / LightningStrike
{ "timestamp": "2026-10-29T12:00:00Z", "device": 1, "coordinates": { "latitude": 63.2893, "longitude": 25.1497 } }
{ "timestamp": "2026-10-29T12:00:00Z", "device_id": 1, "coordinates": { "latitude": 63.2893, "longitude": 25.1497 } }

// NewEarthquake (all fields required) / Earthquake (same, with device_id)
{ "timestamp": "2026-08-10T10:36:42.622Z", "device": 34, "coordinates": { "latitude": 44.0156, "longitude": 147.4942 },
  "depth_km": 66.149, "magnitude": 4.7, "significance": 340, "tsunami": false }
```

`/api/measurements` and the per-device measurement lists are intentionally bounded so they cannot load the whole table into memory; use `/api/measurements/range` for time windows.

`/api/measurements/range` expects RFC 3339 / ISO 8601 timestamps, for example:

```text
/api/measurements/range?start=2026-05-11T00:00:00Z&end=2026-05-12T00:00:00Z
```

If `end` is absent, the backend defaults it to current UTC time. Ranges are inclusive;
an end before the start returns `400` before querying the database.

Latest-measurement cache updates preserve timestamp ordering, including backfilled
inserts into a cold cache. Live events still include backfills, but periodic stream
snapshots do not regress to an older timestamp.

The measurement stream emits named `measurement` events after measurements have been persisted:

```text
event: measurement
data: {"device_id":1,"sensor_id":2,"measurement":{"timestamp":"2026-09-03T12:00:00Z","value":21.5,"unit":"C","device_name":"Office","device_location":"Upstairs","sensor_name":"Temperature"}}
```

The stream sends keepalive comments every 15 seconds when no measurement is available. Delivery is best-effort: slow subscribers can miss immediate updates and disconnected clients receive no replay. The in-memory broadcaster only carries events from the backend process handling the insertion, while the periodic database refresh makes measurements written through another replica visible within 15 seconds.

## Error Behavior

- Unexpected errors are logged with their causes and return `500 Internal server error`, without exposing database details.
- `HandlerError` responses are plain text, not JSON.
- Explicit validation failures return `400`.
- Missing SQLx `fetch_one` rows return `404 Resource not found`.
- A closed measurement ingestion queue returns `503 Measurement ingestion unavailable`.
- Frontend backend helpers propagate HTTP/transport/JSON errors. Optional device/sensor lookups return `null` only for absent IDs or `404`; valid empty lists and zero counts are preserved.

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
| `GET` | `/plot/measurements/range` | Time-series comparison of up to six devices measuring the same sensor/unit in a backend-filtered range | Required `start`, optional `end`, `sensor`, `page` | `image/svg+xml` |
| `GET` | `/plot/measurements/latest/all` | Horizontal bar comparison of latest readings for up to six devices measuring the same sensor/unit | Optional `sensor`, `page` | `image/svg+xml` |

The overview comparison charts use a fixed 8 × 4.8 inch canvas, so the SVG does not grow with the number of sensors. `sensor` is the exact sensor name. Without it, the sensor/unit shared by the most devices is selected (ties sort by name and unit). `page` defaults to `1`, must be positive, and is clamped to the last device page. Devices sort by name and location; both charts use the latest-readings device list to keep their selections and colors aligned, including devices without readings in the requested history window. History falls back to the range data if no latest readings exist. An explicit `end` must be after `start`.

The frontend overview exposes a sensor picker and previous/next device pages. History remains limited to the last 24 hours. Other sensor types and full latest-reading timestamps remain available in the table below the charts.

Plotter backend failures are translated as follows:

- Backend connection failure: `502 Backend unavailable`.
- Backend timeout: `504 Backend timed out` (HTTP client uses 5-second connect and 30-second read timeouts).
- Backend HTTP error: same status code as the backend response.
- No measurements for a chart: `404`.
