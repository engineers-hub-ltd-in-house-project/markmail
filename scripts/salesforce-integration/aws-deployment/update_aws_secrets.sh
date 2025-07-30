#!/bin/bash

# AWS Secrets Managerの更新スクリプト
# 使用方法: ./update_aws_secrets.sh <profile>

PROFILE=${1:-default}

echo "現在のシークレットを取得中..."

# 現在のシークレットを取得
CURRENT_SECRET=$(aws secretsmanager get-secret-value \
  --secret-id markmail-dev-app-secret \
  --query 'SecretString' \
  --output text \
  --profile "$PROFILE")

# Salesforce環境変数を追加
# 注意: 実際のクライアントIDとシークレットに置き換えてください
UPDATED_SECRET=$(echo "$CURRENT_SECRET" | jq '. + {
  "SALESFORCE_CLIENT_ID": "your-salesforce-client-id",
  "SALESFORCE_CLIENT_SECRET": "your-salesforce-client-secret"
}')

echo "シークレットを更新中..."

# シークレットを更新
aws secretsmanager update-secret \
  --secret-id markmail-dev-app-secret \
  --secret-string "$UPDATED_SECRET" \
  --profile "$PROFILE"

if [ $? -eq 0 ]; then
    echo "✅ シークレットの更新に成功しました"
    
    echo "ECSサービスを再デプロイ中..."
    
    # ECSサービスを再デプロイ（新しいデプロイメントを強制）
    aws ecs update-service \
      --cluster markmail-dev \
      --service markmail-dev-backend \
      --profile "$PROFILE" \
      --no-cli-pager
    
    if [ $? -eq 0 ]; then
        echo "✅ ECSサービスの再デプロイを開始しました"
    else
        echo "❌ ECSサービスの再デプロイに失敗しました"
    fi
else
    echo "❌ シークレットの更新に失敗しました"
fi

echo ""
echo "⚠️  注意事項:"
echo "1. SALESFORCE_CLIENT_ID と SALESFORCE_CLIENT_SECRET を実際の値に置き換えてください"
echo "2. これらの値はSalesforce Connected Appから取得できます"
echo "3. RDSへのデータ投入も忘れずに実行してください"