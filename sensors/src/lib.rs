mod dht11;
mod measurement;
mod temperature;

pub use dht11::Dht11;
pub use measurement::Measurement;
use serde::{Deserialize, Serialize};
pub use temperature::Temperature;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sensors {
    Temperature(temperature::Temperature),
    Dht11(dht11::Dht11),
    Measurement(measurement::Measurement),
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

        let measurement = super::Measurement {
            device: 1,
            sensor: 1,
            measurement: 1.0,
        };
        let measurement_str = serde_json::to_string(&measurement).unwrap();
        let measurement_de: super::Sensors = serde_json::from_str(&measurement_str).unwrap();
        match measurement_de {
            crate::Sensors::Measurement(measurement) => {
                assert_eq!(measurement.device, 1);
                assert_eq!(measurement.sensor, 1);
                assert_eq!(measurement.measurement, 1.0);
            }
            _ => panic!("this is illegal"),
        }
    }
}
