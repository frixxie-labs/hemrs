# Database

## Technology

- PostgreSQL.
- SQLx migrations in `backend/migrations`.
- SQLx compile-time checked queries in Rust.
- SQLx offline metadata in `.sqlx/` and `backend/.sqlx/`.

## Schema

### `devices`

Created by `backend/migrations/20250314112625_add_devices_table.sql`.

```sql
CREATE TABLE devices(
    id SERIAL UNIQUE NOT NULL,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    PRIMARY KEY(id)
);
```

### `sensors`

Created by `backend/migrations/20250314112749_add_sensors_table.sql`.

```sql
CREATE TABLE sensors(
    id SERIAL UNIQUE NOT NULL,
    name TEXT UNIQUE NOT NULL,
    unit TEXT NOT NULL,
    PRIMARY KEY(id)
);
```

### `measurements`

Created by `backend/migrations/20250314113030_add_measurements_table.sql`.

```sql
CREATE TABLE measurements(
    id SERIAL UNIQUE NOT NULL,
    ts TIMESTAMP with time zone NOT NULL,
    device_id SERIAL NOT NULL REFERENCES devices (id),
    sensor_id SERIAL NOT NULL REFERENCES sensors (id),
    value REAL NOT NULL,
    PRIMARY KEY (id)
);
```

Note: `device_id` and `sensor_id` are declared as `SERIAL` while also being foreign keys. That is unusual because foreign-key columns normally use `INTEGER`; preserve behavior unless intentionally migrating the schema.

## Views

### `device_sensors`

Initially created as a normal view by `20250523123923_devices_sensor_view.sql`, then replaced by a materialized view in `20250727123833_devices_sensors_view_mat.sql`.

```sql
CREATE MATERIALIZED VIEW device_sensors AS
SELECT m.device_id, s.id as sensor_id, s.name, s.unit
FROM measurements m
JOIN sensors s ON m.sensor_id = s.id
GROUP BY (m.device_id, s.id)
ORDER BY m.device_id;
```

Purpose:

- Identify which sensors have reported measurements for each device.
- Power `Sensor::read_by_device_id` and the frontend device-detail page.

Refresh paths:

- `Device::refresh_device_sensors_view` runs `REFRESH MATERIALIZED VIEW device_sensors`.
- Device insert/update calls refresh.
- Sensor insert/update calls refresh.
- Background `refresh_views` loop refreshes every 6000 seconds.

Measurement insert does not refresh the view directly. New device/sensor relationships from fresh measurements may not appear in `/api/devices/{device_id}/sensors` until the refresh loop runs or refresh is triggered elsewhere.

## Query Semantics

Most measurement read queries join all three tables and return denormalized fields:

- timestamp from `measurements.ts`.
- value from `measurements.value`.
- unit and sensor name from `sensors`.
- device name and location from `devices`.

Ordering conventions:

- Historical reads usually order by `ts` ascending.
- Latest reads order by `ts DESC LIMIT 1`.
- Latest all uses PostgreSQL `DISTINCT ON (m.device_id, m.sensor_id)` ordered by device, sensor, and descending timestamp.

Stats query:

```sql
SELECT
  min(value) as "min!",
  max(value) as "max!",
  count(value) as "count!",
  avg(value) as "avg!",
  stddev(value) as "stddev!",
  variance(value) as "variance!"
FROM measurements
WHERE device_id = ($1) AND sensor_id = ($2)
```

## SQLx Workflow

The project uses offline query metadata in CI and Docker builds.

When changing queries or schema:

1. Start Postgres with `docker compose -f docker-compose-test.yaml up --wait`.
2. Run migrations with `cargo sqlx migrate run --source backend/migrations`.
3. Regenerate metadata with `cargo sqlx prepare --workspace`.
4. Stop Postgres with `docker compose -f docker-compose-test.yaml down`.

The `justfile` wraps this as `just sqlx_prepare`.

## Ad-Hoc SQL

`sqls/` contains example queries for manual analysis, such as measurement listings by device and sensor. These are not part of the application runtime but can be useful when debugging production data.
