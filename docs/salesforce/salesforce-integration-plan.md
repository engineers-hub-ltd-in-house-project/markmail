# Salesforce Integration Development Plan

## Executive Summary

This document outlines the development plan for integrating Salesforce CRM with
MarkMail. The integration will follow the existing provider pattern used in the
codebase, enabling bi-directional synchronization of contacts, campaign
activities, and marketing automation data between MarkMail and Salesforce.

## Architecture Overview

### Design Principles

1. **Provider Pattern**: Following the existing email provider abstraction
   pattern
2. **Modularity**: CRM integration as a pluggable module
3. **Scalability**: Support for multiple CRM providers in the future
4. **Data Consistency**: Bi-directional sync with conflict resolution
5. **Security**: Encrypted credential storage and secure API communication

### High-Level Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   MarkMail UI   │────▶│  MarkMail API   │────▶│  CRM Service    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                          │
                                                          ▼
                                                 ┌─────────────────┐
                                                 │  CRM Provider   │
                                                 │   Interface     │
                                                 └─────────────────┘
                                                          │
                                                          ▼
                                                 ┌─────────────────┐
                                                 │   Salesforce    │
                                                 │    Provider     │
                                                 └─────────────────┘
                                                          │
                                                          ▼
                                                 ┌─────────────────┐
                                                 │  Salesforce API │
                                                 │   (via sf CLI)  │
                                                 └─────────────────┘
```

## Technical Implementation

### 1. Core Components

#### CRM Provider Trait

```rust
// backend/src/services/crm_service/mod.rs
#[async_trait]
pub trait CRMProvider: Send + Sync {
    // Contact Management
    async fn sync_contact(&self, contact: &CRMContact) -> Result<CRMSyncResult, CRMError>;
    async fn get_contact(&self, email: &str) -> Result<Option<CRMContact>, CRMError>;
    async fn update_contact(&self, contact: &CRMContact) -> Result<CRMSyncResult, CRMError>;
    async fn delete_contact(&self, id: &str) -> Result<(), CRMError>;
    async fn bulk_sync_contacts(&self, contacts: Vec<CRMContact>) -> Result<BulkSyncResult, CRMError>;

    // Campaign & Activity Management
    async fn sync_campaign(&self, campaign: &CRMCampaign) -> Result<CRMSyncResult, CRMError>;
    async fn log_email_activity(&self, activity: &EmailActivity) -> Result<(), CRMError>;
    async fn sync_list_membership(&self, list: &CRMList) -> Result<(), CRMError>;

    // Custom Field Management
    async fn get_custom_fields(&self) -> Result<Vec<CustomField>, CRMError>;
    async fn map_custom_fields(&self, mapping: &FieldMapping) -> Result<(), CRMError>;

    // Metadata
    fn provider_name(&self) -> &str;
    fn supports_feature(&self, feature: CRMFeature) -> bool;
}
```

#### Salesforce Provider Implementation

```rust
// backend/src/services/crm_service/salesforce.rs
pub struct SalesforceProvider {
    sf_client: SfCommandClient,
    config: SalesforceConfig,
    field_mappings: HashMap<String, String>,
}

impl SalesforceProvider {
    pub async fn new(config: SalesforceConfig) -> Result<Self, CRMError> {
        let sf_client = SfCommandClient::new(&config)?;
        // Initialize connection and validate credentials
        Ok(Self { sf_client, config, field_mappings: HashMap::new() })
    }
}
```

### 2. Salesforce CLI Integration

Using the `sf` command (Salesforce CLI v2) via Node.js module:

```typescript
// backend/src/services/crm_service/sf_client.rs
pub struct SfCommandClient {
    org_alias: String,
    api_version: String,
}

impl SfCommandClient {
    pub async fn execute_soql(&self, query: &str) -> Result<SoqlResult, SfError> {
        // Execute: sf data query --query "SELECT Id, Email FROM Contact" --json
    }

    pub async fn create_record(&self, object: &str, data: Value) -> Result<String, SfError> {
        // Execute: sf data create record --sobject Contact --values ...
    }

