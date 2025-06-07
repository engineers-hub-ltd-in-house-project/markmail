# MarkMail Infrastructure - Full AWS Deployment

このディレクトリには、MarkMailのAWSインフラストラクチャをCDKで定義したコードが含まれています。

## アーキテクチャ概要

### 含まれるAWSリソース

#### ネットワーク層

- **VPC**: Multi-AZ構成のVPC
- **Subnets**: Public、Private、Isolated サブネット
- **Security Groups**: ALB、ECS、RDS、ElastiCache用

#### データ層

- **RDS (PostgreSQL)**: アプリケーションデータベース
- **ElastiCache (Redis)**: セッション管理とキャッシュ
- **Secrets Manager**: データベース認証情報の管理

#### コンピューティング層

- **ECS Fargate**: コンテナ実行環境
- **ALB**: ロードバランサー
- **ECR**: Dockerイメージレジストリ

#### CI/CD

- **CodePipeline**: 自動デプロイパイプライン
- **CodeBuild**: ビルド環境
- **GitHub連携**: ソースコード管理

#### セキュリティ・監視

- **WAF**: Webアプリケーションファイアウォール（本番環境のみ）
- **CloudWatch**: ログとメトリクス
- **SNS**: アラート通知

#### メール送信

- **SES**: メール送信サービス
- **Configuration Set**: バウンス・苦情処理

## 前提条件

1. AWS CLI がインストールされ、設定されていること
2. Node.js 18以上がインストールされていること
3. AWS CDK CLI がインストールされていること
   ```bash
   npm install -g aws-cdk
   ```
4. GitHubリポジトリへのアクセス権限があること

## セットアップ

### 1. 環境変数の設定

以下の環境変数を設定してください：

```bash
# 必須
export NOTIFICATION_EMAIL="admin@example.com"
export GITHUB_OWNER="your-github-username"
export GITHUB_REPO="your-repo-name"

# 環境別（オプション）
export STAGING_DOMAIN="staging.example.com"  # Staging環境用
export PROD_DOMAIN="example.com"            # 本番環境用
```

### 2. GitHub 連携の準備

AWS CodeConnections（旧CodeStar Connections）を使用してGitHubと連携します。初回デプロイ後、AWS Consoleで接続の承認が必要です。

**注意**: 2025年4月以降、旧CodeStar Connections APIは廃止されるため、新しいCodeConnections APIを使用しています。

### 3. 依存関係のインストール

```bash
cd infrastructure
npm install
```

## デプロイ

### 開発環境へのデプロイ

```bash
./deploy-new.sh dev
```

### ステージング環境へのデプロイ

```bash
export STAGING_DOMAIN="staging.example.com"
./deploy-new.sh staging
```

### 本番環境へのデプロイ

```bash
export PROD_DOMAIN="example.com"
./deploy-new.sh prod
```

## 環境別の設定

| 環境    | ECS タスク数 | CPU  | メモリ  | RDS インスタンス | NAT Gateway |
| ------- | ------------ | ---- | ------- | ---------------- | ----------- |
| dev     | 1            | 512  | 1024 MB | t3.micro         | 1           |
| staging | 2            | 512  | 1024 MB | t3.micro         | 1           |
| prod    | 3            | 1024 | 2048 MB | t3.small         | 2           |

## デプロイ後の作業

### 1. CodeConnection の承認

初回デプロイ後、GitHub連携を有効化：

1. AWS CodePipeline コンソールにアクセス
2. 作成されたパイプライン `markmail-<環境名>` を開く
3. エラーが表示されている Source ステージをクリック
4. 「Update pending connection」または「接続を更新」をクリック
5. GitHubにリダイレクトされるので、AWS CodeConnectionsアプリケーションを承認
6. 接続が「Available」ステータスになることを確認
7. パイプラインの「Release change」をクリックして再実行

**重要**: CodeConnectionsは環境ごとに個別の接続が必要です。各環境（dev/staging/prod）で上記手順を実行してください。

### 2. DNS設定（カスタムドメイン使用時）

CloudFormationの出力からALBのDNS名を取得し、Route53またはDNSプロバイダーでCNAMEレコードを設定。

### 3. SES設定

1. AWSコンソールからSESにアクセス
2. 使用するドメインまたはメールアドレスを検証
3. 本番環境の場合、サンドボックスからの移行をリクエスト

