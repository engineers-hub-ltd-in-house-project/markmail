#!/bin/bash

# OAuth2認証URLを直接ブラウザで開く
# 注意: このClient IDは古いアプリケーションのもので、現在は使用されていません
# 現在の設定は backend/.env ファイルを参照してください
CLIENT_ID="3MVG9q4K8Dm94dAymlhhGfjdWpMRIgZuHeNAUuUrCSIMiskmExZZZzjP2ziEG11WQiu_PxvTAzu2Od7pZy.Bz"
REDIRECT_URI="http://localhost:3000/api/crm/oauth/salesforce/callback"
STATE="test123"

# URLエンコード
ENCODED_REDIRECT_URI=$(echo -n "$REDIRECT_URI" | jq -sRr @uri)

# OAuth URLを生成
OAUTH_URL="https://customization-computing-9993.my.salesforce.com/services/oauth2/authorize?response_type=code&client_id=${CLIENT_ID}&redirect_uri=${ENCODED_REDIRECT_URI}&state=${STATE}&scope=api%20refresh_token%20offline_access"

echo "OAuth2認証URL:"
echo "$OAUTH_URL"
echo ""
echo "このURLをブラウザで開いてください。"