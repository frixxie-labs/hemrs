CREATE TABLE devices(id SERIAL UNIQUE PRIMARY KEY NOT NULL, name TEXT NOT NULL, location TEXT NOT NULL);
CREATE TABLE sensors(id SERIAL UNIQUE PRIMARY KEY NOT NULL, name TEXT NOT NULL, unit TEXT NOT NULL);
CREATE TABLE measurements(ts TIMESTAMP NOT NULL, device_id SERIAL NOT NULL REFERENCES devices (id), sensor_id SERIAL NOT NULL REFERENCES sensors (id), value REAL NOT NULL, PRIMARY KEY (ts, device_id, sensor_id));

insert into sensors (name, unit) values ('dht11_temperature', 'degree celcius');
insert into sensors (name, unit) values ('dht11_humidity', 'degree celcius');
insert into devices (name, location) values ('esp32-0', 'kontoret');
insert into measurements (ts, device_id, sensor_id, value) values (CURRENT_TIMESTAMP, 1, 1, 10.0);
