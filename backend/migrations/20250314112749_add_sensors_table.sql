-- Add migration script here
CREATE TABLE sensors(id SERIAL UNIQUE NOT NULL, name TEXT UNIQUE NOT NULL, unit TEXT NOT NULL, PRIMARY KEY(id, name));