### 3. アプリケーション設定

バックエンドの `.env` ファイルに以下を追加：

```env
# RDS
DATABASE_URL=<CloudFormation出力から取得>

# Redis
REDIS_URL=<CloudFormation出力から取得>

# SES
AWS_SES_CONFIGURATION_SET=markmail-<environment>
AWS_REGION=ap-northeast-1
```

### 4. Dockerイメージのプッシュ

```bash
# ECRにログイン
aws ecr get-login-password --region ap-northeast-1 | \
  docker login --username AWS --password-stdin \
  <account-id>.dkr.ecr.ap-northeast-1.amazonaws.com

# バックエンドイメージのビルドとプッシュ
cd ../backend
docker build -t markmail-backend .
docker tag markmail-backend:latest \
  <backend-repo-uri>:latest
docker push <backend-repo-uri>:latest

# フロントエンドイメージのビルドとプッシュ
cd ../frontend
docker build -t markmail-frontend \
  --build-arg VITE_API_URL=https://<your-domain>/api .
docker tag markmail-frontend:latest \
  <frontend-repo-uri>:latest
docker push <frontend-repo-uri>:latest
```

## トラブルシューティング

### CDK Bootstrap エラー

```bash
npx cdk bootstrap aws://<account-id>/ap-northeast-1
```

### スタック削除

```bash
npx cdk destroy -c environment=<env-name>
```

### ログの確認

```bash
# ECSタスクログ
aws logs tail /ecs/markmail-<env> --follow

# CodeBuildログ
aws logs tail /aws/codebuild/markmail-<env>-backend-build --follow
```

## コスト見積もり

### 開発環境（月額）

- VPC/NAT Gateway: $45
- ECS Fargate: $20
- RDS (t3.micro): $15
- ElastiCache: $13
- ALB: $25
- その他: $10
- **合計: 約$128**

### 本番環境（月額）

- VPC/NAT Gateway: $90
- ECS Fargate: $120
- RDS (t3.small + Multi-AZ): $70
- ElastiCache: $50
- ALB: $25
- WAF: $10
- その他: $20
- **合計: 約$385**

※実際のコストは使用量により変動します

## アーキテクチャ図

```
┌─────────────────────────────────────────────────────────────┐
│                         Internet                             │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
                    ┌─────────────────────┐
                    │    Route 53         │
                    │  (DNS Resolution)   │
                    └─────────────────────┘
                                │
                                ▼
                    ┌─────────────────────┐
                    │    WAF (Prod)       │
                    │  (Web Firewall)     │
                    └─────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                        VPC (10.0.0.0/16)                     │
│                                                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │              Public Subnets (Multi-AZ)             │     │
│  │                                                    │     │
│  │    ┌─────────────┐         ┌─────────────┐       │     │
│  │    │     ALB     │         │  NAT Gateway │       │     │
│  │    └─────────────┘         └─────────────┘       │     │
│  └────────────────────────────────────────────────────┘     │
│                        │                                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │             Private Subnets (Multi-AZ)             │     │
│  │                                                    │     │
│  │    ┌─────────────┐         ┌─────────────┐       │     │
│  │    │ ECS Fargate │         │ ECS Fargate │       │     │
│  │    │  (Backend)  │         │ (Frontend)  │       │     │
│  │    └─────────────┘         └─────────────┘       │     │
│  └────────────────────────────────────────────────────┘     │
│                        │                                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │            Isolated Subnets (Multi-AZ)             │     │
│  │                                                    │     │
│  │    ┌─────────────┐         ┌─────────────┐       │     │
│  │    │     RDS     │         │ ElastiCache │       │     │
│  │    │ PostgreSQL  │         │    Redis    │       │     │
│  │    └─────────────┘         └─────────────┘       │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴────────────┐
                    │                        │
            ┌───────▼────────┐      ┌───────▼────────┐
            │  CloudWatch    │      │  Secrets       │
            │  Logs/Metrics  │      │  Manager       │
            └────────────────┘      └────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                        CI/CD Pipeline                        │
│                                                              │
│  GitHub → CodePipeline → CodeBuild → ECR → ECS Deploy      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     Email Infrastructure                     │
│                                                              │
│         SES → Configuration Set → SNS Topics                │
└─────────────────────────────────────────────────────────────┘
```
