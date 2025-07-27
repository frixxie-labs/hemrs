-- Add migration script here
drop view device_sensors;

create materialized view device_sensors as select m.device_id, s.id as sensor_id, s.name, s.unit from measurements m join sensors s on m.sensor_id = s.id group by (m.device_id, s.id) order by m.device_id;
