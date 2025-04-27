//! # Logging Module
//!
//! Provides a trait and implementation for user event logging and statistics.

use std::fs::{OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// Logger trait for logging user events and statistics
pub trait Logger {
    /// Log a generic event string
    fn log_event(&mut self, event: &str);
    /// Log a text entry event
    fn log_text(&mut self, text: &str);
    /// Log a statistic (e.g., stat name and value)
    fn log_stat(&mut self, stat: &str, value: usize);
}

/// File-based logger implementation
pub struct FileLogger {
    file_path: String,
}

impl FileLogger {
    /// Create a new FileLogger writing to the given file path
    pub fn new(file_path: &str) -> Self {
        Self { file_path: file_path.to_string() }
    }

    fn write_line(&self, line: &str) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.file_path) {
            let _ = writeln!(file, "{}", line);
        }
    }

    fn timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
    }
}

impl Logger for FileLogger {
    fn log_event(&mut self, event: &str) {
        let ts = Self::timestamp();
        self.write_line(&format!("[{}] EVENT: {}", ts, event));
    }
    fn log_text(&mut self, text: &str) {
        let ts = Self::timestamp();
        self.write_line(&format!("[{}] TEXT: {}", ts, text));
    }
    fn log_stat(&mut self, stat: &str, value: usize) {
        let ts = Self::timestamp();
        self.write_line(&format!("[{}] STAT: {} = {}", ts, stat, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_logger_writes_log() {
        let path = "test_dasher.log";
        let mut logger = FileLogger::new(path);
        logger.log_event("test_event");
        logger.log_text("hello world");
        logger.log_stat("chars_entered", 42);
        let contents = fs::read_to_string(path).unwrap();
        assert!(contents.contains("EVENT: test_event"));
        assert!(contents.contains("TEXT: hello world"));
        assert!(contents.contains("STAT: chars_entered = 42"));
        let _ = fs::remove_file(path);
    }
}
