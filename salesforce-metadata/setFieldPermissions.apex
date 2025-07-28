// Set field-level security for MarkMail custom fields on Lead object
// Run this in Salesforce Developer Console or with sf apex run

// Get System Administrator profile
List<Profile> profiles = [SELECT Id, Name FROM Profile WHERE Name = 'システム管理者' OR Name = 'System Administrator' LIMIT 1];

if (profiles.isEmpty()) {
    System.debug('Error: System Administrator profile not found');
    return;
}

Profile adminProfile = profiles[0];
System.debug('Found profile: ' + adminProfile.Name + ' (' + adminProfile.Id + ')');

// Check existing field permissions
List<FieldPermissions> existingPerms = [
    SELECT Id, Field, PermissionsEdit, PermissionsRead 
    FROM FieldPermissions 
    WHERE ParentId IN (SELECT Id FROM PermissionSet WHERE ProfileId = :adminProfile.Id)
    AND Field IN ('Lead.MarkMail_Form_ID__c', 'Lead.MarkMail_Submission_ID__c')
];

System.debug('Existing permissions: ' + existingPerms.size());
for (FieldPermissions perm : existingPerms) {
    System.debug('Field: ' + perm.Field + ', Read: ' + perm.PermissionsRead + ', Edit: ' + perm.PermissionsEdit);
}

// Get the permission set associated with the profile
List<PermissionSet> permSets = [
    SELECT Id, Name 
    FROM PermissionSet 
    WHERE ProfileId = :adminProfile.Id 
    LIMIT 1
];

if (permSets.isEmpty()) {
    System.debug('Error: Permission set for profile not found');
    return;
}

PermissionSet permSet = permSets[0];
System.debug('Found permission set: ' + permSet.Id);

// Create field permissions if they don't exist
List<FieldPermissions> newPerms = new List<FieldPermissions>();

// Check if MarkMail_Form_ID__c permission exists
Boolean hasFormIdPerm = false;
Boolean hasSubmissionIdPerm = false;

for (FieldPermissions perm : existingPerms) {
    if (perm.Field == 'Lead.MarkMail_Form_ID__c') {
        hasFormIdPerm = true;
    } else if (perm.Field == 'Lead.MarkMail_Submission_ID__c') {
        hasSubmissionIdPerm = true;
    }
}

if (!hasFormIdPerm) {
    FieldPermissions formIdPerm = new FieldPermissions();
    formIdPerm.ParentId = permSet.Id;
    formIdPerm.SobjectType = 'Lead';
    formIdPerm.Field = 'Lead.MarkMail_Form_ID__c';
    formIdPerm.PermissionsRead = true;
    formIdPerm.PermissionsEdit = true;
    newPerms.add(formIdPerm);
    System.debug('Creating permission for MarkMail_Form_ID__c');
}

if (!hasSubmissionIdPerm) {
    FieldPermissions submissionIdPerm = new FieldPermissions();
    submissionIdPerm.ParentId = permSet.Id;
    submissionIdPerm.SobjectType = 'Lead';
    submissionIdPerm.Field = 'Lead.MarkMail_Submission_ID__c';
    submissionIdPerm.PermissionsRead = true;
    submissionIdPerm.PermissionsEdit = true;
    newPerms.add(submissionIdPerm);
    System.debug('Creating permission for MarkMail_Submission_ID__c');
}

if (!newPerms.isEmpty()) {
    try {
        insert newPerms;
        System.debug('Successfully created ' + newPerms.size() + ' field permissions');
    } catch (Exception e) {
        System.debug('Error creating field permissions: ' + e.getMessage());
    }
} else {
    System.debug('Field permissions already exist');
}

// Verify the permissions were created
List<FieldPermissions> verifyPerms = [
    SELECT Id, Field, PermissionsEdit, PermissionsRead 
    FROM FieldPermissions 
    WHERE ParentId IN (SELECT Id FROM PermissionSet WHERE ProfileId = :adminProfile.Id)
    AND Field IN ('Lead.MarkMail_Form_ID__c', 'Lead.MarkMail_Submission_ID__c')
];

System.debug('Final permissions count: ' + verifyPerms.size());
for (FieldPermissions perm : verifyPerms) {
    System.debug('Field: ' + perm.Field + ', Read: ' + perm.PermissionsRead + ', Edit: ' + perm.PermissionsEdit);
}