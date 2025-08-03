# Salesforce Lead カスタムフィールド権限問題の完全記録

## 発生日時

2025年7月28日

## 問題の概要

MarkMailのフォーム送信時に自動的にSalesforceのリードを作成する機能を実装した際、カスタムフィールド（MarkMail_Form_ID**c、MarkMail_Submission_ID**c）へのアクセスエラーが発生した。

## 初期状況

### ユーザーの要求

```
申し込みフォームから登録があると、セールスフォースのリードに自動登録されるようにしたいです
```

### 既存の実装確認

- docs/salesforce-integration-status.md を確認
- Salesforce統合は既に実装済みだが、リード作成機能は未実装だった

## 実装内容

### 1. CrmLead モデルの追加

`backend/src/models/crm.rs` に以下を追加：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmLead {
    pub id: Option<String>,
    pub markmail_form_id: Uuid,
    pub markmail_submission_id: Option<Uuid>,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub title: Option<String>,
    pub website: Option<String>,
    pub lead_source: String,
    pub status: Option<String>,
    pub description: Option<String>,
    pub custom_fields: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
}
```

### 2. CrmProvider トレイトの拡張

`backend/src/services/crm_service/mod.rs` に以下のメソッドを追加：

```rust
async fn create_lead(&self, lead: &CrmLead) -> Result<CrmSyncResult, CrmError>;
async fn get_lead(&self, email: &str) -> Result<Option<CrmLead>, CrmError>;
async fn convert_lead_to_contact(&self, lead_id: &str) -> Result<CrmSyncResult, CrmError>;
```

### 3. SalesforceProvider での実装

`backend/src/services/crm_service/salesforce.rs` に実装を追加

### 4. フォーム送信時のリード作成処理

`backend/src/api/forms.rs` の submit_form 関数に以下を追加：

```rust
// CRM統合をチェックしてリードを作成
if let Ok(Some(integration)) = crm_integrations::get_user_crm_integration(
    &state.db,
    form.user_id,
    CrmProviderType::Salesforce,
)
.await
{
    if integration.is_active() {
        let lead = CrmLead::from_form_submission(
            form_id,
            Some(submission.id),
            &request.data,
            &form.form_fields,
            &form.name,
        );

        match CrmService::new(state.db.clone(), form.user_id).await {
            Ok(crm_service) => {
                if let Err(e) = crm_service.provider().create_lead(&lead).await {
                    tracing::error!("Salesforceリード作成エラー: {:?}", e);
                } else {
                    tracing::info!("Salesforceリードを作成しました: {}", lead.email);
                }
            }
            Err(e) => {
                tracing::error!("CRMサービス初期化エラー: {:?}", e);
            }
        }
    }
}
```

## 発生した問題と解決過程

### 問題1: CRM統合が間違ったユーザーIDで設定されていた

**エラー**: CRMサービス初期化エラー **原因**: user_idが異なっていた
**解決**: 正しいuser_id（b7892805-aeff-490e-80dc-544531d5660c）にCRM統合をコピー

### 問題2: Salesforceセッショントークンの期限切れ

**エラー**: INVALID_SESSION_ID **解決**: `sf org display`
コマンドで新しいアクセストークンを取得し、データベースを更新

**ユーザーからのフィードバック**:

```
というか、sf コマンドでまずは正しくデータできているか見ろよハゲ
```

### 問題3: トークンのエスケープ文字問題

**エラー**: Invalid auth header
**原因**: トークンに含まれるバックスラッシュがエスケープされていた **解決**:
PostgreSQLのE'...'記法を使用して正しくトークンを保存

使用したPythonスクリプト（/tmp/fix_token.py）:

```python
#!/usr/bin/env python3
import json
import psycopg2

conn = psycopg2.connect(
    host="localhost",
    database="markmail_dev",
    user="markmail",
    password="postgres"
)

cur = conn.cursor()

# 正しいトークン（バックスラッシュなし）
token = "00DIR000001cWPD!AQwAQMaTQCEHhcgR.ASWLeWdPph3m1bpzbjSjR7XgzaKl6fVaf6y9z8ttDZR0AVlpW8ftqLBrvugHhTX5szMxm_WEe.Lgdpg"

credentials = {
    "access_token": token,
    "refresh_token": None
}

cur.execute(
    "UPDATE crm_integrations SET credentials = %s WHERE user_id = %s",
    (json.dumps(credentials), 'b7892805-aeff-490e-80dc-544531d5660c')
)

conn.commit()
cur.close()
conn.close()

