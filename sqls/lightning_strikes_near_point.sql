-- Strikes within 50 km of Trondheim (lon, lat); geography distances are in metres.
SELECT
  ts,
  ST_X(geom::geometry) AS lon,
  ST_Y(geom::geometry) AS lat,
  ST_Distance(geom, ST_SetSRID(ST_MakePoint(10.3951, 63.4305), 4326)::geography) / 1000 AS distance_km
FROM lightning_strikes
WHERE $__timeFilter(ts)
  AND ST_DWithin(geom, ST_SetSRID(ST_MakePoint(10.3951, 63.4305), 4326)::geography, 50000)
ORDER BY ts DESC;
