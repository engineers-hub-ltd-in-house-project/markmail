//! Utility functions for Salesforce Bulk API operations

use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::io::Read;

use crate::error::Result;

/// Convert a vector of structs to CSV format
pub fn to_csv<T: Serialize>(records: &[T]) -> Result<String> {
    let mut wtr = Writer::from_writer(vec![]);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    let data = wtr.into_inner()
        .map_err(|e| crate::BulkApiError::Other(format!("Failed to get CSV writer data: {}", e)))?;
    Ok(String::from_utf8(data).map_err(|e| crate::BulkApiError::Other(e.to_string()))?)
}

/// Parse CSV data into a vector of structs
pub fn from_csv<T: for<'de> Deserialize<'de>, R: Read>(reader: R) -> Result<Vec<T>> {
    let mut rdr = Reader::from_reader(reader);
    let mut records = Vec::new();
    
    for result in rdr.deserialize() {
        let record: T = result?;
        records.push(record);
    }
    
    Ok(records)
}

/// Split large CSV data into chunks
pub fn chunk_csv_data(csv_data: &str, chunk_size: usize) -> Vec<String> {
    let lines: Vec<&str> = csv_data.lines().collect();
    
    if lines.is_empty() {
        return vec![];
    }
    
    let header = lines[0];
    let data_lines = &lines[1..];
    
    let mut chunks = Vec::new();
    
    for chunk in data_lines.chunks(chunk_size) {
        let mut chunk_data = String::new();
        chunk_data.push_str(header);
        chunk_data.push('\n');
        
        for line in chunk {
            chunk_data.push_str(line);
            chunk_data.push('\n');
        }
        
        chunks.push(chunk_data);
    }
    
    if chunks.is_empty() && !lines.is_empty() {
        // If there's only a header, return it as a single chunk
        chunks.push(csv_data.to_string());
    }
    
    chunks
}

/// Build SOQL query from fields and conditions
pub fn build_soql_query(
    object: &str,
    fields: &[&str],
    conditions: Option<&str>,
    limit: Option<u32>,
) -> String {
    let mut query = format!("SELECT {} FROM {}", fields.join(", "), object);
    
    if let Some(where_clause) = conditions {
        query.push_str(&format!(" WHERE {}", where_clause));
    }
    
    if let Some(limit_value) = limit {
        query.push_str(&format!(" LIMIT {}", limit_value));
    }
    
    query
}

/// Escape special characters in SOQL strings
pub fn escape_soql_string(s: &str) -> String {
    s.replace('\'', "\\'")
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Format Salesforce datetime
pub fn format_salesforce_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestRecord {
        name: String,
        email: String,
        age: u32,
    }

    #[test]
    fn test_to_csv() {
        let records = vec![
            TestRecord {
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                age: 30,
            },
            TestRecord {
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
                age: 25,
            },
        ];

        let csv = to_csv(&records).unwrap();
        assert!(csv.contains("name,email,age"));
        assert!(csv.contains("John Doe,john@example.com,30"));
        assert!(csv.contains("Jane Smith,jane@example.com,25"));
    }

    #[test]
    fn test_from_csv() {
        let csv_data = "name,email,age\nJohn Doe,john@example.com,30\nJane Smith,jane@example.com,25\n";
        let records: Vec<TestRecord> = from_csv(csv_data.as_bytes()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "John Doe");
        assert_eq!(records[1].name, "Jane Smith");
    }

    #[test]
    fn test_chunk_csv_data() {
        let csv_data = "name,email\nJohn,john@example.com\nJane,jane@example.com\nBob,bob@example.com\nAlice,alice@example.com\n";
        let chunks = chunk_csv_data(csv_data, 2);
        
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("John"));
        assert!(chunks[0].contains("Jane"));
        assert!(chunks[1].contains("Bob"));
        assert!(chunks[1].contains("Alice"));
        
        // All chunks should have the header
        for chunk in &chunks {
            assert!(chunk.starts_with("name,email"));
        }
    }

    #[test]
    fn test_build_soql_query() {
        let query = build_soql_query(
            "Contact",
            &["Id", "Name", "Email"],
            Some("Email != null"),
            Some(100),
        );
        
        assert_eq!(
            query,
            "SELECT Id, Name, Email FROM Contact WHERE Email != null LIMIT 100"
        );
    }

    #[test]
    fn test_escape_soql_string() {
        assert_eq!(escape_soql_string("O'Brien"), "O\\'Brien");
        assert_eq!(escape_soql_string("Line1\nLine2"), "Line1\\nLine2");
        assert_eq!(escape_soql_string("Tab\there"), "Tab\\there");
    }

    #[test]
    fn test_format_salesforce_datetime() {
        use chrono::TimeZone;
        
        let dt = chrono::Utc.with_ymd_and_hms(2025, 6, 26, 12, 30, 45).unwrap();
        assert_eq!(
            format_salesforce_datetime(&dt),
            "2025-06-26T12:30:45.000Z"
        );
    }
}