use chrono::{DateTime, TimeZone};

use crate::{rolling_condition::RollingCondition, rolling_frequency::RollingFrequency};

/// Implements a rolling condition based on a certain frequency
/// and/or a size limit. The default condition is to rotate daily.
///
/// # Examples
///
/// ```rust
/// use chrono::Local;
///
/// use rolling_file::rolling_condition_basic::RollingConditionBasicGeneric;
/// let c = RollingConditionBasicGeneric::<Local>::new().daily();
/// let c = RollingConditionBasicGeneric::<Local>::new().hourly().max_size(1024 * 1024);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollingConditionBasicGeneric<Tz: TimeZone> {
    last_write_opt: Option<DateTime<Tz>>,
    frequency_opt: Option<RollingFrequency>,
    max_size_opt: Option<u64>,
}

impl<Tz: TimeZone> RollingConditionBasicGeneric<Tz> {
    /// Constructs a new struct that does not yet have any condition set.
    pub fn new() -> RollingConditionBasicGeneric<Tz> {
        RollingConditionBasicGeneric {
            last_write_opt: None,
            frequency_opt: None,
            max_size_opt: None,
        }
    }

    /// Sets a condition to rollover on the given frequency
    pub fn frequency(mut self, x: RollingFrequency) -> RollingConditionBasicGeneric<Tz> {
        self.frequency_opt = Some(x);
        self
    }

    /// Sets a condition to rollover when the date changes
    pub fn daily(mut self) -> RollingConditionBasicGeneric<Tz> {
        self.frequency_opt = Some(RollingFrequency::EveryDay);
        self
    }

    /// Sets a condition to rollover when the date or hour changes
    pub fn hourly(mut self) -> RollingConditionBasicGeneric<Tz> {
        self.frequency_opt = Some(RollingFrequency::EveryHour);
        self
    }

    /// Sets a condition to rollover when a certain size is reached
    pub fn max_size(mut self, x: u64) -> RollingConditionBasicGeneric<Tz> {
        self.max_size_opt = Some(x);
        self
    }
}

impl<Tz: TimeZone> Default for RollingConditionBasicGeneric<Tz> {
    fn default() -> Self {
        RollingConditionBasicGeneric::new().frequency(RollingFrequency::EveryDay)
    }
}

impl<Tz: TimeZone> RollingCondition<Tz> for RollingConditionBasicGeneric<Tz> {
    fn should_rollover(&mut self, now: &DateTime<Tz>, current_filesize: u64) -> bool {
        let mut rollover = false;
        if let Some(frequency) = self.frequency_opt.as_ref() {
            if let Some(last_write) = self.last_write_opt.as_ref() {
                if frequency.equivalent_datetime(now) != frequency.equivalent_datetime(last_write) {
                    rollover = true;
                }
            }
        }
        if let Some(max_size) = self.max_size_opt.as_ref() {
            if current_filesize >= *max_size {
                rollover = true;
            }
        }
        self.last_write_opt = Some(now.clone());
        rollover
    }
}
