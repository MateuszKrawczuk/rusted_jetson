// SPDX-License-Identifier: LGPL-3.0
// Copyright (C) 2026 Mateusz Krawczuk with work <m.krawczuk@cybrixsystems.com>

//! Error types for rusted-jetsons

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    HardwareNotFound(String),
    PermissionDenied(String),
    UnsupportedPlatform(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::HardwareNotFound(s) => write!(f, "Hardware not found: {}", s),
            Error::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            Error::UnsupportedPlatform(s) => write!(f, "Unsupported platform: {}", s),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        let display = format!("{}", err);
        assert!(display.contains("I/O error"));
    }

    #[test]
    fn test_error_display_hardware_not_found() {
        let err = Error::HardwareNotFound("GPU".to_string());
        let display = format!("{}", err);
        assert_eq!(display, "Hardware not found: GPU");
    }

    #[test]
    fn test_error_display_permission_denied() {
        let err = Error::PermissionDenied("/sys/class/thermal".to_string());
        let display = format!("{}", err);
        assert_eq!(display, "Permission denied: /sys/class/thermal");
    }

    #[test]
    fn test_error_display_unsupported_platform() {
        let err = Error::UnsupportedPlatform("x86_64".to_string());
        let display = format!("{}", err);
        assert_eq!(display, "Unsupported platform: x86_64");
    }

    #[test]
    fn test_error_source_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::Io(io_err);
        assert!(StdError::source(&err).is_some());
    }

    #[test]
    fn test_error_source_other() {
        let err = Error::HardwareNotFound("GPU".to_string());
        assert!(StdError::source(&err).is_none());

        let err = Error::PermissionDenied("/sys".to_string());
        assert!(StdError::source(&err).is_none());

        let err = Error::UnsupportedPlatform("x86".to_string());
        assert!(StdError::source(&err).is_none());
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_debug() {
        let err = Error::HardwareNotFound("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("HardwareNotFound"));
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: Result<i32> = Err(Error::HardwareNotFound("test".to_string()));
        assert!(err_result.is_err());
    }
}
