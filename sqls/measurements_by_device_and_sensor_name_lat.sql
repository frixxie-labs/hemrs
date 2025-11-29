SELECT
  m.ts,
  MAX(m.value) FILTER (WHERE s.name = 'lat') AS lat
FROM measurements m
JOIN sensors s ON m.sensor_id = s.id
WHERE s.name IN ('lon', 'lat') AND $__timeFilter(m.ts)
GROUP BY m.device_id, m.ts
ORDER BY m.ts DESC;
