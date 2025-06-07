#!/bin/bash

# テストメール送信スクリプト

# 環境変数を設定
export DATABASE_URL="postgres://markmail:markmail_password@localhost:5432/markmail_dev"
export JWT_SECRET="test-secret-key-for-development"
export MAIL_PROVIDER="mailhog"
export SMTP_FROM="noreply@markmail.dev"
export SMTP_HOST="localhost"
export SMTP_PORT="1025"

# ユーザー登録
echo "1. ユーザー登録..."
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "mailtest@example.com",
    "password": "Password123!",
    "name": "メールテストユーザー"
  }')

echo "登録レスポンス: $REGISTER_RESPONSE"

# JWTトークンを抽出
TOKEN=$(echo $REGISTER_RESPONSE | jq -r '.token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "トークンの取得に失敗しました。ログインを試みます..."
  
  # ログイン
  LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{
      "email": "mailtest@example.com",
      "password": "Password123!"
    }')
  
  echo "ログインレスポンス: $LOGIN_RESPONSE"
  TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.token')
fi

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "認証に失敗しました"
  exit 1
fi

echo "認証トークン取得成功: $TOKEN"

# テストメール送信
echo -e "\n2. テストメール送信..."
EMAIL_RESPONSE=$(curl -s -X POST http://localhost:3000/api/email/test \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "to": "recipient@example.com",
    "subject": "MarkMailテストメール",
    "content": "これはMarkMailからのテストメールです。MailHog経由で送信されています。"
  }')

echo "メール送信レスポンス: $EMAIL_RESPONSE"

echo -e "\n3. MailHogでメールを確認してください: http://localhost:8025"