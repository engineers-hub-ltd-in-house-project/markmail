# MarkMail ドメインセットアップガイド

このガイドでは、MarkMailアプリケーションに独自ドメインを設定し、SSL証明書を有効化する手順を説明します。

## 📋 前提条件

- AWS CLIがインストールされ、認証設定済み
- Route 53でドメインを管理する権限
- ACM（AWS Certificate Manager）で証明書を作成する権限

## 🚀 セットアップ手順

### 1. ドメインの取得（新規の場合）

#### Option A: Route 53でドメインを直接購入

```bash
# AWS CLIでドメインの価格を確認
aws route53domains list-prices --region us-east-1

# AWSコンソールから購入することを推奨
# https://console.aws.amazon.com/route53/home#DomainRegistration
```

#### Option B: 外部レジストラで購入済みの場合

外部で購入したドメインのネームサーバーをRoute 53に変更する必要があります。

### 2. Route 53ホストゾーンの作成

```bash
# 変数設定
DOMAIN_NAME="markmail.example.com"  # あなたのドメインに置き換えてください
BASE_DOMAIN="example.com"           # ベースドメイン

# ホストゾーンの作成
aws route53 create-hosted-zone \
  --name $BASE_DOMAIN \
  --caller-reference $(date +%s) \
  --hosted-zone-config Comment="MarkMail Application Domain"
```

作成されたホストゾーンIDとネームサーバーを確認：

```bash
# ホストゾーン情報の取得
aws route53 list-hosted-zones-by-name \
  --dns-name $BASE_DOMAIN \
  --query "HostedZones[0].[Id,Name]" \
  --output table

# ネームサーバーの取得
HOSTED_ZONE_ID=$(aws route53 list-hosted-zones-by-name \
  --dns-name $BASE_DOMAIN \
  --query "HostedZones[0].Id" \
  --output text | sed 's/\/hostedzone\///')

aws route53 get-hosted-zone \
  --id $HOSTED_ZONE_ID \
  --query "DelegationSet.NameServers" \
  --output table
```

### 3. ドメインのネームサーバー設定

外部レジストラで購入した場合は、取得したネームサーバーをレジストラの管理画面で設定してください。

Route 53で購入した場合は自動的に設定されます。

### 4. 環境変数の設定

```bash
# 本番環境用
export PROD_DOMAIN="markmail.example.com"

# ステージング環境用（オプション）
export STAGING_DOMAIN="staging.markmail.example.com"

# その他の必要な環境変数
export ENVIRONMENT_NAME="prod"  # または "staging"
export NOTIFICATION_EMAIL="admin@example.com"
export GITHUB_OWNER="engineers-hub-ltd-in-house-project"
export GITHUB_REPO="markmail"
export GITHUB_BRANCH="main"
```

### 5. CDKスタックのデプロイ

```bash
cd infrastructure

# 依存関係の確認
npm install

# CDKブートストラップ（初回のみ）
npm run cdk bootstrap

# ALBスタックをデプロイ（SSL証明書が自動作成されます）
npm run deploy:alb

# または全スタックを順番にデプロイ
./deploy-sequential.sh
```

### 6. 証明書の検証

ACM証明書の作成と検証には通常5-30分かかります。

```bash
# 証明書の状態を確認
aws acm list-certificates --region $AWS_REGION

# 詳細情報の取得
CERT_ARN=$(aws acm list-certificates \
  --region $AWS_REGION \
  --query "CertificateSummaryList[?DomainName=='$PROD_DOMAIN'].CertificateArn" \
  --output text)

aws acm describe-certificate \
  --certificate-arn $CERT_ARN \
  --region $AWS_REGION \
  --query "Certificate.Status"
```

### 7. デプロイ完了後の確認

```bash
# ALBのDNS名を取得
ALB_DNS=$(aws cloudformation describe-stacks \
  --stack-name MarkMail-$ENVIRONMENT_NAME-ALBStack \
  --query "Stacks[0].Outputs[?OutputKey=='LoadBalancerDNS'].OutputValue" \
  --output text)

echo "ALB DNS: $ALB_DNS"

# Route 53レコードの確認
aws route53 list-resource-record-sets \
  --hosted-zone-id $HOSTED_ZONE_ID \
  --query "ResourceRecordSets[?Name=='$PROD_DOMAIN.']"
```

### 8. アプリケーションへのアクセス

デプロイが完了し、DNSが伝播したら（通常5-15分）、以下のURLでアクセスできます：

- 本番環境: `https://markmail.example.com`
- ステージング環境: `https://staging.markmail.example.com`

## 🔒 SSL/TLS設定の詳細

CDKスタックは自動的に以下を設定します：

1. **ACM証明書**

   - ドメイン名とワイルドカード（\*.example.com）をカバー
   - DNS検証を使用（自動更新対応）

2. **ALB設定**

   - HTTPS（443）リスナー
   - HTTP（80）からHTTPSへの自動リダイレクト
   - TLS 1.2以上のみ許可

3. **セキュリティグループ**
   - HTTP（80）とHTTPS（443）のインバウンドを許可

## 🔧 トラブルシューティング

### 証明書の検証が完了しない

1. Route 53ホストゾーンが正しく設定されているか確認
2. ドメインのネームサーバーがRoute 53を指しているか確認
3. DNS伝播を待つ（最大48時間かかる場合があります）

```bash
# DNSレコードの確認
dig $PROD_DOMAIN

# Route 53の検証レコードを確認
aws route53 list-resource-record-sets \
  --hosted-zone-id $HOSTED_ZONE_ID \
  --query "ResourceRecordSets[?Type=='CNAME']"
```

### アプリケーションにアクセスできない

1. ALBのヘルスチェックを確認
2. ECSタスクが正常に動作しているか確認
3. セキュリティグループの設定を確認

```bash
# ALBターゲットグループのヘルス状態
aws elbv2 describe-target-health \
  --target-group-arn $(aws elbv2 describe-target-groups \
    --names "markmail-*" \
    --query "TargetGroups[0].TargetGroupArn" \
    --output text)
```

## 📝 追加設定

### サブドメインの追加

```bash
# api.markmail.example.com などのサブドメインを追加
aws route53 change-resource-record-sets \
  --hosted-zone-id $HOSTED_ZONE_ID \
  --change-batch file://subdomain-record.json
```

### メール送信用ドメインの設定（SES）

MarkMailはメール送信にAWS SESを使用します。ドメインの検証が必要です：

```bash
# SESでドメインを検証
aws ses put-identity-dkim-enabled \
  --identity $BASE_DOMAIN \
  --dkim-enabled \
  --region $AWS_REGION

# DKIMトークンの取得
aws ses get-identity-dkim-attributes \
  --identities $BASE_DOMAIN \
  --region $AWS_REGION
```

## 🎉 完了

これで、MarkMailアプリケーションに独自ドメインとSSL証明書が設定されました。
安全なHTTPS接続でアプリケーションにアクセスできます。

---

質問や問題がある場合は、[Issues](https://github.com/engineers-hub-ltd-in-house-project/markmail/issues)を作成してください。
