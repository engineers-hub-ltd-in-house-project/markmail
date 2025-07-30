//! Data models for Salesforce Bulk API 2.0

use serde::{Deserialize, Serialize};
use std::fmt;

/// Bulk API operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    /// Insert new records
    Insert,
    /// Update existing records
    Update,
    /// Update if exists, insert if not
    Upsert,
    /// Delete records
    Delete,
    /// Hard delete records (bypasses recycle bin)
    HardDelete,
    /// Query records
    Query,
    /// Query all records (including deleted)
    QueryAll,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Insert => write!(f, "insert"),
            Operation::Update => write!(f, "update"),
            Operation::Upsert => write!(f, "upsert"),
            Operation::Delete => write!(f, "delete"),
            Operation::HardDelete => write!(f, "hardDelete"),
            Operation::Query => write!(f, "query"),
            Operation::QueryAll => write!(f, "queryAll"),
        }
    }
}

/// Line ending format for CSV files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LineEnding {
    /// Line feed (Unix)
    LF,
    /// Carriage return + line feed (Windows)
    CRLF,
}

impl Default for LineEnding {
    fn default() -> Self {
        LineEnding::LF
    }
}

/// Column delimiter for CSV files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColumnDelimiter {
    /// Comma
    COMMA,
    /// Tab
    TAB,
    /// Pipe
    PIPE,
    /// Semicolon
    SEMICOLON,
    /// Caret
    CARET,
    /// Backquote
    BACKQUOTE,
}

impl Default for ColumnDelimiter {
    fn default() -> Self {
        ColumnDelimiter::COMMA
    }
}

/// Content type for data upload
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// CSV format
    Csv,
    /// JSON format (future support)
    Json,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Csv => write!(f, "text/csv"),
            ContentType::Json => write!(f, "application/json"),
        }
    }
}

/// API version for Salesforce
#[derive(Debug, Clone)]
pub struct ApiVersion(pub String);

impl ApiVersion {
    /// Create a new API version
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Latest stable API version as of 2025
    pub fn latest() -> Self {
        Self("v62.0".to_string())
    }
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::latest()
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Salesforce instance URL
#[derive(Debug, Clone)]
pub struct InstanceUrl(pub String);

impl InstanceUrl {
    /// Create a new instance URL
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        // Remove trailing slash if present
        let url = url.trim_end_matches('/');
        Self(url.to_string())
    }

    /// Get the base URL for API calls
    pub fn api_base(&self, version: &ApiVersion) -> String {
        format!("{}/services/data/{}", self.0, version)
    }

    /// Get the Bulk API 2.0 base URL
    pub fn bulk_api_base(&self, version: &ApiVersion) -> String {
        format!("{}/jobs/ingest", self.api_base(version))
    }

    /// Get the query job base URL
    pub fn query_api_base(&self, version: &ApiVersion) -> String {
        format!("{}/jobs/query", self.api_base(version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_display() {
        assert_eq!(Operation::Insert.to_string(), "insert");
        assert_eq!(Operation::Upsert.to_string(), "upsert");
        assert_eq!(Operation::Query.to_string(), "query");
    }

    #[test]
    fn test_instance_url() {
        let url = InstanceUrl::new("https://myinstance.salesforce.com/");
        let version = ApiVersion::new("v62.0");
        
        assert_eq!(url.0, "https://myinstance.salesforce.com");
        assert_eq!(
            url.api_base(&version),
            "https://myinstance.salesforce.com/services/data/v62.0"
        );
        assert_eq!(
            url.bulk_api_base(&version),
            "https://myinstance.salesforce.com/services/data/v62.0/jobs/ingest"
        );
    }

    #[test]
    fn test_content_type_display() {
        assert_eq!(ContentType::Csv.to_string(), "text/csv");
        assert_eq!(ContentType::Json.to_string(), "application/json");
    }
}