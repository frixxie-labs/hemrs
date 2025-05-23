-- Add migration script here
CREATE TABLE measurements(id SERIAL UNIQUE NOT NULL, ts TIMESTAMP with time zone NOT NULL, device_id SERIAL NOT NULL REFERENCES devices (id), sensor_id SERIAL NOT NULL REFERENCES sensors (id), value REAL NOT NULL, PRIMARY KEY (id));
