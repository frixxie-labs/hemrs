SELECT
    m.id AS measurement_id,
    m.ts AS timestamp,
    d.name AS device_name,
    s.name AS sensor_name,
    s.unit AS sensor_unit,
    m.value
FROM
    measurements m
    JOIN devices d ON m.device_id = d.id
    JOIN sensors s ON m.sensor_id = s.id
ORDER BY
    m.ts DESC;
