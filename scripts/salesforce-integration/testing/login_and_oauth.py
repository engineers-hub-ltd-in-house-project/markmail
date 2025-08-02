#!/usr/bin/env python3
import requests
import json
import os

# API設定
API_BASE_URL = "http://localhost:3000/api"

# 環境変数から認証情報を取得
EMAIL = os.getenv("MARKMAIL_TEST_EMAIL", "yusuke.sato@engineers-hub.ltd")
PASSWORD = os.getenv("MARKMAIL_TEST_PASSWORD")

if not PASSWORD:
    print("❌ エラー: 環境変数 MARKMAIL_TEST_PASSWORD が設定されていません")
    print("使用方法: MARKMAIL_TEST_PASSWORD=your_password python3 login_and_oauth.py")
    exit(1)

# 1. ログイン
print("1. ログイン中...")
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
if "token" in auth_data:
    access_token = auth_data["token"]
    print("✅ ログイン成功!")
else:
    print(f"❌ ログインレスポンスにトークンがありません")
    print(f"レスポンス: {auth_data}")
    exit(1)

# 2. OAuth2認証状態を確認
print("\n2. OAuth2認証状態を確認中...")
status_response = requests.get(
    f"{API_BASE_URL}/crm/oauth/salesforce/status",
    headers={"Authorization": f"Bearer {access_token}"}
)

if status_response.status_code == 200:
    status = status_response.json()
    if status.get("is_authenticated"):
        print("✅ 既にOAuth2認証済みです")
        print(f"インスタンスURL: {status['instance_url']}")
        print(f"有効期限: {status['expires_at']}")
    else:
        print("⚠️ OAuth2認証が必要です")
        
        # 3. OAuth2認証を開始
        print("\n3. OAuth2認証を開始中...")
        init_response = requests.get(
            f"{API_BASE_URL}/crm/oauth/salesforce/init",
            headers={"Authorization": f"Bearer {access_token}"}
        )
        
        if init_response.status_code == 200:
            init_data = init_response.json()
            print("\n🌐 以下のURLをブラウザで開いて認証してください:")
            print(init_data["auth_url"])
            print("\n認証が完了したら、このスクリプトを再実行してください。")
else:
    print(f"❌ 認証状態確認エラー: {status_response.status_code}")
    print(status_response.text)