#!/usr/bin/env python3
import requests
import sys

# MarkMail API設定（開発環境）
API_BASE_URL = "https://dev.markmail.engineers-hub.ltd/api"

# コマンドライン引数から取得
if len(sys.argv) != 4:
    print("使用方法: python publish_form_dev.py <email> <password> <form_id>")
    sys.exit(1)

EMAIL = sys.argv[1]
PASSWORD = sys.argv[2]
FORM_ID = sys.argv[3]

# ログイン
print("MarkMail開発環境にログイン中...")
login_response = requests.post(
    f"{API_BASE_URL}/auth/login",
    json={"email": EMAIL, "password": PASSWORD}
)

if login_response.status_code != 200:
    print(f"ログインエラー: {login_response.status_code}")
    print(login_response.text)
    exit(1)

auth_data = login_response.json()
token = auth_data.get("access_token") or auth_data.get("token")
headers = {"Authorization": f"Bearer {token}"}

print(f"ログイン成功")

# フォームを公開
print(f"\nフォーム {FORM_ID} を公開中...")
update_response = requests.put(
    f"{API_BASE_URL}/forms/{FORM_ID}",
    headers=headers,
    json={"status": "published"}
)

if update_response.status_code != 200:
    print(f"フォーム更新エラー: {update_response.status_code}")
    print(update_response.text)
    exit(1)

updated_form = update_response.json()
print(f"\nフォーム公開成功!")
print(f"フォーム名: {updated_form['name']}")
print(f"ステータス: {updated_form['status']}")
print(f"\n公開フォームURL:")
print(f"https://dev.markmail.engineers-hub.ltd/forms/{FORM_ID}/public")