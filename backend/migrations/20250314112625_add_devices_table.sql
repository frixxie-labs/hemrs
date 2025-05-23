-- Add migration script here
CREATE TABLE devices(id SERIAL UNIQUE NOT NULL, name TEXT NOT NULL, location TEXT NOT NULL, PRIMARY KEY(id));
