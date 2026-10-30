SELECT
  ts,
  ST_X(geom::geometry) AS lon
FROM lightning_strikes
WHERE $__timeFilter(ts)
ORDER BY ts DESC;
