use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Temperature {
    pub room: String,
    pub temperature: f32,
}

impl Temperature {
    pub fn new(room: String, temperature: f32) -> Self {
        Self { room, temperature }
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.room, self.temperature)
    }
}
