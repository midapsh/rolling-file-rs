use chrono::{DateTime, Datelike, TimeZone, Timelike};

/// Determines how often a file should be rolled over
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RollingFrequency {
    EveryDay,
    EveryHour,
    EveryMinute,
}

impl RollingFrequency {
    /// Calculates a datetime that will be different if data should be in
    /// different files.
    pub fn equivalent_datetime<Tz: TimeZone>(&self, dt: &DateTime<Tz>) -> DateTime<Tz> {
        match self {
            RollingFrequency::EveryDay => dt.timezone().ymd(dt.year(), dt.month(), dt.day()).and_hms(0, 0, 0),
            RollingFrequency::EveryHour => dt
                .timezone()
                .ymd(dt.year(), dt.month(), dt.day())
                .and_hms(dt.hour(), 0, 0),
            RollingFrequency::EveryMinute => {
                dt.timezone()
                    .ymd(dt.year(), dt.month(), dt.day())
                    .and_hms(dt.hour(), dt.minute(), 0)
            },
        }
    }
}
