#!/usr/bin/env python3
import requests
import sys
import os

# コールバックURLから認証コードとstateを取得
if len(sys.argv) < 2:
    print("使用方法: python3 complete_oauth_callback.py 'コールバックURL全体'")
    print("例: python3 complete_oauth_callback.py 'http://localhost:3000/api/crm/oauth/salesforce/callback?code=xxx&state=yyy'")
    exit(1)

callback_url = sys.argv[1]

# URLからcodeとstateを抽出
from urllib.parse import urlparse, parse_qs
parsed = urlparse(callback_url)
params = parse_qs(parsed.query)

if 'code' not in params or 'state' not in params:
    print("❌ URLにcodeまたはstateパラメータがありません")
    exit(1)

code = params['code'][0]
state = params['state'][0]

print(f"認証コード: {code[:20]}...")
print(f"State: {state}")

# まずログインしてトークンを取得
API_BASE_URL = "http://localhost:3000/api"

# 環境変数から認証情報を取得
EMAIL = os.getenv("MARKMAIL_TEST_EMAIL", "yusuke.sato@engineers-hub.ltd")
PASSWORD = os.getenv("MARKMAIL_TEST_PASSWORD")

if not PASSWORD:
    print("❌ エラー: 環境変数 MARKMAIL_TEST_PASSWORD が設定されていません")
    print("使用方法: MARKMAIL_TEST_PASSWORD=your_password python3 complete_oauth_callback.py 'callback_url'")
    exit(1)

print("\n1. ログイン中...")
login_response = requests.post(
    f"{API_BASE_URL}/auth/login",
    json={
        "email": EMAIL,
        "password": PASSWORD
    }
)

if login_response.status_code != 200:
    print(f"❌ ログインエラー: {login_response.status_code}")
    print(login_response.text)
    exit(1)

auth_data = login_response.json()
access_token = auth_data["token"]
print("✅ ログイン成功!")

# コールバックを手動で実行
print("\n2. OAuth2コールバックを処理中...")
callback_response = requests.get(
    f"{API_BASE_URL}/crm/oauth/salesforce/callback",
    params={
        "code": code,
        "state": state
    },
    headers={"Authorization": f"Bearer {access_token}"}
)

if callback_response.status_code == 200:
    print("✅ OAuth2認証が完了しました!")
    result = callback_response.json()
    print(f"メッセージ: {result.get('message', 'Success')}")
else:
    print(f"❌ コールバックエラー: {callback_response.status_code}")
    print(callback_response.text)

# 認証状態を確認
print("\n3. 認証状態を確認中...")
status_response = requests.get(
    f"{API_BASE_URL}/crm/oauth/salesforce/status",
    headers={"Authorization": f"Bearer {access_token}"}
)

if status_response.status_code == 200:
    status = status_response.json()
    if status.get("is_authenticated"):
        print("✅ OAuth2認証が確認されました!")
        print(f"インスタンスURL: {status.get('instance_url')}")
        print(f"有効期限: {status.get('expires_at')}")
    else:
        print("⚠️ まだ認証されていません")
else:
    print(f"❌ 状態確認エラー: {status_response.status_code}")