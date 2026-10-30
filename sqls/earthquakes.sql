SELECT
  ts,
  ST_X(geom::geometry) AS lon,
  ST_Y(geom::geometry) AS lat,
  depth_km,
  magnitude,
  significance,
  tsunami
FROM earthquakes
WHERE $__timeFilter(ts)
ORDER BY ts DESC;
