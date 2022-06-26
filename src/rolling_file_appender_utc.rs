use chrono::{DateTime, Utc};
use std::{
    convert::TryFrom,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io,
    io::{BufWriter, Write},
    path::Path,
};

use crate::rolling_condition::RollingCondition;

/// Writes data to a file, and "rolls over" to preserve older data in
/// a separate set of files. Old files have a Debian-style naming scheme
/// where we have base_filename, base_filename.1, ..., base_filename.N
/// where N is the maximum number of rollover files to keep.
#[derive(Debug)]
pub struct RollingFileAppenderUtc<RC>
where
    RC: RollingCondition<Utc>,
{
    condition: RC,
    base_filename: OsString,
    max_files: usize,
    pub(crate) current_filesize: u64,
    pub(crate) writer_opt: Option<BufWriter<File>>,
}

impl<RC> RollingFileAppenderUtc<RC>
where
    RC: RollingCondition<Utc>,
{
    /// Creates a new rolling file appender with the given condition.
    /// The parent directory of the base path must already exist.
    pub fn new<P>(path: P, condition: RC, max_files: usize) -> io::Result<RollingFileAppenderUtc<RC>>
    where
        P: AsRef<Path>,
    {
        let mut rfa = RollingFileAppenderUtc {
            condition,
            base_filename: path.as_ref().as_os_str().to_os_string(),
            max_files,
            current_filesize: 0,
            writer_opt: None,
        };
        // Fail if we can't open the file initially...
        rfa.open_writer_if_needed()?;
        Ok(rfa)
    }

    /// Determines the final filename, where n==0 indicates the current file
    pub(crate) fn filename_for(&self, n: usize) -> OsString {
        let mut f = self.base_filename.clone();
        if n > 0 {
            f.push(OsString::from(format!(".{}", n)))
        }
        f
    }

    /// Rotates old files to make room for a new one.
    /// This may result in the deletion of the oldest file
    fn rotate_files(&mut self) -> io::Result<()> {
        // ignore any failure removing the oldest file (may not exist)
        let _ = fs::remove_file(self.filename_for(self.max_files.max(1)));
        let mut r = Ok(());
        for i in (0..self.max_files.max(1)).rev() {
            let rotate_from = self.filename_for(i);
            let rotate_to = self.filename_for(i + 1);
            if let Err(e) = fs::rename(&rotate_from, &rotate_to).or_else(|e| match e.kind() {
                io::ErrorKind::NotFound => Ok(()),
                _ => Err(e),
            }) {
                // capture the error, but continue the loop,
                // to maximize ability to rename everything
                r = Err(e);
            }
        }
        r
    }

    /// Forces a rollover to happen immediately.
    pub fn rollover(&mut self) -> io::Result<()> {
        // Before closing, make sure all data is flushed successfully.
        self.flush()?;
        // We must close the current file before rotating files
        self.writer_opt.take();
        self.current_filesize = 0;
        self.rotate_files()?;
        self.open_writer_if_needed()
    }

    /// Opens a writer for the current file.
    fn open_writer_if_needed(&mut self) -> io::Result<()> {
        if self.writer_opt.is_none() {
            let p = self.filename_for(0);
            self.writer_opt = Some(BufWriter::new(OpenOptions::new().append(true).create(true).open(&p)?));
            self.current_filesize = fs::metadata(&p).map_or(0, |m| m.len());
        }
        Ok(())
    }

    /// Writes data using the given datetime to calculate the rolling condition
    pub fn write_with_datetime(&mut self, buf: &[u8], now: &DateTime<Utc>) -> io::Result<usize> {
        if self.condition.should_rollover(&now, self.current_filesize) {
            if let Err(e) = self.rollover() {
                // If we can't rollover, just try to continue writing anyway
                // (better than missing data).
                // This will likely used to implement logging, so
                // avoid using log::warn and log to stderr directly
                eprintln!(
                    "WARNING: Failed to rotate file {}: {}",
                    self.base_filename.to_string_lossy(),
                    e
                );
            }
        }
        self.open_writer_if_needed()?;
        if let Some(writer) = self.writer_opt.as_mut() {
            let buf_len = buf.len();
            writer.write_all(buf).map(|_| {
                self.current_filesize += u64::try_from(buf_len).unwrap_or(u64::MAX);
                buf_len
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "unexpected condition: writer is missing",
            ))
        }
    }
}

impl<RC> io::Write for RollingFileAppenderUtc<RC>
where
    RC: RollingCondition<Utc>,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let now = Utc::now();
        self.write_with_datetime(buf, &now)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(writer) = self.writer_opt.as_mut() {
            writer.flush()?;
        }
        Ok(())
    }
}
