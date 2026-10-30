SELECT
  ts,
  ST_X(geom::geometry) AS lon,
  ST_Y(geom::geometry) AS lat
FROM lightning_strikes
WHERE $__timeFilter(ts)
ORDER BY ts DESC;
