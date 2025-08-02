#\!/bin/bash
echo "Salesforce CLI認証を確認中..."

# 現在の認証状態を確認
sf org display --target-org eng-hub-ltd-test-org --json | jq -r '.result.accessToken' > /tmp/sf_token.txt

if [ -s /tmp/sf_token.txt ]; then
    TOKEN=$(cat /tmp/sf_token.txt)
    echo "新しいアクセストークンを取得しました"
    
    # .envファイルを更新
    sed -i "s|SALESFORCE_ACCESS_TOKEN=.*|SALESFORCE_ACCESS_TOKEN=$TOKEN|" .env
    echo ".envファイルを更新しました"
    
    # 新しいトークンを表示
    echo "新しいトークン: $TOKEN"
else
    echo "エラー: アクセストークンを取得できませんでした"
    echo "sf auth:web:loginを実行して再認証してください"
fi
