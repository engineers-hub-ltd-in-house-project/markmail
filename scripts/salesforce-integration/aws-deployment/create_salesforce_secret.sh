#!/bin/bash

# Salesforce用のシークレットを作成するスクリプト
# 使用方法: ./create_salesforce_secret.sh

echo "Salesforce用のシークレットを作成中..."

# 新しいシークレットを作成
# 注意: 実際のクライアントIDとシークレットに置き換えてください
aws secretsmanager create-secret \
  --name markmail-dev-salesforce-secret \
  --description "Salesforce integration credentials for MarkMail dev environment" \
  --secret-string '{
    "SALESFORCE_CLIENT_ID": "your-salesforce-client-id",
    "SALESFORCE_CLIENT_SECRET": "your-salesforce-client-secret"
  }' \
  --profile yusuke.sato

if [ $? -eq 0 ]; then
    echo "✅ Salesforce用シークレットの作成に成功しました"
    
    echo ""
    echo "⚠️  重要: 次のステップ"
    echo "1. インフラコードを更新してECSタスク定義にシークレットを追加"
    echo "2. 実際のSalesforce Connected AppのクライアントIDとシークレットに更新"
    echo ""
    echo "シークレットを更新するには:"
    echo "aws secretsmanager update-secret \\"
    echo "  --secret-id markmail-dev-salesforce-secret \\"
    echo "  --secret-string '{\"SALESFORCE_CLIENT_ID\": \"実際のID\", \"SALESFORCE_CLIENT_SECRET\": \"実際のシークレット\"}' \\"
    echo "  --profile yusuke.sato"
else
    echo "❌ シークレットの作成に失敗しました"
fi