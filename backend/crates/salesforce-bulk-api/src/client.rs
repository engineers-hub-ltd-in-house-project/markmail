//! Salesforce Bulk API 2.0 client implementation

use std::time::Duration;

use reqwest::{header, Client, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::{
    error::{BulkApiError, Result},
    job::{Job, JobConfig, JobCreateResponse, JobListResponse, JobState},
    models::{ApiVersion, ContentType, InstanceUrl, Operation},
};

/// Default timeout for HTTP requests (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default polling interval for job status (5 seconds)
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Maximum number of retries for retryable errors
const MAX_RETRIES: u32 = 3;

/// Salesforce Bulk API 2.0 client
#[derive(Debug, Clone)]
pub struct BulkClient {
    /// HTTP client
    client: Client,
    /// Salesforce instance URL
    instance_url: InstanceUrl,
    /// API version
    api_version: ApiVersion,
    /// Access token
    access_token: String,
}

impl BulkClient {
    /// Create a new Bulk API client
    pub fn new(instance_url: impl Into<String>, access_token: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(|e| BulkApiError::Other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            instance_url: InstanceUrl::new(instance_url),
            api_version: ApiVersion::default(),
            access_token: access_token.into(),
        })
    }

    /// Create a new client with custom API version
    pub fn with_version(
        instance_url: impl Into<String>,
        access_token: impl Into<String>,
        api_version: ApiVersion,
    ) -> Result<Self> {
        let mut client = Self::new(instance_url, access_token)?;
        client.api_version = api_version;
        Ok(client)
    }

    /// Create a new bulk job
    pub async fn create_job(&self, config: JobConfig) -> Result<Job> {
        config.validate()?;

        let url = if matches!(config.operation, Operation::Query | Operation::QueryAll) {
            self.instance_url.query_api_base(&self.api_version)
        } else {
            self.instance_url.bulk_api_base(&self.api_version)
        };

        let response: JobCreateResponse = self
            .post(&url, &config)
            .await?;

        // Fetch full job details
        self.get_job(&response.id).await
    }

    /// Get job details
    pub async fn get_job(&self, job_id: &str) -> Result<Job> {
        let url = format!(
            "{}/{}",
            self.instance_url.bulk_api_base(&self.api_version),
            job_id
        );

        self.get(&url).await
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Result<Vec<Job>> {
        let url = self.instance_url.bulk_api_base(&self.api_version);
        let mut all_jobs = Vec::new();
        let mut next_url = Some(url);

        while let Some(url) = next_url {
            let response: JobListResponse = self.get(&url).await?;
            
            for job_info in response.records {
                // Fetch full details for each job
                if let Ok(job) = self.get_job(&job_info.id).await {
                    all_jobs.push(job);
                }
            }

            next_url = response.next_records_url.map(|path| {
                format!("{}{}", self.instance_url.0, path)
            });

            if response.done {
                break;
            }
        }

        Ok(all_jobs)
    }

    /// Upload data to a job (CSV format)
    pub async fn upload_data(&self, job_id: &str, csv_data: &str) -> Result<()> {
        let url = format!(
            "{}/{}/batches",
            self.instance_url.bulk_api_base(&self.api_version),
            job_id
        );

        let response = self
            .client
            .put(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
            .header(header::CONTENT_TYPE, ContentType::Csv.to_string())
            .body(csv_data.to_string())
            .send()
            .await?;

        self.handle_response::<Value>(response).await?;
        Ok(())
    }

    /// Close/start a job (mark upload as complete)
    pub async fn start_job(&self, job_id: &str) -> Result<Job> {
        self.update_job_state(job_id, JobState::UploadComplete).await
    }

    /// Abort a job
    pub async fn abort_job(&self, job_id: &str) -> Result<Job> {
        self.update_job_state(job_id, JobState::Aborted).await
    }

    /// Update job state
    async fn update_job_state(&self, job_id: &str, state: JobState) -> Result<Job> {
        let url = format!(
            "{}/{}",
            self.instance_url.bulk_api_base(&self.api_version),
            job_id
        );

        let body = serde_json::json!({
            "state": state
        });

        self.patch(&url, &body).await
    }

    /// Wait for job completion with progress updates
    pub async fn wait_for_completion(&self, job_id: &str) -> Result<Job> {
        let mut poll_interval = DEFAULT_POLL_INTERVAL;
        let mut last_processed = 0u64;

        loop {
            let job = self.get_job(job_id).await?;

            if job.number_records_processed > last_processed {
                info!(
                    "Job {}: Processed {} records ({} failed)",
                    job_id, job.number_records_processed, job.number_records_failed
                );
                last_processed = job.number_records_processed;
            }

            match job.state {
                JobState::JobComplete => {
                    info!(
                        "Job {} completed: {} records processed, {} failed ({}% success rate)",
                        job_id,
                        job.number_records_processed,
                        job.number_records_failed,
                        job.success_rate()
                    );
                    return Ok(job);
                }
                JobState::Failed => {
                    return Err(BulkApiError::JobFailed(format!(
                        "Job {} failed: {} records processed, {} failed",
                        job_id, job.number_records_processed, job.number_records_failed
                    )));
                }
                JobState::Aborted => {
                    return Err(BulkApiError::JobAborted);
                }
                _ => {
                    debug!("Job {} state: {:?}", job_id, job.state);
                    sleep(poll_interval).await;
                    
                    // Gradually increase poll interval up to 30 seconds
                    if poll_interval < Duration::from_secs(30) {
                        poll_interval += Duration::from_secs(5);
                    }
                }
            }
        }
    }

    /// Get successful records from a completed job
    pub async fn get_successful_records(&self, job_id: &str) -> Result<String> {
        let url = format!(
            "{}/{}/successfulResults",
            self.instance_url.bulk_api_base(&self.api_version),
            job_id
        );

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Get failed records from a completed job
    pub async fn get_failed_records(&self, job_id: &str) -> Result<String> {
        let url = format!(
            "{}/{}/failedResults",
            self.instance_url.bulk_api_base(&self.api_version),
            job_id
        );

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Get unprocessed records from a completed job
    pub async fn get_unprocessed_records(&self, job_id: &str) -> Result<String> {
        let url = format!(
            "{}/{}/unprocessedrecords",
            self.instance_url.bulk_api_base(&self.api_version),
            job_id
        );

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Execute GET request with retry logic
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        self.execute_with_retry(|| async {
            let response = self
                .client
                .get(url)
                .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
                .send()
                .await?;

            self.handle_response(response).await
        })
        .await
    }

    /// Execute POST request with retry logic
    async fn post<T: DeserializeOwned>(&self, url: &str, body: &impl serde::Serialize) -> Result<T> {
        self.execute_with_retry(|| async {
            let response = self
                .client
                .post(url)
                .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
                .json(body)
                .send()
                .await?;

            self.handle_response(response).await
        })
        .await
    }

    /// Execute PATCH request with retry logic
    async fn patch<T: DeserializeOwned>(&self, url: &str, body: &impl serde::Serialize) -> Result<T> {
        self.execute_with_retry(|| async {
            let response = self
                .client
                .patch(url)
                .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
                .json(body)
                .send()
                .await?;

            self.handle_response(response).await
        })
        .await
    }

    /// Execute request with retry logic
    async fn execute_with_retry<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;
        let mut last_error = None;

        while retries < MAX_RETRIES {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if e.is_retryable() && retries < MAX_RETRIES - 1 {
                        retries += 1;
                        let delay = Duration::from_secs(2u64.pow(retries));
                        warn!("Request failed (attempt {}/{}): {}. Retrying in {:?}...", 
                              retries, MAX_RETRIES, e, delay);
                        sleep(delay).await;
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| BulkApiError::Other("Max retries exceeded".to_string())))
    }

    /// Handle HTTP response
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| BulkApiError::InvalidResponse(format!("Failed to parse response: {}", e)))
        } else {
            Err(self.handle_error_response(response).await)
        }
    }

    /// Handle error response
    async fn handle_error_response(&self, response: reqwest::Response) -> BulkApiError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        match status {
            StatusCode::UNAUTHORIZED => BulkApiError::AuthenticationError("Invalid or expired access token".to_string()),
            StatusCode::TOO_MANY_REQUESTS => {
                // Try to parse retry-after header
                let retry_after = 60; // Default to 60 seconds
                BulkApiError::RateLimitExceeded { retry_after }
            }
            _ => {
                // Try to parse Salesforce error response
                if let Ok(error_response) = serde_json::from_str::<Value>(&body) {
                    if let Some(message) = error_response.get("message").and_then(|v| v.as_str()) {
                        let error_code = error_response
                            .get("errorCode")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        BulkApiError::api_error(message, error_code)
                    } else {
                        BulkApiError::api_error(format!("API error ({}): {}", status, body), None)
                    }
                } else {
                    BulkApiError::api_error(format!("API error ({}): {}", status, body), None)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BulkClient::new(
            "https://myinstance.salesforce.com",
            "test_token"
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_job_config_validation() {
        let config = JobConfig::new("", Operation::Insert);
        let client = BulkClient::new("https://test.salesforce.com", "token").unwrap();
        
        // This would fail during create_job due to validation
        assert!(config.validate().is_err());
    }
}