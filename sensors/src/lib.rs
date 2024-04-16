mod dht11;
mod temperature;

pub use dht11::Dht11;
use serde::{Deserialize, Serialize};
pub use temperature::Temperature;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sensors {
    Temperature(temperature::Temperature),
    Dht11(dht11::Dht11),
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_serialize_deserialize() {
        let temp = super::Temperature {
            room: "room".to_string(),
            temperature: 20.0,
        };
        let temp_str = serde_json::to_string(&temp).unwrap();
        let temp_de: super::Sensors = serde_json::from_str(&temp_str).unwrap();
        match temp_de {
            crate::Sensors::Temperature(measurement) => {
                assert_eq!(measurement.room, "room");
                assert_eq!(measurement.temperature, 20.0);
            }
            _ => panic!("this is illegal"),
        }

        let dht11 = super::Dht11 {
            room: "room".to_string(),
            temp: 20.0,
            hum: 50.0,
        };
        let dht11_str = serde_json::to_string(&dht11).unwrap();
        let dht11_de: super::Sensors = serde_json::from_str(&dht11_str).unwrap();
        match dht11_de {
            crate::Sensors::Dht11(measurement) => {
                assert_eq!(measurement.room, "room");
                assert_eq!(measurement.temp, 20.0);
                assert_eq!(measurement.hum, 50.0);
            }
            _ => panic!("this is illegal"),
        }
    }
}
