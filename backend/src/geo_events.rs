use crate::{coordinates::Coordinates, measurements::date_range::DateRange};

pub mod earthquakes;
pub mod lightning;

/// Events within `radius_km` of `center`, measured along the WGS 84 spheroid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    center: Coordinates,
    radius_km: f64,
}

impl Area {
    pub fn new(center: Coordinates, radius_km: f64) -> Result<Self, &'static str> {
        if radius_km.is_finite() && radius_km > 0.0 {
            Ok(Self { center, radius_km })
        } else {
            Err("radius_km must be a positive number")
        }
    }
}

/// A validated geo event search: the `limit` most recent events in `range`, optionally within `area`.
#[derive(Debug, Clone, Copy)]
pub struct EventQuery {
    pub range: DateRange,
    pub area: Option<Area>,
    pub limit: i64,
}

impl EventQuery {
    fn center(self) -> Option<Coordinates> {
        self.area.map(|area| area.center)
    }

    fn latitude(self) -> Option<f64> {
        self.center().map(Coordinates::latitude)
    }

    fn longitude(self) -> Option<f64> {
        self.center().map(Coordinates::longitude)
    }

    /// PostGIS geography distances are in metres.
    fn radius_m(self) -> Option<f64> {
        self.area.map(|area| area.radius_km * 1_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck_macros::quickcheck]
    fn area_accepts_only_positive_finite_radii(radius_km: f64) -> bool {
        let center = Coordinates::new(63.4, 10.4).unwrap();
        Area::new(center, radius_km).is_ok() == (radius_km.is_finite() && radius_km > 0.0)
    }
}
