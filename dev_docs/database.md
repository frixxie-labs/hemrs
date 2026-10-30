# Database

## Technology

- PostgreSQL 18 with the TimescaleDB and PostGIS extensions, built from `db/Dockerfile` (`postgres:18` plus the TimescaleDB and PGDG apt packages, with `shared_preload_libraries=timescaledb`).
- SQLx migrations in `backend/migrations`.
- SQLx compile-time checked queries in Rust.
- SQLx offline metadata in `.sqlx/` and `backend/.sqlx/`.

Both extensions are created by `20261030120000_geo_events.sql`, so any database used by the backend (including `sqlx::test` databases) must run on a server with them installed.

## Schema

### `devices`

Created by `backend/migrations/20250314112625_add_devices_table.sql`; `geom` added by `20261030120050_measurements_hypertable.sql`.

```sql
CREATE TABLE devices(
    id SERIAL UNIQUE NOT NULL,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    geom geography(Point, 4326),
    PRIMARY KEY(id)
);
```

`location` stays a free-text label; `geom` is the optional geographic position exposed through the API as `coordinates`.

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

Created by `backend/migrations/20250314113030_add_measurements_table.sql` and converted to a TimescaleDB hypertable by `20261030120050_measurements_hypertable.sql`. Lightning and earthquake positions were first moved to their own tables (see below). The conversion then removed exact `(device_id, sensor_id, ts)` duplicates (resends from `wictk_opm`/`wictk_met`, and `finance` quotes stamped with stale timestamps, keeping the first), dropped the unused surrogate `id`, and turned the `SERIAL` foreign-key columns into plain integers.

```sql
CREATE TABLE measurements(
    ts TIMESTAMP with time zone NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices (id),
    sensor_id INTEGER NOT NULL REFERENCES sensors (id),
    value REAL NOT NULL,
    PRIMARY KEY (device_id, sensor_id, ts)
);
-- hypertable partitioned on ts in 1-day chunks; Timescale adds measurements_ts_idx (ts DESC)
```

- The primary key serves latest-measurement lookups for a device/sensor pair, and inserts use `ON CONFLICT DO NOTHING`, so re-sent readings are ignored.
- Compression: chunks older than 7 days are compressed, segmented by `device_id, sensor_id` and ordered by `ts DESC`. Late backfills into compressed chunks still work but are slower.
- Retention: raw chunks older than 1 year are dropped by a Timescale background job. Totals such as `/api/measurements/count` and the stats query cover only the retained raw data.

## Geo Events

Created by `20261030120000_geo_events.sql`. Positions are PostGIS `geography(Point, 4326)` with GiST indexes, so `ST_DWithin`/`ST_Distance` work in metres. Both tables are hypertables with 1-year retention.

```sql
CREATE TABLE lightning_strikes(
    ts TIMESTAMP WITH TIME ZONE NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices (id),
    geom geography(Point, 4326) NOT NULL
);  -- 7-day chunks, compressed after 7 days (segment by device_id)

CREATE TABLE earthquakes(
    ts TIMESTAMP WITH TIME ZONE NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices (id),
    geom geography(Point, 4326) NOT NULL,
    depth_km REAL NOT NULL,
    magnitude REAL NOT NULL,
    significance INTEGER NOT NULL,
    tsunami BOOLEAN NOT NULL,
    PRIMARY KEY (device_id, ts)
);  -- 30-day chunks
```

- `lightning_strikes` has no key: the lightning feed reports several strikes within the same second, and each one is a separate event.
- `earthquakes` is keyed by `(device_id, ts)`; resends are ignored.

Backfill: previously, `wictk_lightning` sent each strike as a `lat` measurement followed by a `lon` measurement. Several strikes can share a timestamp, so the migration pairs the k-th `lat` with the k-th `lon` per timestamp in insertion (`id`) order. Before the migration, a read-only check of production found 1,122,643 pairs, one orphan `lat` (dropped) and no `lon` arriving before its `lat`. `wictk_earthquake` rows (`lat`, `lon`, `earthquake_depth`, `earthquake_magnitude`, `earthquake_significance`, `earthquake_tsunami`, each sent twice) are aggregated per timestamp. The moved measurements are then deleted from `measurements`. Devices are matched by name, so the backfill does nothing on databases without them.

## Continuous Aggregates

Created by `20261030120100_measurement_rollups.sql`, backfilled by the two `backfill_*` migrations, and given refresh policies by `20261030120400_measurement_rollup_policies.sql`. The policies come last so a policy run cannot race the initial refresh. Both use real-time aggregation (`materialized_only = false`) and are not yet used by the API.

| View | Bucket | Refresh policy window | Columns |
| --- | --- | --- | --- |
| `measurements_hourly` | 1 hour | every 30 min, last 3 days up to 1 hour ago | `bucket, device_id, sensor_id, min, max, avg, count` |
| `measurements_daily` | 1 day | every hour, last 7 days up to 1 hour ago | same |

The aggregates have no retention policy, so rollups outlive the raw data. Refresh policies only touch recent windows, so retention never invalidates older buckets.

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
