#!/usr/bin/env python3
import requests
import json

# MarkMail API設定
API_BASE_URL = "http://localhost:3000/api"

# ログイン情報
EMAIL = "yusuke.sato@engineers-hub.ltd"
PASSWORD = os.getenv("MARKMAIL_PASSWORD", "your-password")

# フォームID
FORM_ID = "92c55a20-85cf-4418-a127-5ccdf39c4c0f"

# ログイン
print("MarkMailにログイン中...")
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

# フォームを公開（published）に更新
print(f"\nフォームを公開中...")
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
print(f"http://localhost:5173/forms/{FORM_ID}/public")
print(f"\n埋め込みコード:")
print(f'<iframe src="http://localhost:5173/forms/{FORM_ID}/public" width="100%" height="1200" frameborder="0"></iframe>')