use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EnvDataEntry {
    pub ts: DateTime<Utc>,
    pub room: String,
    pub temperature: f32,
    pub humidity: f32,
}

impl EnvDataEntry {
    pub fn new(ts: DateTime<Utc>, room: String, temperature: f32, humidity: f32) -> Self {
        Self { ts, room, temperature, humidity }
    }
}

impl fmt::Display for EnvDataEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timestamp: {}, room: {}, temp: {}, hum: {}",
            self.ts, self.room, self.temperature, self.humidity
        )
    }
}

impl From<EnvData> for EnvDataEntry {
    fn from(env_data: EnvData) -> Self {
        EnvDataEntry {
            ts: Utc::now(),
            room: env_data.room,
            temperature: env_data.temp,
            humidity: env_data.hum,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvData {
    room: String,
    temp: f32,
    hum: f32,
}

impl fmt::Display for EnvData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.room, self.temp, self.hum,)
    }
}
