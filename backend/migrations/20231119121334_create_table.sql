-- Add migration script here

CREATE TABLE env_data(ts TIMESTAMP with time zone not null, room TEXT not null, temperature REAL not null, humidity REAL not null);
