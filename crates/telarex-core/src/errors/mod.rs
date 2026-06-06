use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum ErrorLevel {
    #[error("INFO")]
    Info,
    #[error("WARN")]
    Warning,
    #[error("ERROR")]
    Error,
    #[error("FATAL")]
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("[{code}] {level}: {message}")]
pub struct TrexError {
    pub code: String,
    pub level: ErrorLevel,
    pub message: String,
    pub solution: String,
}

impl TrexError {
    pub fn new(code: &str, level: ErrorLevel, message: &str, solution: &str) -> Self {
        Self {
            code: code.to_string(),
            level,
            message: message.to_string(),
            solution: solution.to_string(),
        }
    }

    pub fn file_not_found(path: &str) -> Self {
        Self::new("TRX-F01", ErrorLevel::Error, &format!("File not found: {}", path), "Check the path and permissions.")
    }

    pub fn network_failure(details: &str) -> Self {
        Self::new("TRX-N01", ErrorLevel::Warning, &format!("Network error: {}", details), "Check your internet connection or bootstrap nodes.")
    }

    pub fn auth_failed() -> Self {
        Self::new("TRX-A01", ErrorLevel::Error, "ZK-Authentication failed.", "Ensure you have the correct Lodge Key.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_level_display() {
        assert_eq!(ErrorLevel::Info.to_string(), "INFO");
        assert_eq!(ErrorLevel::Warning.to_string(), "WARN");
        assert_eq!(ErrorLevel::Error.to_string(), "ERROR");
        assert_eq!(ErrorLevel::Fatal.to_string(), "FATAL");
    }

    #[test]
    fn test_trex_error_format() {
        let err = TrexError::file_not_found("/path/to/file.rs");
        let msg = err.to_string();
        assert!(msg.contains("TRX-F01"));
        assert!(msg.contains("ERROR"));
        assert!(msg.contains("File not found"));
    }

    #[test]
    fn test_file_not_found() {
        let err = TrexError::file_not_found("test.txt");
        assert_eq!(err.code, "TRX-F01");
        assert_eq!(err.level.to_string(), "ERROR");
    }

    #[test]
    fn test_network_failure() {
        let err = TrexError::network_failure("timeout");
        assert_eq!(err.code, "TRX-N01");
        assert_eq!(err.level.to_string(), "WARN");
    }

    #[test]
    fn test_auth_failed() {
        let err = TrexError::auth_failed();
        assert_eq!(err.code, "TRX-A01");
        assert_eq!(err.level.to_string(), "ERROR");
        assert_eq!(err.message, "ZK-Authentication failed.");
    }
}
