//! Job management for Salesforce Bulk API 2.0

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{ColumnDelimiter, LineEnding, Operation};

/// Job state in the Bulk API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum JobState {
    /// Job is open and ready to receive data
    Open,
    /// Upload phase is complete, job is ready to be processed
    UploadComplete,
    /// Job is being processed
    InProgress,
    /// Job has been aborted
    Aborted,
    /// Job completed successfully
    JobComplete,
    /// Job failed
    Failed,
}

impl JobState {
    /// Check if job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobState::Aborted | JobState::JobComplete | JobState::Failed
        )
    }

    /// Check if job is in progress
    pub fn is_processing(&self) -> bool {
        matches!(self, JobState::InProgress | JobState::UploadComplete)
    }
}

/// Configuration for creating a new bulk job
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobConfig {
    /// The Salesforce object type (e.g., "Contact", "Account")
    pub object: String,
    /// The operation to perform
    pub operation: Operation,
    /// External ID field name (required for upsert)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id_field_name: Option<String>,
    /// Line ending for CSV files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_ending: Option<LineEnding>,
    /// Column delimiter for CSV files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_delimiter: Option<ColumnDelimiter>,
}

impl JobConfig {
    /// Create a new job configuration
    pub fn new(object: impl Into<String>, operation: Operation) -> Self {
        Self {
            object: object.into(),
            operation,
            external_id_field_name: None,
            line_ending: None,
            column_delimiter: None,
        }
    }

    /// Set external ID field for upsert operations
    pub fn with_external_id(mut self, field_name: impl Into<String>) -> Self {
        self.external_id_field_name = Some(field_name.into());
        self
    }

    /// Set line ending format
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = Some(line_ending);
        self
    }

    /// Set column delimiter
    pub fn with_column_delimiter(mut self, delimiter: ColumnDelimiter) -> Self {
        self.column_delimiter = Some(delimiter);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.object.is_empty() {
            return Err(crate::BulkApiError::invalid_config("Object name cannot be empty"));
        }

        if self.operation == Operation::Upsert && self.external_id_field_name.is_none() {
            return Err(crate::BulkApiError::invalid_config(
                "External ID field is required for upsert operations",
            ));
        }

        Ok(())
    }
}

/// Represents a Bulk API job
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Unique job ID
    pub id: String,
    /// Job state
    pub state: JobState,
    /// Salesforce object type
    pub object: String,
    /// Operation being performed
    pub operation: Operation,
    /// External ID field name (for upsert)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id_field_name: Option<String>,
    /// Created date
    pub created_date: DateTime<Utc>,
    /// System modified date
    pub system_modstamp: DateTime<Utc>,
    /// Content type
    pub content_type: String,
    /// API version
    pub api_version: String,
    /// Created by user ID
    pub created_by_id: String,
    /// Job type (always "BigObjectIngest" for Bulk API 2.0)
    pub job_type: String,
    /// Line ending format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_ending: Option<LineEnding>,
    /// Column delimiter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_delimiter: Option<ColumnDelimiter>,
    /// Number of records processed
    #[serde(default)]
    pub number_records_processed: u64,
    /// Number of records failed
    #[serde(default)]
    pub number_records_failed: u64,
    /// Retry limit
    #[serde(default)]
    pub retry_limit: u32,
}

impl Job {
    /// Check if the job has completed (successfully or with failure)
    pub fn is_complete(&self) -> bool {
        self.state.is_terminal()
    }

    /// Check if the job is ready to receive data
    pub fn is_open(&self) -> bool {
        self.state == JobState::Open
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.number_records_processed == 0 {
            return 0.0;
        }
        let successful = self.number_records_processed - self.number_records_failed;
        (successful as f64 / self.number_records_processed as f64) * 100.0
    }
}

/// Job creation response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobCreateResponse {
    /// Job ID
    pub id: String,
    /// Job state
    pub state: JobState,
    /// Object type
    pub object: String,
    /// Created date
    pub created_date: DateTime<Utc>,
    /// Content type
    pub content_type: String,
}

/// Job info for listing
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    /// Job ID
    pub id: String,
    /// Operation
    pub operation: Operation,
    /// Object type
    pub object: String,
    /// Created date
    pub created_date: DateTime<Utc>,
    /// Job state
    pub state: JobState,
}

/// Job list response
#[derive(Debug, Deserialize)]
pub struct JobListResponse {
    /// Done flag
    pub done: bool,
    /// Next records URL
    #[serde(rename = "nextRecordsUrl")]
    pub next_records_url: Option<String>,
    /// Job records
    pub records: Vec<JobInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_config_validation() {
        // Valid insert job
        let config = JobConfig::new("Contact", Operation::Insert);
        assert!(config.validate().is_ok());

        // Invalid: empty object
        let config = JobConfig::new("", Operation::Insert);
        assert!(config.validate().is_err());

        // Invalid: upsert without external ID
        let config = JobConfig::new("Contact", Operation::Upsert);
        assert!(config.validate().is_err());

        // Valid: upsert with external ID
        let config = JobConfig::new("Contact", Operation::Upsert)
            .with_external_id("External_Id__c");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_job_state() {
        assert!(JobState::JobComplete.is_terminal());
        assert!(JobState::Failed.is_terminal());
        assert!(JobState::Aborted.is_terminal());
        assert!(!JobState::Open.is_terminal());
        assert!(!JobState::InProgress.is_terminal());

        assert!(JobState::InProgress.is_processing());
        assert!(JobState::UploadComplete.is_processing());
        assert!(!JobState::Open.is_processing());
    }

    #[test]
    fn test_job_success_rate() {
        let mut job = Job {
            id: "test".to_string(),
            state: JobState::JobComplete,
            object: "Contact".to_string(),
            operation: Operation::Insert,
            external_id_field_name: None,
            created_date: Utc::now(),
            system_modstamp: Utc::now(),
            content_type: "CSV".to_string(),
            api_version: "v62.0".to_string(),
            created_by_id: "user123".to_string(),
            job_type: "BigObjectIngest".to_string(),
            line_ending: None,
            column_delimiter: None,
            number_records_processed: 100,
            number_records_failed: 5,
            retry_limit: 0,
        };

        assert_eq!(job.success_rate(), 95.0);

        job.number_records_processed = 0;
        assert_eq!(job.success_rate(), 0.0);
    }
}