use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A WGS 84 position, stored as a PostGIS `geography(Point, 4326)`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "UncheckedCoordinates")]
pub struct Coordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct UncheckedCoordinates {
    latitude: f64,
    longitude: f64,
}

impl TryFrom<UncheckedCoordinates> for Coordinates {
    type Error = String;

    fn try_from(raw: UncheckedCoordinates) -> Result<Self, Self::Error> {
        Self::new(raw.latitude, raw.longitude)
    }
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, String> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(format!("latitude {latitude} is outside -90..=90"));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(format!("longitude {longitude} is outside -180..=180"));
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }

    pub fn latitude(self) -> f64 {
        self.latitude
    }

    pub fn longitude(self) -> f64 {
        self.longitude
    }

    /// Reads a nullable point back from `ST_Y`/`ST_X` columns.
    pub fn from_columns(
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> anyhow::Result<Option<Self>> {
        latitude
            .zip(longitude)
            .map(|(latitude, longitude)| Self::new(latitude, longitude))
            .transpose()
            .map_err(anyhow::Error::msg)
    }
}

#[cfg(test)]
mod tests {
    use super::Coordinates;

    #[quickcheck_macros::quickcheck]
    fn coordinates_accept_exactly_the_valid_ranges(latitude: f64, longitude: f64) -> bool {
        let valid = (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude);
        Coordinates::new(latitude, longitude).is_ok() == valid
    }

    #[test]
    fn coordinates_reject_out_of_range_json() {
        for json in [
            r#"{"latitude":91,"longitude":0}"#,
            r#"{"latitude":0,"longitude":-181}"#,
        ] {
            assert!(serde_json::from_str::<Coordinates>(json).is_err());
        }
    }
}
