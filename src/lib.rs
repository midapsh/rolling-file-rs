#![deny(warnings)]
pub mod rolling_condition;
pub mod rolling_condition_basic;
pub mod rolling_file_appender;
pub mod rolling_file_appender_generic;
pub mod rolling_file_appender_utc;
pub mod rolling_frequency;
pub mod system_clock;

use rolling_condition_basic::RollingConditionBasicGeneric;
use rolling_file_appender::RollingFileAppender;
use rolling_file_appender_generic::RollingFileAppenderGeneric;
use rolling_file_appender_utc::RollingFileAppenderUtc;

use chrono::{Local, Utc};

/// A rolling file appender with a rolling condition based on date/time or size.
pub type RollingConditionBasic = RollingConditionBasicGeneric<Local>;
pub type BasicRollingFileAppender = RollingFileAppender<RollingConditionBasic>;
pub type RollingConditionBasicUtc = RollingConditionBasicGeneric<Utc>;
pub type BasicRollingFileAppenderUtc = RollingFileAppenderUtc<RollingConditionBasicUtc>;
pub type BasicRollingFileAppenderGeneric<Tz> = RollingFileAppenderGeneric<RollingConditionBasicGeneric<Tz>, Tz>;

// LCOV_EXCL_START
#[cfg(test)]
mod test_basic_rolling_appender {
    use chrono::TimeZone;

    use crate::rolling_frequency::RollingFrequency;

    use super::*;
    use std::{fs, io::Write, path::Path};

    struct Context {
        _tempdir: tempfile::TempDir,
        rolling: BasicRollingFileAppender,
    }

    impl Context {
        fn verify_contains(&mut self, needle: &str, n: usize) {
            self.rolling.flush().unwrap();
            let p = self.rolling.filename_for(n);
            let haystack = fs::read_to_string(&p).unwrap();
            if !haystack.contains(needle) {
                panic!("file {:?} did not contain expected contents {}", p, needle);
            }
        }
    }

    fn build_context(condition: RollingConditionBasic, max_files: usize) -> Context {
        let tempdir = tempfile::tempdir().unwrap();
        let rolling =
            BasicRollingFileAppender::new(tempdir.path().join("test-basic.log"), condition, max_files).unwrap();
        Context {
            _tempdir: tempdir,
            rolling,
        }
    }

