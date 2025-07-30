//! Tests for the Salesforce Bulk API client

#[cfg(test)]
mod integration_tests {
    use crate::{BulkClient, JobConfig, Operation};

    // These tests require actual Salesforce credentials
    // Set environment variables:
    // - SALESFORCE_INSTANCE_URL
    // - SALESFORCE_ACCESS_TOKEN
    
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_create_job() {
        let instance_url = std::env::var("SALESFORCE_INSTANCE_URL")
            .expect("SALESFORCE_INSTANCE_URL not set");
        let access_token = std::env::var("SALESFORCE_ACCESS_TOKEN")
            .expect("SALESFORCE_ACCESS_TOKEN not set");

        let client = BulkClient::new(instance_url, access_token).unwrap();
        
        let config = JobConfig::new("Contact", Operation::Insert);
        let job = client.create_job(config).await.unwrap();
        
        assert_eq!(job.object, "Contact");
        assert_eq!(job.operation, Operation::Insert);
        assert!(job.is_open());
        
        // Clean up: abort the job
        let _ = client.abort_job(&job.id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_upload_and_process_data() {
        let instance_url = std::env::var("SALESFORCE_INSTANCE_URL")
            .expect("SALESFORCE_INSTANCE_URL not set");
        let access_token = std::env::var("SALESFORCE_ACCESS_TOKEN")
            .expect("SALESFORCE_ACCESS_TOKEN not set");

        let client = BulkClient::new(instance_url, access_token).unwrap();
        
        // Create job
        let config = JobConfig::new("Contact", Operation::Insert);
        let job = client.create_job(config).await.unwrap();
        
        // Upload data
        let csv_data = "FirstName,LastName,Email\nTest,User,test@example.com";
        client.upload_data(&job.id, csv_data).await.unwrap();
        
        // Start processing
        let job = client.start_job(&job.id).await.unwrap();
        assert!(job.state.is_processing() || job.state.is_terminal());
        
        // Wait for completion
        let completed_job = client.wait_for_completion(&job.id).await.unwrap();
        assert!(completed_job.is_complete());
        
        // Get results
        let successful = client.get_successful_records(&job.id).await.unwrap();
        println!("Successful records: {}", successful);
        
        if completed_job.number_records_failed > 0 {
            let failed = client.get_failed_records(&job.id).await.unwrap();
            println!("Failed records: {}", failed);
        }
    }
}