-- no-transaction
CREATE INDEX CONCURRENTLY measurements_device_sensor_ts_idx
ON measurements (device_id, sensor_id, ts DESC);
