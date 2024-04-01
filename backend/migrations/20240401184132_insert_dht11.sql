-- Add migration script here

INSERT INTO sensors (name, unit) VALUES ('dht11_temperature', 'degree celcius');

INSERT INTO sensors (name, unit) VALUES ('dht11_humidity', 'percent relative humidity');
