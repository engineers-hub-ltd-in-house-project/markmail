# Salesforce Integration Setup Manual

This manual provides step-by-step instructions to set up Salesforce integration
with MarkMail, including custom field creation and subscriber synchronization.

## Prerequisites

1. Salesforce Developer/Sandbox Account
2. Salesforce CLI (`sf`) installed
3. MarkMail backend running locally
4. Docker containers running (PostgreSQL)

## Step 1: Salesforce CLI Setup

### 1.1 Install Salesforce CLI

```bash
npm install -g @salesforce/cli
```

### 1.2 Verify Installation

```bash
sf --version
```

### 1.3 Authenticate with Salesforce

```bash
sf org login web --alias markmail-org
```

This will open a browser for authentication. Log in with your Salesforce
credentials.

### 1.4 Verify Connection

```bash
sf org display --target-org markmail-org
```

## Step 2: Create Custom Field in Salesforce

The MarkMail integration requires a custom field `MarkMail_ID__c` on the Contact
object to track the relationship between MarkMail subscribers and Salesforce
contacts.

### 2.1 Create Metadata Structure

```bash
# Create directory structure
mkdir -p salesforce-metadata/objects/Contact/fields
```

### 2.2 Create Field Definition

Create
`salesforce-metadata/objects/Contact/fields/MarkMail_ID__c.field-meta.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<CustomField xmlns="http://soap.sforce.com/2006/04/metadata">
    <fullName>MarkMail_ID__c</fullName>
    <caseSensitive>false</caseSensitive>
    <description>Unique identifier from MarkMail system</description>
    <externalId>true</externalId>
    <label>MarkMail ID</label>
    <length>255</length>
    <required>false</required>
    <trackFeedHistory>false</trackFeedHistory>
    <type>Text</type>
    <unique>true</unique>
</CustomField>
```

### 2.3 Create Admin Profile Permissions

Create `salesforce-metadata/profiles/Admin.profile-meta.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Profile xmlns="http://soap.sforce.com/2006/04/metadata">
    <fieldPermissions>
        <editable>true</editable>
        <field>Contact.MarkMail_ID__c</field>
        <readable>true</readable>
    </fieldPermissions>
</Profile>
```

### 2.4 Create Package Manifest

Create `salesforce-metadata/package.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Package xmlns="http://soap.sforce.com/2006/04/metadata">
    <types>
        <members>Contact.MarkMail_ID__c</members>
        <name>CustomField</name>
    </types>
    <types>
        <members>Admin</members>
        <name>Profile</name>
    </types>
    <version>62.0</version>
</Package>
```

### 2.5 Create SFDX Project Configuration

Create `sfdx-project.json` in the project root:

```json
{
  "packageDirectories": [
    {
      "path": "salesforce-metadata",
      "default": true
    }
  ],
  "namespace": "",
  "sfdcLoginUrl": "https://login.salesforce.com",
  "sourceApiVersion": "62.0"
}
```

### 2.6 Deploy to Salesforce

```bash
# Deploy the custom field and permissions
sf project deploy start -d salesforce-metadata -o markmail-org -w 10
```

### 2.7 Verify Deployment

```bash
# Query to verify the field exists
sf data query -q "SELECT Id, FirstName, LastName, Email, MarkMail_ID__c FROM Contact LIMIT 1" -o markmail-org
```

## Step 3: Configure MarkMail Integration

### 3.1 Create Test User (if needed)

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@markmail.com",
    "password": "test1234",
    "name": "Test User"
  }'
```

Save the JWT token from the response.

### 3.2 Create CRM Integration

```bash
# Replace YOUR_JWT_TOKEN with the token from the previous step
curl -X POST http://localhost:3000/api/crm/integrations \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "salesforce",
    "org_alias": "markmail-org",
    "settings": {
      "sync_enabled": true,
      "sync_interval_minutes": 60,
      "batch_size": 200,
      "field_mappings": []
    }
  }'
```

## Step 4: Test Subscriber Synchronization

### 4.1 Create a Test Subscriber

```bash
curl -X POST http://localhost:3000/api/subscribers \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "name": "John Doe",
    "status": "Active"
  }'
```

Note the subscriber ID from the response.

### 4.2 Sync Subscriber to Salesforce

```bash
# Replace SUBSCRIBER_ID with the ID from the previous step
curl -X POST http://localhost:3000/api/crm/sync/contacts \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "contacts",
    "entity_ids": ["SUBSCRIBER_ID"]
  }'
```

### 4.3 Verify in Salesforce

```bash
# Replace SALESFORCE_CONTACT_ID with the crm_id from the sync response
sf data query -q "SELECT Id, FirstName, LastName, Email, MarkMail_ID__c FROM Contact WHERE Email = 'john.doe@example.com'" -o markmail-org
```

## Step 5: Bulk Synchronization

### 5.1 Sync All Subscribers

```bash
curl -X POST http://localhost:3000/api/crm/sync/subscribers/bulk \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

## Troubleshooting

### Common Issues

1. **"No such column 'MarkMail_ID\_\_c'" Error**

   - Ensure the custom field is deployed with proper permissions
   - Check that the Admin profile has read/write access to the field
   - Wait a few minutes for Salesforce to propagate the changes

2. **Authentication Errors**

   - Re-authenticate with `sf org login web --alias markmail-org`
   - Verify the org connection with `sf org display`

3. **CRM Integration Not Found**
   - Ensure you've created the integration (Step 3.2)
   - Check that the JWT token is valid and not expired

### Useful Commands

```bash
# List all Salesforce orgs
sf org list

# Check deployment status
sf project deploy report -o markmail-org --use-most-recent

# Retrieve metadata from Salesforce
sf project retrieve start -m CustomField:Contact.MarkMail_ID__c -o markmail-org

# View Salesforce logs
sf apex tail log -o markmail-org
```

## API Endpoints Reference

### CRM Integration Management

- `POST /api/crm/integrations` - Create CRM integration
- `GET /api/crm/integrations/current` - Get current integration
- `PUT /api/crm/integrations/:id` - Update integration
- `DELETE /api/crm/integrations/:id` - Delete integration

### Synchronization

- `POST /api/crm/sync/contacts` - Sync specific contacts
- `POST /api/crm/sync/subscribers/bulk` - Bulk sync all subscribers
- `GET /api/crm/sync/status` - Get sync status

## Notes

- The integration uses Salesforce Bulk API 2.0 for efficient data
  synchronization
- Custom field `MarkMail_ID__c` is used to maintain the relationship between
  MarkMail and Salesforce
- Field-Level Security (FLS) must be properly configured for the custom field
- The integration supports bi-directional synchronization (future enhancement)
