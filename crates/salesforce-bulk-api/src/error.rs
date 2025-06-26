//! Error types for the Salesforce Bulk API client

use thiserror::Error;

/// Result type alias for Bulk API operations
pub type Result<T> = std::result::Result<T, BulkApiError>;

/// Main error type for Salesforce Bulk API operations
#[derive(Debug, Error)]
pub enum BulkApiError {
    /// HTTP request errors
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// CSV processing errors
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    /// Invalid job configuration
    #[error("Invalid job configuration: {0}")]
    InvalidJobConfig(String),

    /// Job failed
    #[error("Job failed: {0}")]
    JobFailed(String),

    /// Job aborted
    #[error("Job was aborted")]
    JobAborted,

    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// API error from Salesforce
    #[error("Salesforce API error: {message}")]
    ApiError {
        message: String,
        error_code: Option<String>,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Retry after: {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },

    /// Invalid response
    #[error("Invalid response from server: {0}")]
    InvalidResponse(String),

    /// Timeout error
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl BulkApiError {
    /// Create a new API error
    pub fn api_error(message: impl Into<String>, error_code: Option<String>) -> Self {
        Self::ApiError {
            message: message.into(),
            error_code,
        }
    }

    /// Create a new job configuration error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidJobConfig(message.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::HttpError(_) | Self::RateLimitExceeded { .. } | Self::Timeout(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = BulkApiError::JobFailed("Test failure".to_string());
        assert_eq!(error.to_string(), "Job failed: Test failure");
    }

    #[test]
    fn test_is_retryable() {
        assert!(BulkApiError::RateLimitExceeded { retry_after: 60 }.is_retryable());
        assert!(BulkApiError::Timeout(30).is_retryable());
        assert!(!BulkApiError::JobAborted.is_retryable());
        assert!(!BulkApiError::InvalidJobConfig("test".to_string()).is_retryable());
    }
}