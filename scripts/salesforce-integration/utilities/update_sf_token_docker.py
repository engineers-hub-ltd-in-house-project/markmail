#!/usr/bin/env python3
import json
import subprocess

# 新しいトークン（エスケープ不要）
new_token = "00DIR000001cWPD!AQwAQJS4qLKbsWVJ7pBVNGD2tC4im.Sjk.uXVSElzam60HsT2dvOst1ZGuX5wUKoTSpL5lIKBuXUKw2F8nYw30.RHHAsC5ch"

# credentials JSONを作成
credentials = {
    "access_token": new_token,
    "refresh_token": None
}

# JSONをエスケープ
credentials_json = json.dumps(credentials).replace("'", "''")

# SQLコマンドを作成（E'...'記法を使用）
sql_command = f"""
UPDATE crm_integrations 
SET credentials = '{credentials_json}'::json,
    updated_at = NOW()
WHERE user_id = 'b7892805-aeff-490e-80dc-544531d5660c' 
AND provider = 'salesforce';

SELECT id, provider, updated_at 
FROM crm_integrations 
WHERE user_id = 'b7892805-aeff-490e-80dc-544531d5660c';
"""

# Dockerコマンドを実行
result = subprocess.run([
    "docker", "exec", "markmail-postgres-1", "psql", 
    "-U", "markmail", "-d", "markmail_dev", 
    "-c", sql_command
], capture_output=True, text=True)

print("実行結果:")
print(result.stdout)

if result.stderr:
    print("エラー:")
    print(result.stderr)

print("\nSalesforceトークンが更新されました！")