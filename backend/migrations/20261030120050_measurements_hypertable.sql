-- Recreated below once measurements is a hypertable.
DROP MATERIALIZED VIEW device_sensors;

-- Remaining exact (device, sensor, ts) collisions are resends, or finance quotes with stale timestamps; keep the first.
DELETE FROM measurements a
USING measurements b
WHERE a.device_id = b.device_id
  AND a.sensor_id = b.sensor_id
  AND a.ts = b.ts
  AND a.id > b.id;

-- Hypertable unique constraints must include the partitioning column, and nothing reads the surrogate id.
ALTER TABLE measurements DROP COLUMN id;

-- device_id and sensor_id were declared SERIAL; they are plain foreign keys.
ALTER TABLE measurements ALTER COLUMN device_id DROP DEFAULT;
ALTER TABLE measurements ALTER COLUMN sensor_id DROP DEFAULT;
DROP SEQUENCE measurements_device_id_seq;
DROP SEQUENCE measurements_sensor_id_seq;

-- The primary key index serves latest-measurement lookups through a backward scan.
DROP INDEX measurements_device_sensor_ts_idx;
ALTER TABLE measurements ADD PRIMARY KEY (device_id, sensor_id, ts);

SELECT create_hypertable(
    'measurements',
    by_range('ts', INTERVAL '1 day'),
    migrate_data => true
);

ALTER TABLE measurements SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'device_id, sensor_id',
    timescaledb.compress_orderby = 'ts DESC'
);
SELECT add_compression_policy('measurements', INTERVAL '7 days');
SELECT add_retention_policy('measurements', INTERVAL '1 year');

CREATE MATERIALIZED VIEW device_sensors AS
SELECT m.device_id, s.id AS sensor_id, s.name, s.unit
FROM measurements m
JOIN sensors s ON m.sensor_id = s.id
GROUP BY (m.device_id, s.id)
ORDER BY m.device_id;

ALTER TABLE devices ADD COLUMN geom geography(Point, 4326);
