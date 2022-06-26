use chrono::prelude::{DateTime, TimeZone};

/// Determines when a file should be "rolled over".
pub trait RollingCondition<Tz: TimeZone> {
    /// Determine and return whether or not the file should be rolled over.
    fn should_rollover(&mut self, now: &DateTime<Tz>, current_filesize: u64) -> bool;
}
