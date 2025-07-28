// Create custom fields on Lead object for MarkMail integration
MetadataService.MetadataPort service = new MetadataService.MetadataPort();
service.SessionHeader = new MetadataService.SessionHeader_element();
service.SessionHeader.sessionId = UserInfo.getSessionId();

// Create MarkMail_Form_ID__c field
MetadataService.CustomField formIdField = new MetadataService.CustomField();
formIdField.fullName = 'Lead.MarkMail_Form_ID__c';
formIdField.label = 'MarkMail Form ID';
formIdField.type_x = 'Text';
formIdField.length = 255;
formIdField.required = false;
formIdField.unique_x = false;
formIdField.externalId = false;
formIdField.description = 'MarkMail form ID that created this lead';

// Create MarkMail_Submission_ID__c field  
MetadataService.CustomField submissionIdField = new MetadataService.CustomField();
submissionIdField.fullName = 'Lead.MarkMail_Submission_ID__c';
submissionIdField.label = 'MarkMail Submission ID';
submissionIdField.type_x = 'Text';
submissionIdField.length = 255;
submissionIdField.required = false;
submissionIdField.unique_x = false;
submissionIdField.externalId = false;
submissionIdField.description = 'MarkMail form submission ID that created this lead';

// Create fields
List<MetadataService.SaveResult> results = 
    service.createMetadata(
        new MetadataService.Metadata[] { formIdField, submissionIdField }
    );

// Output results
for(MetadataService.SaveResult result : results) {
    if(result.success) {
        System.debug('Successfully created field: ' + result.fullName);
    } else {
        System.debug('Error creating field: ' + result.errors[0].message);
    }
}