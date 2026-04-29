use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct DateHelper {
    dt: DateTime<Utc>,
}

impl DateHelper {
    /// Start from current UTC time
    pub fn now() -> Self {
        Self { dt: Utc::now() }
    }

    /// Start from a custom datetime
    pub fn from(dt: DateTime<Utc>) -> Self {
        Self { dt }
    }

    /// Add days
    pub fn add_days(mut self, days: i64) -> Self {
        self.dt = self.dt + Duration::days(days);
        self
    }

    /// Add months (approx 30 days logic via chrono limitation workaround)
    pub fn add_months(mut self, months: i64) -> Self {
        self.dt = self.dt + Duration::days(30 * months);
        self
    }

    /// Add years (approx 365 days logic)
    pub fn add_years(mut self, years: i64) -> Self {
        self.dt = self.dt + Duration::days(365 * years);
        self
    }

    /// Format to string (ISO 8601 default)
    pub fn to_string(self) -> String {
        self.dt.to_rfc3339()
    }

    /// Get raw datetime if needed
    pub fn value(self) -> DateTime<Utc> {
        self.dt
    }
}
