use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Measurement {
    pub device: i32,
    pub sensor: i32,
    pub measurement: f32,
}

impl Measurement {
    pub fn new(device: i32, sensor: i32, measurement: f32) -> Self {
        Self {
            device,
            sensor,
            measurement,
        }
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.device, self.sensor, self.measurement)
    }
}
