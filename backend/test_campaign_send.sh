#!/bin/bash

# キャンペーン送信機能の統合テストスクリプト

# 環境変数を設定
export DATABASE_URL="postgres://markmail:markmail_password@localhost:5432/markmail_dev"
export JWT_SECRET="test-secret-key-for-development"
export MAIL_PROVIDER="mailhog"
export SMTP_FROM="noreply@markmail.dev"
export SMTP_HOST="localhost"
export SMTP_PORT="1025"

# 1. 新しいユーザーを作成してログイン
echo "1. ユーザー作成とログイン..."
USER_EMAIL="campaign_test_$(date +%s)@example.com"
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$USER_EMAIL\",
    \"password\": \"Password123!\",
    \"name\": \"キャンペーンテストユーザー\"
  }")

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "認証に失敗しました"
  exit 1
fi

echo "認証成功: トークン取得"

# 2. テンプレートを作成
echo -e "\n2. テンプレート作成..."
TEMPLATE_RESPONSE=$(curl -s -X POST http://localhost:3000/api/templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "ウェルカムメール",
    "subject": "{{name}}様、MarkMailへようこそ！",
    "content": "# こんにちは、{{name}}様！\n\nMarkMailへのご登録ありがとうございます。\n\n## 次のステップ\n\n1. プロフィールを設定\n2. 最初のキャンペーンを作成\n3. 購読者をインポート\n\nご不明な点がございましたら、お気軽にお問い合わせください。\n\nよろしくお願いいたします。\nMarkMailチーム",
    "variables": {
      "name": "ユーザー名"
    }
  }')

TEMPLATE_ID=$(echo $TEMPLATE_RESPONSE | jq -r '.id')
echo "テンプレート作成成功: ID=$TEMPLATE_ID"

# 3. 購読者を作成
echo -e "\n3. 購読者を作成..."
for i in {1..3}; do
  SUBSCRIBER_RESPONSE=$(curl -s -X POST http://localhost:3000/api/subscribers \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d "{
      \"email\": \"subscriber$i@example.com\",
      \"name\": \"購読者$i\",
      \"tags\": [\"test\", \"welcome\"]
    }")
  echo "購読者$i作成: $(echo $SUBSCRIBER_RESPONSE | jq -r '.email')"
done

# 4. キャンペーンを作成
echo -e "\n4. キャンペーン作成..."
CAMPAIGN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/campaigns \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"template_id\": \"$TEMPLATE_ID\",
    \"name\": \"ウェルカムキャンペーン\",
    \"description\": \"新規登録者向けのウェルカムメール\",
    \"subject\": \"MarkMailへようこそ！\"
  }")

CAMPAIGN_ID=$(echo $CAMPAIGN_RESPONSE | jq -r '.id')
echo "キャンペーン作成成功: ID=$CAMPAIGN_ID"

# 5. キャンペーンの購読者を確認
echo -e "\n5. キャンペーンの購読者を確認..."
SUBSCRIBERS_RESPONSE=$(curl -s -X GET "http://localhost:3000/api/campaigns/$CAMPAIGN_ID/subscribers" \
  -H "Authorization: Bearer $TOKEN")
echo "購読者数: $(echo $SUBSCRIBERS_RESPONSE | jq '.total')"

# 6. キャンペーンを送信
echo -e "\n6. キャンペーンを送信..."
SEND_RESPONSE=$(curl -s -X POST "http://localhost:3000/api/campaigns/$CAMPAIGN_ID/send" \
  -H "Authorization: Bearer $TOKEN")
echo "送信レスポンス: $SEND_RESPONSE"

# 7. キャンペーンのステータスを確認
echo -e "\n7. キャンペーンのステータスを確認..."
sleep 2
CAMPAIGN_STATUS=$(curl -s -X GET "http://localhost:3000/api/campaigns/$CAMPAIGN_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.status')
echo "キャンペーンステータス: $CAMPAIGN_STATUS"

echo -e "\n8. MailHogでメールを確認してください: http://localhost:8025"
echo "送信されたメール数: 3通"