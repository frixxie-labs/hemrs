CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS postgis;

-- Fresh statistics keep the planner from misjudging the large backfill queries below.
ANALYZE measurements;

-- Several strikes can share a timestamp, so strikes have no natural key.
CREATE TABLE lightning_strikes(
    ts TIMESTAMP WITH TIME ZONE NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices (id),
    geom geography(Point, 4326) NOT NULL
);
SELECT create_hypertable('lightning_strikes', by_range('ts', INTERVAL '7 days'));
CREATE INDEX lightning_strikes_geom_idx ON lightning_strikes USING GIST (geom);
ALTER TABLE lightning_strikes SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'device_id',
    timescaledb.compress_orderby = 'ts DESC'
);
SELECT add_compression_policy('lightning_strikes', INTERVAL '7 days');
SELECT add_retention_policy('lightning_strikes', INTERVAL '1 year');

CREATE TABLE earthquakes(
    ts TIMESTAMP WITH TIME ZONE NOT NULL,
    device_id INTEGER NOT NULL REFERENCES devices (id),
    geom geography(Point, 4326) NOT NULL,
    depth_km REAL NOT NULL,
    magnitude REAL NOT NULL,
    significance INTEGER NOT NULL,
    tsunami BOOLEAN NOT NULL,
    PRIMARY KEY (device_id, ts)
);
SELECT create_hypertable('earthquakes', by_range('ts', INTERVAL '30 days'));
CREATE INDEX earthquakes_geom_idx ON earthquakes USING GIST (geom);
SELECT add_retention_policy('earthquakes', INTERVAL '1 year');

-- Positions used to arrive as separate lat/lon measurements. Several strikes may share a
-- timestamp, so the k-th lat pairs with the k-th lon in insertion (id) order. Grouping on
-- (device, ts, k) pairs them in one pass; a self-join risks a quadratic nested loop.
-- REAL values go through text to keep their shortest decimal form, e.g. 63.2893 rather than 63.28929901.
INSERT INTO lightning_strikes (ts, device_id, geom)
SELECT ts, device_id, ST_SetSRID(ST_MakePoint(lon, lat), 4326)::geography
FROM (
    SELECT device_id, ts,
           max(degrees) FILTER (WHERE axis = 'lat') AS lat,
           max(degrees) FILTER (WHERE axis = 'lon') AS lon
    FROM (
        SELECT m.device_id, m.ts, s.name AS axis, m.value::text::float8 AS degrees,
               row_number() OVER (PARTITION BY m.device_id, m.ts, s.name ORDER BY m.id) AS k
        FROM measurements m
        JOIN sensors s ON s.id = m.sensor_id
        JOIN devices d ON d.id = m.device_id
        WHERE s.name IN ('lat', 'lon') AND d.name = 'wictk_lightning'
    ) positions
    GROUP BY device_id, ts, k
) strike
WHERE lat IS NOT NULL AND lon IS NOT NULL;

-- Earthquake readings were each sent twice with identical values, so aggregating per timestamp is exact.
INSERT INTO earthquakes (ts, device_id, geom, depth_km, magnitude, significance, tsunami)
SELECT ts, device_id, ST_SetSRID(ST_MakePoint(lon, lat), 4326)::geography,
       depth_km, magnitude, significance::integer, tsunami <> 0
FROM (
    SELECT m.device_id, m.ts,
           max(m.value::text::float8) FILTER (WHERE s.name = 'lat') AS lat,
           max(m.value::text::float8) FILTER (WHERE s.name = 'lon') AS lon,
           max(m.value) FILTER (WHERE s.name = 'earthquake_depth') AS depth_km,
           max(m.value) FILTER (WHERE s.name = 'earthquake_magnitude') AS magnitude,
           max(m.value) FILTER (WHERE s.name = 'earthquake_significance') AS significance,
           max(m.value) FILTER (WHERE s.name = 'earthquake_tsunami') AS tsunami
    FROM measurements m
    JOIN sensors s ON s.id = m.sensor_id
    JOIN devices d ON d.id = m.device_id
    WHERE d.name = 'wictk_earthquake'
    GROUP BY m.device_id, m.ts
) quake
WHERE lat IS NOT NULL AND lon IS NOT NULL AND depth_km IS NOT NULL
  AND magnitude IS NOT NULL AND significance IS NOT NULL AND tsunami IS NOT NULL;

DELETE FROM measurements m
USING sensors s, devices d
WHERE s.id = m.sensor_id
  AND d.id = m.device_id
  AND (
    (d.name = 'wictk_lightning' AND s.name IN ('lat', 'lon'))
    OR (d.name = 'wictk_earthquake' AND s.name IN (
        'lat', 'lon', 'earthquake_depth', 'earthquake_magnitude',
        'earthquake_significance', 'earthquake_tsunami'))
  );
