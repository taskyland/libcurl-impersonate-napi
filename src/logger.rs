use std::sync::atomic::{AtomicBool, Ordering};

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
  Debug,
  Info,
  Warn,
  Error,
}

/// Global flag to enable/disable logging
static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);
/// Current log level
static mut CURRENT_LOG_LEVEL: LogLevel = LogLevel::Info;

/// Enable log output
pub fn enable_logging(enable: bool) {
  LOGGING_ENABLED.store(enable, Ordering::SeqCst);
}

/// Set log level
pub fn set_log_level(level: LogLevel) {
  unsafe {
    CURRENT_LOG_LEVEL = level;
  }
}

/// Get current log level
pub fn get_log_level() -> LogLevel {
  unsafe { CURRENT_LOG_LEVEL }
}

/// Check if logging is enabled
pub fn is_logging_enabled() -> bool {
  LOGGING_ENABLED.load(Ordering::SeqCst)
}

/// Determine if logs of the specified level should be output
pub fn should_log(level: LogLevel) -> bool {
  if !is_logging_enabled() {
    return false;
  }

  matches!(
    (unsafe { CURRENT_LOG_LEVEL }, level),
    (LogLevel::Debug, _)
      | (
        LogLevel::Info,
        LogLevel::Info | LogLevel::Warn | LogLevel::Error
      )
      | (LogLevel::Warn, LogLevel::Warn | LogLevel::Error)
      | (LogLevel::Error, LogLevel::Error)
  )
}

/// Record debug log
pub fn log_debug(module: &str, message: &str) {
  if should_log(LogLevel::Debug) {
    println!("[DEBUG][{}] {}", module, message);
  }
}

/// Record info log
pub fn log_info(module: &str, message: &str) {
  if should_log(LogLevel::Info) {
    println!("[INFO][{}] {}", module, message);
  }
}

/// Record warning log
pub fn log_warn(module: &str, message: &str) {
  if should_log(LogLevel::Warn) {
    println!("[WARN][{}] {}", module, message);
  }
}

/// Record error log
pub fn log_error(module: &str, message: &str) {
  if should_log(LogLevel::Error) {
    eprintln!("[ERROR][{}] {}", module, message);
  }
}

// Convenience macros that support formatted output
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Debug) {
            $crate::logger::log_debug($module, &format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_info{
    ($module:expr, $($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Info) {
            $crate::logger::log_info($module, &format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_warn{
    ($module:expr, $($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Warn) {
            $crate::logger::log_warn($module, &format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_error{
    ($module:expr, $($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Error) {
            $crate::logger::log_error($module, &format!($($arg)*));
        }
    };
}