    #[test]
    fn frequency_every_day() {
        let mut c = build_context(RollingConditionBasic::new().daily(), 9);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Local.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Local.ymd(2021, 3, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Local.ymd(2021, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &Local.ymd(2022, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        c.verify_contains("Line 1", 3);
        c.verify_contains("Line 2", 3);
        c.verify_contains("Line 3", 2);
        c.verify_contains("Line 4", 1);
        c.verify_contains("Line 5", 0);
    }

    #[test]
    fn frequency_every_day_limited_files() {
        let mut c = build_context(RollingConditionBasic::new().daily(), 2);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Local.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Local.ymd(2021, 3, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Local.ymd(2021, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &Local.ymd(2022, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("Line 3", 2);
        c.verify_contains("Line 4", 1);
        c.verify_contains("Line 5", 0);
    }

    #[test]
    fn frequency_every_hour() {
        let mut c = build_context(RollingConditionBasic::new().hourly(), 9);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Local.ymd(2021, 3, 30).and_hms(1, 3, 2))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Local.ymd(2021, 3, 30).and_hms(2, 1, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Local.ymd(2021, 3, 31).and_hms(2, 1, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("Line 1", 2);
        c.verify_contains("Line 2", 2);
        c.verify_contains("Line 3", 1);
        c.verify_contains("Line 4", 0);
    }

    #[test]
    fn frequency_every_minute() {
        let mut c = build_context(RollingConditionBasic::new().frequency(RollingFrequency::EveryMinute), 9);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Local.ymd(2021, 3, 30).and_hms(1, 2, 4))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Local.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &Local.ymd(2021, 3, 30).and_hms(2, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 6\n", &Local.ymd(2022, 3, 30).and_hms(2, 3, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        c.verify_contains("Line 1", 3);
        c.verify_contains("Line 2", 3);
        c.verify_contains("Line 3", 3);
        c.verify_contains("Line 4", 2);
        c.verify_contains("Line 5", 1);
        c.verify_contains("Line 6", 0);
    }

    #[test]
    fn max_size() {
        let mut c = build_context(RollingConditionBasic::new().max_size(10), 9);
        c.rolling
            .write_with_datetime(b"12345", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"6789", &Local.ymd(2021, 3, 30).and_hms(1, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &Local.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &Local.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &Local.ymd(2022, 3, 31).and_hms(1, 2, 3))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("1234567890", 2);
        c.verify_contains("abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }

    #[test]
    fn max_size_existing() {
        let mut c = build_context(RollingConditionBasic::new().max_size(10), 9);
        c.rolling
            .write_with_datetime(b"12345", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        // close the file and make sure that it can re-open it, and that it
        // resets the file size properly.
        c.rolling.writer_opt.take();
        c.rolling.current_filesize = 0;
        c.rolling
            .write_with_datetime(b"6789", &Local.ymd(2021, 3, 30).and_hms(1, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &Local.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &Local.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &Local.ymd(2022, 3, 31).and_hms(1, 2, 3))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("1234567890", 2);
        c.verify_contains("abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }

    #[test]
    fn daily_and_max_size() {
        let mut c = build_context(RollingConditionBasic::new().daily().max_size(10), 9);
        c.rolling
            .write_with_datetime(b"12345", &Local.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"6789", &Local.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &Local.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &Local.ymd(2021, 3, 31).and_hms(3, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &Local.ymd(2021, 3, 31).and_hms(4, 4, 4))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("123456789", 2);
        c.verify_contains("0abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }
}

#[cfg(test)]
mod test_basic_utc_rolling_appender {
    use chrono::TimeZone;

    use crate::rolling_frequency::RollingFrequency;

    use super::*;
    use std::{fs, io::Write, path::Path};

    struct Context {
        _tempdir: tempfile::TempDir,
        rolling: BasicRollingFileAppenderUtc,
    }

    impl Context {
        fn verify_contains(&mut self, needle: &str, n: usize) {
            self.rolling.flush().unwrap();
            let p = self.rolling.filename_for(n);
            let haystack = fs::read_to_string(&p).unwrap();
            if !haystack.contains(needle) {
                panic!("file {:?} did not contain expected contents {}", p, needle);
            }
        }
    }

    fn build_context(condition: RollingConditionBasicUtc, max_files: usize) -> Context {
        let tempdir = tempfile::tempdir().unwrap();
        let rolling =
            BasicRollingFileAppenderUtc::new(tempdir.path().join("test-utc.log"), condition, max_files).unwrap();
        Context {
            _tempdir: tempdir,
            rolling,
        }
    }

    #[test]
    fn frequency_every_day() {
        let mut c = build_context(RollingConditionBasicUtc::new().daily(), 9);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Utc.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Utc.ymd(2021, 3, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Utc.ymd(2021, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &Utc.ymd(2022, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        c.verify_contains("Line 1", 3);
        c.verify_contains("Line 2", 3);
        c.verify_contains("Line 3", 2);
        c.verify_contains("Line 4", 1);
        c.verify_contains("Line 5", 0);
    }

    #[test]
    fn frequency_every_day_limited_files() {
        let mut c = build_context(RollingConditionBasicUtc::new().daily(), 2);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Utc.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Utc.ymd(2021, 3, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Utc.ymd(2021, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &Utc.ymd(2022, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("Line 3", 2);
        c.verify_contains("Line 4", 1);
        c.verify_contains("Line 5", 0);
    }

    #[test]
    fn frequency_every_hour() {
        let mut c = build_context(RollingConditionBasicUtc::new().hourly(), 9);
        c.rolling
            .write_with_datetime(b"Line 1\n", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Utc.ymd(2021, 3, 30).and_hms(1, 3, 2))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Utc.ymd(2021, 3, 30).and_hms(2, 1, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Utc.ymd(2021, 3, 31).and_hms(2, 1, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("Line 1", 2);
        c.verify_contains("Line 2", 2);
        c.verify_contains("Line 3", 1);
        c.verify_contains("Line 4", 0);
    }

    #[test]
    fn frequency_every_minute() {
        let mut c = build_context(
            RollingConditionBasicUtc::new().frequency(RollingFrequency::EveryMinute),
            9,
        );
        c.rolling
            .write_with_datetime(b"Line 1\n", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 4))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &Utc.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &Utc.ymd(2021, 3, 30).and_hms(2, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 6\n", &Utc.ymd(2022, 3, 30).and_hms(2, 3, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        c.verify_contains("Line 1", 3);
        c.verify_contains("Line 2", 3);
        c.verify_contains("Line 3", 3);
        c.verify_contains("Line 4", 2);
        c.verify_contains("Line 5", 1);
        c.verify_contains("Line 6", 0);
    }

    #[test]
    fn max_size() {
        let mut c = build_context(RollingConditionBasicUtc::new().max_size(10), 9);
        c.rolling
            .write_with_datetime(b"12345", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"6789", &Utc.ymd(2021, 3, 30).and_hms(1, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &Utc.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &Utc.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &Utc.ymd(2022, 3, 31).and_hms(1, 2, 3))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("1234567890", 2);
        c.verify_contains("abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }

    #[test]
    fn max_size_existing() {
        let mut c = build_context(RollingConditionBasicUtc::new().max_size(10), 9);
        c.rolling
            .write_with_datetime(b"12345", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        // close the file and make sure that it can re-open it, and that it
        // resets the file size properly.
        c.rolling.writer_opt.take();
        c.rolling.current_filesize = 0;
        c.rolling
            .write_with_datetime(b"6789", &Utc.ymd(2021, 3, 30).and_hms(1, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &Utc.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &Utc.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &Utc.ymd(2022, 3, 31).and_hms(1, 2, 3))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("1234567890", 2);
        c.verify_contains("abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }

    #[test]
    fn daily_and_max_size() {
        let mut c = build_context(RollingConditionBasicUtc::new().daily().max_size(10), 9);
        c.rolling
            .write_with_datetime(b"12345", &Utc.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"6789", &Utc.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &Utc.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &Utc.ymd(2021, 3, 31).and_hms(3, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &Utc.ymd(2021, 3, 31).and_hms(4, 4, 4))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("123456789", 2);
        c.verify_contains("0abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }
}

#[cfg(test)]
mod test_generic_rolling_appender_with_local_time {
    use chrono::TimeZone;

    use crate::rolling_frequency::RollingFrequency;

    use super::*;
    use std::{fs, io::Write, path::Path};

    struct Context {
        _tempdir: tempfile::TempDir,
        rolling: BasicRollingFileAppenderGeneric<Local>,
    }

    impl Context {
        fn verify_contains(&mut self, needle: &str, n: usize) {
            self.rolling.flush().unwrap();
            let p = self.rolling.filename_for(n);
            let haystack = fs::read_to_string(&p).unwrap();
            if !haystack.contains(needle) {
                panic!("file {:?} did not contain expected contents {}", p, needle);
            }
        }
    }

    fn build_context(condition: RollingConditionBasic, max_files: usize) -> Context {
        let tempdir = tempfile::tempdir().unwrap();
        let rolling =
            BasicRollingFileAppenderGeneric::new(tempdir.path().join("test-generic.log"), condition, max_files, Local)
                .unwrap();
        Context {
            _tempdir: tempdir,
            rolling,
        }
    }

    #[test]
    fn frequency_every_day() {
        let mut c = build_context(RollingConditionBasicGeneric::new().daily(), 9);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"Line 1\n", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &timezone.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &timezone.ymd(2021, 3, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &timezone.ymd(2021, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &timezone.ymd(2022, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        c.verify_contains("Line 1", 3);
        c.verify_contains("Line 2", 3);
        c.verify_contains("Line 3", 2);
        c.verify_contains("Line 4", 1);
        c.verify_contains("Line 5", 0);
    }

    #[test]
    fn frequency_every_day_limited_files() {
        let mut c = build_context(RollingConditionBasic::new().daily(), 2);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"Line 1\n", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &timezone.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &timezone.ymd(2021, 3, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &timezone.ymd(2021, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &timezone.ymd(2022, 5, 31).and_hms(1, 4, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("Line 3", 2);
        c.verify_contains("Line 4", 1);
        c.verify_contains("Line 5", 0);
    }

    #[test]
    fn frequency_every_hour() {
        let mut c = build_context(RollingConditionBasic::new().hourly(), 9);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"Line 1\n", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &timezone.ymd(2021, 3, 30).and_hms(1, 3, 2))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &timezone.ymd(2021, 3, 30).and_hms(2, 1, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &timezone.ymd(2021, 3, 31).and_hms(2, 1, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("Line 1", 2);
        c.verify_contains("Line 2", 2);
        c.verify_contains("Line 3", 1);
        c.verify_contains("Line 4", 0);
    }

    #[test]
    fn frequency_every_minute() {
        let mut c = build_context(RollingConditionBasic::new().frequency(RollingFrequency::EveryMinute), 9);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"Line 1\n", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 2\n", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 3\n", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 4))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 4\n", &timezone.ymd(2021, 3, 30).and_hms(1, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 5\n", &timezone.ymd(2021, 3, 30).and_hms(2, 3, 0))
            .unwrap();
        c.rolling
            .write_with_datetime(b"Line 6\n", &timezone.ymd(2022, 3, 30).and_hms(2, 3, 0))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(4)).exists(), false);
        c.verify_contains("Line 1", 3);
        c.verify_contains("Line 2", 3);
        c.verify_contains("Line 3", 3);
        c.verify_contains("Line 4", 2);
        c.verify_contains("Line 5", 1);
        c.verify_contains("Line 6", 0);
    }

    #[test]
    fn max_size() {
        let mut c = build_context(RollingConditionBasic::new().max_size(10), 9);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"12345", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"6789", &timezone.ymd(2021, 3, 30).and_hms(1, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &timezone.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &timezone.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &timezone.ymd(2022, 3, 31).and_hms(1, 2, 3))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("1234567890", 2);
        c.verify_contains("abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }

    #[test]
    fn max_size_existing() {
        let mut c = build_context(RollingConditionBasic::new().max_size(10), 9);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"12345", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        // close the file and make sure that it can re-open it, and that it
        // resets the file size properly.
        c.rolling.writer_opt.take();
        c.rolling.current_filesize = 0;
        c.rolling
            .write_with_datetime(b"6789", &timezone.ymd(2021, 3, 30).and_hms(1, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &timezone.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &timezone.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &timezone.ymd(2022, 3, 31).and_hms(1, 2, 3))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("1234567890", 2);
        c.verify_contains("abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }

    #[test]
    fn daily_and_max_size() {
        let mut c = build_context(RollingConditionBasic::new().daily().max_size(10), 9);
        let timezone = c.rolling.time_zone.clone();
        c.rolling
            .write_with_datetime(b"12345", &timezone.ymd(2021, 3, 30).and_hms(1, 2, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"6789", &timezone.ymd(2021, 3, 30).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"0", &timezone.ymd(2021, 3, 31).and_hms(2, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"abcdefghijklmn", &timezone.ymd(2021, 3, 31).and_hms(3, 3, 3))
            .unwrap();
        c.rolling
            .write_with_datetime(b"ZZZ", &timezone.ymd(2021, 3, 31).and_hms(4, 4, 4))
            .unwrap();
        assert_eq!(AsRef::<Path>::as_ref(&c.rolling.filename_for(3)).exists(), false);
        c.verify_contains("123456789", 2);
        c.verify_contains("0abcdefghijklmn", 1);
        c.verify_contains("ZZZ", 0);
    }
}
// LCOV_EXCL_STOP
