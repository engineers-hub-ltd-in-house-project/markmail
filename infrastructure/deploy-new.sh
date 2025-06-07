#!/bin/bash

# エラーで停止
set -e

# 色付き出力用の関数
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 環境名を引数から取得（デフォルトはdev）
ENVIRONMENT=${1:-dev}

echo -e "${GREEN}🚀 MarkMail Infrastructure デプロイスクリプト${NC}"
echo -e "${GREEN}環境: ${ENVIRONMENT}${NC}"
echo ""

# 環境変数の検証
if [[ "$ENVIRONMENT" == "prod" ]]; then
    if [[ -z "$PROD_DOMAIN" ]]; then
        echo -e "${RED}Error: PROD_DOMAIN environment variable is required for production deployment${NC}"
        exit 1
    fi
elif [[ "$ENVIRONMENT" == "staging" ]]; then
    if [[ -z "$STAGING_DOMAIN" ]]; then
        echo -e "${RED}Error: STAGING_DOMAIN environment variable is required for staging deployment${NC}"
        exit 1
    fi
fi

if [[ -z "$NOTIFICATION_EMAIL" ]]; then
    echo -e "${RED}Error: NOTIFICATION_EMAIL environment variable is required${NC}"
    exit 1
fi

if [[ -z "$GITHUB_OWNER" ]] || [[ -z "$GITHUB_REPO" ]]; then
    echo -e "${YELLOW}Warning: GITHUB_OWNER or GITHUB_REPO not set, using defaults${NC}"
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
REGION=$(aws configure get region || echo "ap-northeast-1")
echo -e "${GREEN}✅ AWSアカウント: ${ACCOUNT_ID}${NC}"
echo -e "${GREEN}✅ リージョン: ${REGION}${NC}"
echo ""

# 依存関係のインストール
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}📦 依存関係をインストール中...${NC}"
    npm install
fi

# TypeScriptのビルド
echo -e "${YELLOW}🔨 TypeScriptをビルド中...${NC}"
npm run build
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ ビルドに失敗しました${NC}"
    exit 1
fi

# CDK Bootstrap（初回のみ必要）
echo -e "${YELLOW}🏗️  CDK Bootstrapを確認中...${NC}"
npx cdk bootstrap aws://${ACCOUNT_ID}/${REGION} 2>&1 | grep -v "already bootstrapped" || true

# CodeConnections の注意事項を表示
echo -e "${YELLOW}📌 注意: AWS CodeConnections を使用したGitHub連携${NC}"
echo "初回デプロイ後、AWS ConsoleでCodeConnectionの承認が必要です。"
echo "1. AWS CodePipeline コンソールに移動"
echo "2. 作成されたパイプラインを開く"
echo "3. Source ステージのエラーをクリック"
echo "4. 'Update pending connection' または '接続を更新' をクリック"
echo "5. GitHubで認証・承認を完了"
echo ""

# CDK Synthesize
echo -e "${YELLOW}📋 CloudFormationテンプレートを生成中...${NC}"
npx cdk synth -c environment=$ENVIRONMENT

# 差分確認
echo ""
echo -e "${YELLOW}📊 変更内容を確認中...${NC}"
npx cdk diff -c environment=$ENVIRONMENT || true

# 確認プロンプト
echo ""
if [[ "$ENVIRONMENT" == "prod" ]]; then
    echo -e "${RED}⚠️  WARNING: You are about to deploy to PRODUCTION!${NC}"
fi
echo -e "${YELLOW}この内容でデプロイしますか？ (yes/no):${NC}"
read -r confirm
if [[ "$confirm" != "yes" ]]; then
    echo -e "${RED}❌ デプロイをキャンセルしました${NC}"
    exit 0
fi

# CDK Deploy
echo ""
echo -e "${GREEN}🚀 デプロイを開始します...${NC}"
npx cdk deploy -c environment=$ENVIRONMENT --require-approval never

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ デプロイが完了しました！${NC}"
    echo ""
    
    # デプロイ結果の取得
    echo -e "${YELLOW}📊 スタック出力:${NC}"
    aws cloudformation describe-stacks \
        --stack-name "MarkMail-${ENVIRONMENT}-Stack" \
        --query 'Stacks[0].Outputs' \
        --output table
    
    # ECRログイン情報の表示
    echo ""
    echo -e "${YELLOW}📝 ECRにDockerイメージをプッシュするには:${NC}"
    echo "aws ecr get-login-password --region ${REGION} | docker login --username AWS --password-stdin ${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"
    
    echo ""
    echo -e "${YELLOW}📝 次のステップ:${NC}"
    echo "1. 上記のスタック出力からエンドポイント情報を確認"
    echo "2. 必要に応じてRoute53でDNSレコードを設定"
    echo "3. AWS SESコンソールでドメイン/メールアドレスを検証"
    echo "4. 必要に応じて本番環境アクセスをリクエスト（SESサンドボックスの解除）"
    echo "5. アプリケーションの環境変数を更新"
else
    echo -e "${RED}❌ デプロイに失敗しました${NC}"
    exit 1
fi