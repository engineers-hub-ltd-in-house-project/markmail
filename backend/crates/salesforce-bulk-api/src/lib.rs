//! # Salesforce Bulk API 2.0 Client for Rust
//!
//! A Rust client library for interacting with Salesforce Bulk API 2.0.
//! This library provides an async, type-safe interface for performing bulk operations
//! on Salesforce objects.
//!
//! ## Features
//!
//! - Async/await support with Tokio
//! - Type-safe job creation and management
//! - CSV data upload and download
//! - Progress tracking for long-running operations
//! - Comprehensive error handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use salesforce_bulk_api::{BulkClient, JobConfig, Operation};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = BulkClient::new(
//!         "https://myinstance.salesforce.com",
//!         "your_access_token"
//!     )?;
//!
//!     // Create a bulk insert job
//!     let job_config = JobConfig::new("Contact", Operation::Insert);
//!     let job = client.create_job(job_config).await?;
//!
//!     // Upload CSV data
//!     let csv_data = "FirstName,LastName,Email\nJohn,Doe,john@example.com";
//!     client.upload_data(&job.id, csv_data).await?;
//!
//!     // Start the job
//!     let job = client.start_job(&job.id).await?;
//!
//!     // Monitor progress
//!     let job = client.wait_for_completion(&job.id).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod job;
pub mod models;
pub mod utils;

pub use client::BulkClient;
pub use error::{BulkApiError, Result};
pub use job::{Job, JobConfig, JobState};
pub use models::{Operation, LineEnding, ColumnDelimiter, ContentType};

#[cfg(test)]
mod tests;