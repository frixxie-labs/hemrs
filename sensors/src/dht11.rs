use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dht11Entry {
    pub ts: DateTime<Utc>,
    pub room: String,
    pub temperature: f32,
    pub humidity: f32,
}

impl Dht11Entry {
    pub fn new(ts: DateTime<Utc>, room: String, temperature: f32, humidity: f32) -> Self {
        Self {
            ts,
            room,
            temperature,
            humidity,
        }
    }
}

impl fmt::Display for Dht11Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timestamp: {}, room: {}, temp: {}, hum: {}",
            self.ts, self.room, self.temperature, self.humidity
        )
    }
}

impl From<Dht11> for Dht11Entry {
    fn from(env_data: Dht11) -> Self {
        Dht11Entry {
            ts: Utc::now(),
            room: env_data.room,
            temperature: env_data.temp,
            humidity: env_data.hum,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dht11 {
    room: String,
    temp: f32,
    hum: f32,
}

impl Dht11 {
    pub fn new(room: String, temp: f32, hum: f32) -> Self {
        Self { room, temp, hum }
    }
}

impl fmt::Display for Dht11 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.room, self.temp, self.hum,)
    }
}