    pub async fn update_record(&self, object: &str, id: &str, data: Value) -> Result<(), SfError> {
        // Execute: sf data update record --sobject Contact --record-id ... --values ...
    }
}
```

### 3. Data Models

```rust
// backend/src/models/crm.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRMContact {
    pub id: Option<String>,           // Salesforce ID
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, Value>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRMCampaign {
    pub id: Option<String>,
    pub name: String,
    pub status: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub member_count: i32,
    pub email_stats: EmailStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub markmail_field: String,
    pub salesforce_field: String,
    pub field_type: FieldType,
    pub sync_direction: SyncDirection,
}
```

### 4. Database Schema

```sql
-- CRM Integration Configuration
CREATE TABLE crm_integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    org_id VARCHAR(255),                    -- Salesforce Org ID
    credentials JSONB NOT NULL,             -- Encrypted credentials
    settings JSONB DEFAULT '{}',            -- Integration settings
    field_mappings JSONB DEFAULT '{}',      -- Field mapping configuration
    sync_enabled BOOLEAN DEFAULT true,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Sync Status Tracking
CREATE TABLE crm_sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID REFERENCES crm_integrations(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,       -- contact, campaign, etc.
    markmail_id UUID NOT NULL,              -- ID in MarkMail
    crm_id VARCHAR(255),                    -- ID in CRM
    last_sync_hash VARCHAR(64),             -- Hash of last synced data
    sync_status VARCHAR(20) NOT NULL,       -- synced, pending, error
    sync_direction VARCHAR(20),             -- to_crm, from_crm, bidirectional
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(integration_id, entity_type, markmail_id)
);

-- Sync Activity Log
CREATE TABLE crm_sync_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID REFERENCES crm_integrations(id) ON DELETE CASCADE,
    sync_type VARCHAR(50) NOT NULL,         -- manual, scheduled, webhook
    entity_type VARCHAR(50) NOT NULL,
    entity_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    error_count INTEGER DEFAULT 0,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_details JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Field Mapping Templates
CREATE TABLE crm_field_mapping_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    mappings JSONB NOT NULL,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, name)
);
```

### 5. API Endpoints

```rust
// backend/src/api/integrations.rs

// Integration Management
POST   /api/integrations/crm                    // Connect CRM
GET    /api/integrations/crm                    // Get integration status
PUT    /api/integrations/crm                    // Update settings
DELETE /api/integrations/crm                    // Disconnect CRM

// Sync Operations
POST   /api/integrations/crm/sync               // Trigger manual sync
GET    /api/integrations/crm/sync/status        // Get sync status
POST   /api/integrations/crm/sync/contacts      // Sync specific contacts
POST   /api/integrations/crm/sync/campaigns     // Sync campaigns

// Field Mapping
GET    /api/integrations/crm/fields             // Get available fields
GET    /api/integrations/crm/field-mappings     // Get current mappings
PUT    /api/integrations/crm/field-mappings     // Update mappings
GET    /api/integrations/crm/field-templates    // Get mapping templates

// Activity & Logs
GET    /api/integrations/crm/activity           // Get sync activity
GET    /api/integrations/crm/logs               // Get detailed logs
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)

1. **Core Infrastructure**

   - Create CRM provider trait and base models
   - Implement database schema migrations
   - Set up Salesforce CLI integration wrapper
   - Create basic error handling and logging

2. **Authentication & Connection**
   - Implement Salesforce OAuth flow
   - Secure credential storage
   - Connection testing endpoint
   - Basic configuration UI

### Phase 2: Contact Synchronization (Week 3-4)

1. **Contact Sync Implementation**

   - Implement contact CRUD operations
   - Bi-directional sync logic
   - Conflict resolution strategy
   - Field mapping engine

2. **Sync Management**
   - Manual sync triggers
   - Sync status tracking
   - Error handling and retry logic
   - Activity logging

### Phase 3: Campaign Integration (Week 5-6)

1. **Campaign & Activity Sync**

   - Campaign creation and updates
   - Email activity logging
   - Campaign member management
   - Performance metrics sync

2. **List Management**
   - Sync subscriber lists as campaigns
   - Manage campaign membership
   - Handle subscription status

### Phase 4: Advanced Features (Week 7-8)

