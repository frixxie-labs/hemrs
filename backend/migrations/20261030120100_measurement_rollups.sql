-- Rollups keep long-term history after raw retention drops old chunks.
CREATE MATERIALIZED VIEW measurements_hourly
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT
    time_bucket(INTERVAL '1 hour', ts) AS bucket,
    device_id,
    sensor_id,
    min(value) AS min,
    max(value) AS max,
    avg(value) AS avg,
    count(*) AS count
FROM measurements
GROUP BY bucket, device_id, sensor_id
WITH NO DATA;

CREATE MATERIALIZED VIEW measurements_daily
WITH (timescaledb.continuous, timescaledb.materialized_only = false) AS
SELECT
    time_bucket(INTERVAL '1 day', ts) AS bucket,
    device_id,
    sensor_id,
    min(value) AS min,
    max(value) AS max,
    avg(value) AS avg,
    count(*) AS count
FROM measurements
GROUP BY bucket, device_id, sensor_id
WITH NO DATA;