print("Token updated successfully")
```

### 問題4: Descriptionフィールドが存在しない

**エラー**: No such column 'Description' on sobject of type Lead
**原因**: ユーザーのSalesforce組織にDescriptionフィールドが存在しない **解決**:
SalesforceLead構造体でdescriptionフィールドをコメントアウト

**ユーザーからのフィードバック**:

```
短期的な解決はやめろ、あとで放置するんだから
```

### 問題5: カスタムフィールドへのアクセスエラー（最も重要な問題）

**エラー**: No such column 'MarkMail_Form_ID\_\_c' on sobject of type Lead
**原因**: カスタムフィールドは存在するが、フィールドレベルセキュリティが設定されていない

#### 調査過程

1. **Tooling APIでフィールドの存在確認**:

```bash
sf data query --query "SELECT Id, DeveloperName, TableEnumOrId FROM CustomField WHERE TableEnumOrId = 'Lead' AND DeveloperName LIKE 'MarkMail%'" --use-tooling-api --json
```

結果: 3つのカスタムフィールドが存在

- MarkMail_ID (ID: 00NIR00000FUe3J2AT)
- MarkMail_Form_ID (ID: 00NIR00000FVMdg2AH)
- MarkMail_Submission_ID (ID: 00NIR00000FVMdh2AH)

2. **通常のAPIでアクセス試行**:

```bash
sf data query --query "SELECT Id, MarkMail_Form_ID__c FROM Lead LIMIT 1"
```

結果: ERROR: No such column 'MarkMail_Form_ID\_\_c' on sobject of type Lead

3. **フィールドレベルセキュリティの確認**:

```bash
sf data query --query "SELECT Id, Field, PermissionsEdit, PermissionsRead FROM FieldPermissions WHERE Parent.ProfileId = '00eIR000001ywwXYAQ' AND SobjectType = 'Lead' AND Field IN ('Lead.MarkMail_Form_ID__c', 'Lead.MarkMail_Submission_ID__c')" --json
```

結果: レコードが0件（フィールドレベルセキュリティが未設定）

**ユーザーからのフィードバック**:

```
確かなのか？言い方を変えるだけではだめでしょ
```

#### 解決方法

1. **メタデータの再デプロイ**:

```bash
sf project deploy start --source-dir salesforce-metadata/objects/Lead/fields --wait 10
```

2. **Apexコードでフィールドレベルセキュリティを設定**: 作成したApexコード（salesforce-metadata/setFieldPermissions.apex）を実行：

```apex
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

// Create field permissions if they don't exist
List<FieldPermissions> newPerms = new List<FieldPermissions>();

// Check if permissions exist
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
}

if (!hasSubmissionIdPerm) {
    FieldPermissions submissionIdPerm = new FieldPermissions();
    submissionIdPerm.ParentId = permSet.Id;
    submissionIdPerm.SobjectType = 'Lead';
    submissionIdPerm.Field = 'Lead.MarkMail_Submission_ID__c';
    submissionIdPerm.PermissionsRead = true;
    submissionIdPerm.PermissionsEdit = true;
    newPerms.add(submissionIdPerm);
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
```

実行結果:

```
Successfully created 2 field permissions
Final permissions count: 2
Field: Lead.MarkMail_Form_ID__c, Read: true, Edit: true
Field: Lead.MarkMail_Submission_ID__c, Read: true, Edit: true
```

## テスト結果

### フィールドアクセス確認

```bash
sf data query --query "SELECT Id, Email, FirstName, LastName, Company, MarkMail_Form_ID__c, MarkMail_Submission_ID__c FROM Lead LIMIT 1" --json
```

結果: カスタムフィールドにアクセス可能になった

### フォーム送信テスト

```
2025-07-28T11:56:15.207420Z  INFO request{method=POST uri=/api/forms/2ee0bfd4-2e2b-409d-b74c-9fd184ee6947/submit version=HTTP/1.1}: markmail_backend::api::forms: Salesforceリードを作成しました: test5@example.com
```

### Salesforceでの確認

```bash
sf data query --query "SELECT Id, Email, FirstName, LastName, Company, MarkMail_Form_ID__c, MarkMail_Submission_ID__c FROM Lead WHERE Email = 'test5@example.com'" --json
```

結果:

```json
{
  "Id": "00QIR00000dPmyK2AS",
  "Email": "test5@example.com",
  "FirstName": null,
  "LastName": "Unknown",
  "Company": "Unknown Company",
  "MarkMail_Form_ID__c": "2ee0bfd4-2e2b-409d-b74c-9fd184ee6947",
  "MarkMail_Submission_ID__c": "1cc9beba-e212-4135-97a3-521eeb5280f9"
}
```

## 重要な教訓

1. **Salesforceのカスタムフィールドは作成しただけではREST
   APIからアクセスできない**

   - フィールドレベルセキュリティの設定が必須
   - Tooling APIでは見えるが、通常のAPIでは見えないという状況が発生する

2. **sf コマンドを活用した確認の重要性**

   - ユーザーから「sf コマンドでまずは正しくデータできているか見ろよハゲ」という指摘
   - 問題の切り分けに sf コマンドは非常に有効

3. **恒久的な解決策を実装する**

   - ユーザーから「短期的な解決はやめろ、あとで放置するんだから」という指摘
   - 根本的な解決（フィールドレベルセキュリティの設定）を実施

4. **年度の確認**
   - ユーザーから「おい、今年 2025 年だぞ」という指摘
   - ドキュメントやコメントで正しい年度を使用することの重要性

## 関連ファイル

- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/backend/src/models/crm.rs`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/backend/src/services/crm_service/mod.rs`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/backend/src/services/crm_service/salesforce.rs`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/backend/src/api/forms.rs`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/salesforce-metadata/objects/Lead/fields/MarkMail_Form_ID__c.field-meta.xml`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/salesforce-metadata/objects/Lead/fields/MarkMail_Submission_ID__c.field-meta.xml`
- `/home/yusuke/engineers-hub.ltd/in-house-project/markmail/salesforce-metadata/setFieldPermissions.apex`
- `/tmp/fix_token.py`
- `/tmp/update_token.sql`

## 今後の課題

1. **フィールドレベルセキュリティの自動設定**

   - メタデータデプロイ時にプロファイル設定も含める
   - または、セットアップ手順にフィールドレベルセキュリティの設定を明記

2. **エラーハンドリングの向上**

   - カスタムフィールドへのアクセスエラーをより明確に表示
   - フィールドレベルセキュリティの問題を示唆するメッセージを追加

3. **ドキュメントの充実**
   - Salesforce統合のトラブルシューティングガイドを作成
   - カスタムフィールドの設定手順を詳細に記載
