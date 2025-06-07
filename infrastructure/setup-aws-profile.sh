#!/bin/bash

# 色付き出力用の関数
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔐 AWS SSO プロファイル設定スクリプト${NC}"
echo ""
echo -e "${BLUE}📝 このスクリプトは 'aws configure sso' のインタラクティブウィザードを実行します${NC}"
echo ""

# プロファイル名の入力
echo -e "${YELLOW}プロファイル名を入力してください (例: markmail-dev):${NC}"
read -r PROFILE_NAME
if [ -z "$PROFILE_NAME" ]; then
    PROFILE_NAME="markmail-dev"
fi

echo ""
echo -e "${YELLOW}🚀 AWS SSO設定ウィザードを開始します...${NC}"
echo -e "${BLUE}以下の情報を入力してください:${NC}"
echo -e "  1. SSO session name (任意の名前)"
echo -e "  2. SSO start URL"
echo -e "  3. SSO region (通常は us-east-1)"
echo -e "  4. SSO registration scopes (デフォルトでEnter)"
echo -e "  5. ブラウザが開いたらログイン"
echo -e "  6. AWSアカウントとロールを選択"
echo -e "  7. CLI default client Region"
echo -e "  8. CLI default output format"
echo -e "  9. CLI profile name: ${GREEN}${PROFILE_NAME}${NC}"
echo ""

# AWS SSO設定ウィザードを実行
aws configure sso --profile "${PROFILE_NAME}"

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ SSO設定が完了しました${NC}"
    echo ""

    echo -e "${YELLOW}🔐 SSOログインをテストしますか？ (y/n)${NC}"
    read -r test_login
    if [[ "$test_login" == "y" ]]; then
        echo -e "${YELLOW}🚀 SSOログインを開始します...${NC}"
        aws sso login --profile "${PROFILE_NAME}"
        
        if [ $? -eq 0 ]; then
            echo ""
            echo -e "${GREEN}✅ ログインが完了しました！${NC}"
        else
            echo -e "${RED}❌ ログインに失敗しました${NC}"
            echo -e "${YELLOW}💡 ヒント: ブラウザでSSOポータルにログインする必要があります${NC}"
        fi
    fi
    
    echo ""
    echo -e "${BLUE}📝 使い方:${NC}"
    echo -e "  ${GREEN}# SSOログイン${NC}"
    echo -e "  aws sso login --profile ${PROFILE_NAME}"
    echo ""
    echo -e "  ${GREEN}# プロファイルを使用してコマンドを実行${NC}"
    echo -e "  aws s3 ls --profile ${PROFILE_NAME}"
    echo ""
    echo -e "  ${GREEN}# 環境変数でプロファイルを設定${NC}"
    echo -e "  export AWS_PROFILE=${PROFILE_NAME}"
    echo ""
    echo -e "  ${GREEN}# CDKデプロイで使用${NC}"
    echo -e "  export AWS_PROFILE=${PROFILE_NAME}"
    echo -e "  npm run deploy"
    echo ""
    
    # .envファイルの作成オプション
    echo -e "${YELLOW}.envファイルを作成しますか？ (y/n)${NC}"
    read -r create_env
    if [[ "$create_env" == "y" ]]; then
        # デフォルトリージョンを取得
        DEFAULT_REGION=$(aws configure get region --profile "${PROFILE_NAME}")
        
        cat > .env << EOL
# AWS Profile設定
AWS_PROFILE=${PROFILE_NAME}
AWS_REGION=${DEFAULT_REGION}

# SES設定（オプション）
# SES_DOMAIN=mail.example.com
# NOTIFICATION_EMAIL=admin@example.com

# 環境
ENVIRONMENT=development
EOL
        echo -e "${GREEN}✅ .envファイルを作成しました${NC}"
    fi
    
    # deploy.shの更新オプション
    if [ -f "deploy.sh" ]; then
        echo -e "${YELLOW}deploy.shにプロファイル設定を追加しますか？ (y/n)${NC}"
        read -r update_deploy
        if [[ "$update_deploy" == "y" ]]; then
            # deploy.shの先頭にプロファイル設定を追加
            sed -i.bak '2i\
\
# AWS Profile設定\
if [ -z "$AWS_PROFILE" ]; then\
    export AWS_PROFILE='${PROFILE_NAME}'\
fi\
' deploy.sh
            echo -e "${GREEN}✅ deploy.shを更新しました${NC}"
        fi
    fi
else
    echo -e "${RED}❌ SSO設定に失敗しました${NC}"
    exit 1
fi