#!/usr/bin/env python3
import subprocess
import json

# カスタムフィールドのリスト（新しく追加する必要があるもの）
REQUIRED_FIELDS = [
    "Java__c",
    "Python__c", 
    "JavaScript_TypeScript__c",
    "C_C__c",
    "C__c",
    "PHP__c",
    "Go__c",
    "Ruby__c",
    "Swift__c",
    "Kotlin__c",
    "React__c",
    "Next_js__c",
    "Django__c",
    "Ruby_on_Rails__c",
    "React_Native__c",
    "PostgreSQL__c",
    "SQL_Server__c",
    "Kubernetes__c",
    "Azure__c",
    "Vue_js__c",
    "Svelte__c",
    "Flask__c",
    "Laravel__c",
    "Flutter__c",
    "MongoDB__c",
    "Redis__c",
    "AWS__c",
    "Jenkins__c",
    "Angular__c",
    "Spring__c",
    "Express__c",
    "ASP_NET__c",
    "MySQL__c",
    "Oracle__c",
    "Docker__c",
    "GCP__c",
    "GitHub_Actions__c",
    "GitHub_URL__c",
    "URL__c",
    "PR__c",
    "Co_Ltd__c"
]

print("Salesforceのフィールドレベルセキュリティを確認中...")

# システム管理者プロファイルのパーミッションセットを取得
result = subprocess.run([
    "sf", "data", "query",
    "--query", "SELECT Id FROM PermissionSet WHERE ProfileId = '00eIR000001ywwXYAQ' LIMIT 1",
    "-o", "markmail-org",
    "--json"
], capture_output=True, text=True)

if result.returncode != 0:
    print("Error getting permission set")
    exit(1)

perm_set_data = json.loads(result.stdout)
if not perm_set_data["result"]["records"]:
    print("Permission set not found")
    exit(1)

perm_set_id = perm_set_data["result"]["records"][0]["Id"]
print(f"パーミッションセットID: {perm_set_id}")

# 既存のフィールドパーミッションを確認
missing_fields = []
for field in REQUIRED_FIELDS:
    query = f"SELECT Id, Field, PermissionsEdit, PermissionsRead FROM FieldPermissions WHERE ParentId = '{perm_set_id}' AND Field = 'Lead.{field}'"
    
    result = subprocess.run([
        "sf", "data", "query",
        "--query", query,
        "-o", "markmail-org",
        "--json"
    ], capture_output=True, text=True)
    
    if result.returncode == 0:
        data = json.loads(result.stdout)
        if data["result"]["totalSize"] == 0:
            missing_fields.append(field)
            print(f"❌ {field} - 権限設定なし")
        else:
            perm = data["result"]["records"][0]
            print(f"✅ {field} - 読み取り: {perm['PermissionsRead']}, 編集: {perm['PermissionsEdit']}")
    else:
        missing_fields.append(field)
        print(f"❌ {field} - 確認エラー")

print(f"\n権限設定が必要なフィールド数: {len(missing_fields)}")
if missing_fields:
    print("以下のフィールドに権限設定が必要です:")
    for field in missing_fields:
        print(f"  - {field}")