1. **Custom Field Support**

   - Dynamic field discovery
   - Custom field mapping UI
   - Type conversion handling
   - Validation rules

2. **Automation & Webhooks**
   - Real-time sync via webhooks
   - Scheduled sync jobs
   - Bulk operations optimization
   - Rate limiting implementation

### Phase 5: UI & Polish (Week 9-10)

1. **User Interface**

   - Integration settings page
   - Field mapping interface
   - Sync status dashboard
   - Activity timeline view

2. **Testing & Documentation**
   - Comprehensive test suite
   - Integration tests with sandbox
   - API documentation
   - User guide creation

## Security Considerations

1. **Credential Management**

   - Encrypt all stored credentials using AES-256
   - Use environment-specific encryption keys
   - Implement key rotation mechanism
   - Audit credential access

2. **API Security**

   - Rate limiting per integration
   - Request signing/validation
   - IP allowlisting option
   - Comprehensive audit logging

3. **Data Protection**
   - PII handling compliance
   - Data retention policies
   - Right to deletion support
   - GDPR compliance features

## Performance Optimization

1. **Sync Optimization**

   - Batch API operations
   - Incremental sync using modified timestamps
   - Parallel processing for large datasets
   - Caching frequently accessed data

2. **Resource Management**
   - Connection pooling
   - Background job queuing
   - Memory-efficient streaming
   - Database query optimization

## Monitoring & Observability

1. **Metrics**

   - Sync success/failure rates
   - API call volumes
   - Sync duration tracking
   - Error rate monitoring

2. **Alerting**
   - Failed sync notifications
   - Rate limit warnings
   - Connection failure alerts
   - Data inconsistency detection

## Testing Strategy

1. **Unit Tests**

   - Provider interface tests
   - Data transformation tests
   - Field mapping tests
   - Error handling tests

2. **Integration Tests**

   - Salesforce sandbox testing
   - End-to-end sync tests
   - Webhook integration tests
   - Performance benchmarks

3. **User Acceptance Testing**
   - Field mapping workflows
   - Sync reliability testing
   - Error recovery scenarios
   - UI/UX validation

## Future Enhancements

1. **Additional CRM Support**

   - HubSpot integration
   - Pipedrive integration
   - Microsoft Dynamics
   - Custom CRM adapters

2. **Advanced Features**

   - AI-powered field mapping suggestions
   - Duplicate detection and merging
   - Advanced segmentation sync
   - Multi-org support

3. **Analytics Integration**
   - Salesforce reports integration
   - Custom dashboard widgets
   - ROI tracking
   - Attribution modeling

## Dependencies

1. **External Libraries**

   - `sf` CLI (Salesforce CLI v2)
   - `tokio` for async runtime
   - `serde` for serialization
   - `sqlx` for database access

2. **Infrastructure**
   - Redis for job queuing (future)
   - PostgreSQL for data storage
   - AWS S3 for backup/restore (future)

## Risks & Mitigation

1. **API Rate Limits**

   - Risk: Hitting Salesforce API limits
   - Mitigation: Implement intelligent rate limiting and batching

2. **Data Consistency**

   - Risk: Sync conflicts and data loss
   - Mitigation: Implement robust conflict resolution and audit trails

3. **Performance Impact**

   - Risk: Sync operations affecting app performance
   - Mitigation: Background processing and resource throttling

4. **Security Vulnerabilities**
   - Risk: Credential exposure or unauthorized access
   - Mitigation: Regular security audits and encryption best practices

## Success Criteria

1. **Functional Requirements**

   - Successful bi-directional contact sync
   - Campaign activity tracking
   - Custom field support
   - Real-time sync capability

2. **Performance Requirements**

   - Sync 10,000 contacts in under 5 minutes
   - API response time < 200ms
   - 99.9% sync reliability

3. **User Experience**
   - Intuitive setup process
   - Clear sync status visibility
   - Helpful error messages
   - Comprehensive documentation

## Conclusion

This Salesforce integration will provide MarkMail users with powerful CRM
synchronization capabilities while maintaining the flexibility to add other CRM
providers in the future. The modular architecture ensures maintainability and
scalability as the platform grows.
