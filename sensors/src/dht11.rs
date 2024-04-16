use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dht11 {
    pub room: String,
    pub temp: f32,
    pub hum: f32,
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
