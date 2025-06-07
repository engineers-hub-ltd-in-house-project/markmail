#!/bin/bash

# AWS Profile設定
if [ -z "$AWS_PROFILE" ]; then
    export AWS_PROFILE=yusuke.sato
fi


# 色付き出力用の関数
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 MarkMail Infrastructure デプロイスクリプト${NC}"
echo ""

# 環境変数のチェック
if [ -f .env ]; then
    echo -e "${YELLOW}📋 .envファイルを読み込み中...${NC}"
    export $(cat .env | grep -v '^#' | xargs)
fi

# AWS認証情報の確認
echo -e "${YELLOW}🔐 AWS認証情報を確認中...${NC}"
aws sts get-caller-identity > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ AWS認証情報が設定されていません。'aws configure'を実行してください。${NC}"
    exit 1
fi

# AWS情報を表示
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
REGION=$(aws configure get region)
echo -e "${GREEN}✅ AWSアカウント: ${ACCOUNT_ID}${NC}"
echo -e "${GREEN}✅ リージョン: ${REGION}${NC}"
echo ""

# ビルド
echo -e "${YELLOW}🔨 TypeScriptをビルド中...${NC}"
npm run build
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ ビルドに失敗しました${NC}"
    exit 1
fi

# CDK Bootstrap（必要な場合）
echo -e "${YELLOW}🏗️  CDK Bootstrapを確認中...${NC}"
npx cdk bootstrap aws://${ACCOUNT_ID}/${REGION} 2>&1 | grep -v "already bootstrapped"

# 差分確認
echo ""
echo -e "${YELLOW}📊 変更内容を確認中...${NC}"
npm run diff

# デプロイ確認
echo ""
echo -e "${YELLOW}この内容でデプロイしますか？ (y/n)${NC}"
read -r response
if [[ "$response" != "y" ]]; then
    echo -e "${RED}❌ デプロイをキャンセルしました${NC}"
    exit 0
fi

# デプロイ実行
echo ""
echo -e "${GREEN}🚀 デプロイを開始します...${NC}"
npm run deploy -- --require-approval never

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ デプロイが完了しました！${NC}"
    echo ""
    echo -e "${YELLOW}📝 次のステップ:${NC}"
    echo "1. AWS SESコンソールでドメイン/メールアドレスを検証"
    echo "2. 必要に応じて本番環境アクセスをリクエスト"
    echo "3. CloudFormation OutputsからアクセスキーをコピーしてBackendの.envに設定"
    echo ""
    echo -e "${YELLOW}📊 スタック情報を表示:${NC}"
    aws cloudformation describe-stacks \
        --stack-name MarkMailInfrastructureStack \
        --query "Stacks[0].Outputs[?OutputKey=='SESUserAccessKeyId' || OutputKey=='ConfigurationSetName'].{Key:OutputKey,Value:OutputValue}" \
        --output table
else
    echo -e "${RED}❌ デプロイに失敗しました${NC}"
    exit 1
fi