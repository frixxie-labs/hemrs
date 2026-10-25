use chrono::{DateTime, Utc};

/// Validated inclusive range; callers resolve defaults before querying persistence.
#[derive(Debug, Clone, Copy)]
pub struct DateRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl DateRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, &'static str> {
        if end < start {
            return Err("End must not be before start");
        }
        Ok(Self { start, end })
    }

    pub fn start(self) -> DateTime<Utc> {
        self.start
    }

    pub fn end(self) -> DateTime<Utc> {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusive_ranges_allow_equal_endpoints_but_not_reversed_ones() {
        let now = Utc::now();
        assert!(DateRange::new(now, now).is_ok());
        assert!(DateRange::new(now, now - chrono::Duration::seconds(1)).is_err());
    }
}
