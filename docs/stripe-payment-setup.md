# Stripe決済機能のセットアップとテスト手順

このドキュメントでは、MarkMailプロジェクトでStripe決済機能をセットアップし、テストする手順を説明します。

## 目次

1. [前提条件](#前提条件)
2. [Stripeアカウントの準備](#stripeアカウントの準備)
3. [Stripe CLIのインストール](#stripe-cliのインストール)
4. [環境変数の設定](#環境変数の設定)
5. [Stripeダッシュボードでの商品作成](#stripeダッシュボードでの商品作成)
6. [データベースの更新](#データベースの更新)
7. [開発環境でのテスト手順](#開発環境でのテスト手順)
8. [トラブルシューティング](#トラブルシューティング)
9. [本番環境への移行](#本番環境への移行)

## 前提条件

- Dockerが起動していること
- PostgreSQLデータベースが実行中であること
- バックエンドとフロントエンドの開発環境が構築済みであること

## Stripeアカウントの準備

1. [Stripe](https://stripe.com)にアクセスしてアカウントを作成
2. ダッシュボードにログイン後、右上の環境切り替えで「**テスト環境**」を選択
3. 「**開発者**」セクションから「**APIキー**」にアクセス

## Stripe CLIのインストール

### Linux/WSL環境

```bash
# GitHubから最新版をダウンロード
curl -L https://github.com/stripe/stripe-cli/releases/latest/download/stripe_1.27.0_linux_x86_64.tar.gz -o stripe-cli.tar.gz

# アーカイブを展開
tar -xzf stripe-cli.tar.gz

# バイナリを適切な場所に移動
mkdir -p ~/.local/bin
mv stripe ~/.local/bin/

# PATHに追加（.bashrcに追記）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# インストール確認
stripe --version
```

### macOS環境

```bash
# Homebrewを使用
brew install stripe/stripe-cli/stripe
```

## 環境変数の設定

### 1. Stripeダッシュボードから取得する情報

1. **APIキーの取得**
   - ダッシュボード > 開発者 > APIキー
   - **公開可能キー**: `pk_test_...` で始まるキー
   - **シークレットキー**: `sk_test_...` で始まるキー

### 2. backend/.envファイルの設定

```bash
# Stripe設定
STRIPE_SECRET_KEY=sk_test_xxxxxxxxxxxxxxxx  # Stripeダッシュボードから取得
STRIPE_PUBLISHABLE_KEY=pk_test_xxxxxxxxxxxxxxxx  # Stripeダッシュボードから取得
STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxxxxxx  # Stripe CLIから取得（後述）
STRIPE_CURRENCY=jpy
```

### 3. Webhook署名シークレットの取得

別のターミナルで以下を実行：

```bash
stripe listen --forward-to localhost:3000/api/stripe/webhook
```

出力される以下の形式の文字列をコピー：

```
Ready! Your webhook signing secret is whsec_xxxxxxxxxxxxxxxx
```

この `whsec_` で始まる文字列を `.env` ファイルの `STRIPE_WEBHOOK_SECRET`
に設定します。

**重要**: Stripe
CLIを再起動するたびに新しい署名シークレットが生成されるため、その都度 `.env`
ファイルを更新する必要があります。

## Stripeダッシュボードでの商品作成

1. Stripeダッシュボードにログイン
2. 「**商品カタログ**」にアクセス
3. 以下の3つの商品を作成：

### Free プラン

- 商品名: Free プラン
- 価格: ¥0/月
- 定期的な支払い: 月次

### Pro プラン

- 商品名: Pro プラン
- 価格: ¥4,980/月
- 定期的な支払い: 月次

### Business プラン

- 商品名: Business プラン
- 価格: ¥19,800/月
- 定期的な支払い: 月次

各商品作成後、表示される Price ID（`price_...` で始まる）をメモしておきます。

## データベースの更新

### 1. 作成した商品のIDを確認

```bash
# Stripe CLIを使用して商品情報を取得
stripe products list --limit 10
stripe prices list --limit 10
```

### 2. データベースのプラン情報を更新

```sql
-- Freeプラン
UPDATE subscription_plans
SET stripe_price_id = 'price_xxxxx',
    stripe_product_id = 'prod_xxxxx'
WHERE name = 'free';

-- Proプラン
UPDATE subscription_plans
SET stripe_price_id = 'price_xxxxx',
    stripe_product_id = 'prod_xxxxx'
WHERE name = 'pro';

-- Businessプラン
UPDATE subscription_plans
SET stripe_price_id = 'price_xxxxx',
    stripe_product_id = 'prod_xxxxx'
WHERE name = 'business';
```

## 開発環境でのテスト手順

### 1. 必要なサービスの起動（3つのターミナルが必要）

**ターミナル1 - バックエンド:**

```bash
cd backend
cargo watch -c -w src -w .env -x run
```

**ターミナル2 - フロントエンド:**

```bash
cd frontend
npm run dev
```

**ターミナル3 - Stripe Webhook:**

```bash
stripe listen --forward-to localhost:3000/api/stripe/webhook
```

### 2. テスト実行手順

1. ブラウザで http://localhost:5173 にアクセス
2. テストユーザーでログイン（または新規登録）
3. `/subscriptions` ページに移動
4. アップグレードしたいプランを選択
5. 「アップグレード」ボタンをクリック
6. Stripe Checkoutページでテストカード情報を入力

### 3. テストカード情報

| カード番号          | 説明                 | 結果             |
| ------------------- | -------------------- | ---------------- |
| 4242 4242 4242 4242 | 成功するカード       | 決済成功         |
| 4000 0000 0000 0002 | 決済が拒否される     | カード拒否エラー |
| 4000 0000 0000 9995 | 残高不足             | 残高不足エラー   |
| 4000 0025 0000 3155 | 3Dセキュア認証が必要 | 追加認証         |

**その他の入力情報:**

- 有効期限: 任意の将来の日付（例：12/34）
- CVC: 任意の3桁（例：123）
- 郵便番号: 任意の5桁（例：12345）

### 4. 成功の確認

決済完了後、以下を確認：

- `/subscriptions/success` ページが表示される
- `/subscriptions` ページに戻ると、プランが更新されている
- バックエンドログに以下のメッセージが表示される：
  - `Processing checkout.session.completed for subscription: ...`
  - `Updated subscription for user: ...`

## トラブルシューティング

### よくある問題と解決方法

#### 1. 401 Unauthorized エラー

**原因**: Webhook署名シークレットの不一致

**解決方法**:

1. Stripe CLIを再起動
2. 新しく表示される `whsec_...` をコピー
3. `.env` ファイルの `STRIPE_WEBHOOK_SECRET` を更新
4. バックエンドサーバーを再起動

#### 2. 500 Internal Server Error

**原因**: データベースの制約エラーまたはJSONパースエラー

**解決方法**:

- バックエンドログでエラーの詳細を確認
- データベースのマイグレーションが最新か確認
- Stripe APIバージョンの互換性を確認

#### 3. プランが更新されない

**原因**: Webhookイベントが正しく処理されていない

**確認事項**:

- Stripe CLIが実行中か
- バックエンドログにWebhookイベントが記録されているか
- データベースのPrice IDが正しく設定されているか

### デバッグ用コマンド

```bash
# Stripeイベントの確認
stripe events list --limit 5

# 特定のイベントの詳細
stripe events retrieve evt_xxxxx

# データベースの確認
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "
SELECT u.email, sp.name as plan_name, us.status, us.stripe_subscription_id
FROM user_subscriptions us
JOIN users u ON u.id = us.user_id
JOIN subscription_plans sp ON sp.id = us.plan_id;"
```

## AWS環境でのセットアップ（開発環境）

### 前提条件

- AWS CLIがインストール・設定済み
- AWS環境がCDKでデプロイ済み
- 踏み台ホスト（Bastion Host）が作成済み
- Route 53でドメインが設定済み（dev.markmail.engineers-hub.ltd）
- Stripe CLIがインストール済み

### 1. Stripe APIキーの設定

#### AWS Secrets Managerでの管理

Stripe APIキーはAWS Secrets Managerで安全に管理されます。

```bash
# Stripeシークレットの更新
aws secretsmanager update-secret \
  --secret-id markmail-dev-stripe-secret \
  --secret-string '{
    "STRIPE_SECRET_KEY": "sk_test_xxxxxxxxxxxxxxxx",
    "STRIPE_PUBLISHABLE_KEY": "pk_test_xxxxxxxxxxxxxxxx",
    "STRIPE_WEBHOOK_SECRET": "whsec_xxxxxxxxxxxxxxxx"
  }' \
  --profile your-profile
```

#### ECSサービスの再デプロイ

シークレット更新後、変更を反映させるためECSサービスを再デプロイします：

```bash
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --force-new-deployment \
  --profile your-profile
```

### 2. データベースの更新

#### 踏み台ホスト経由でのアクセス

AWS RDSへは踏み台ホスト経由でアクセスします。

##### 方法1: SSM Session Manager経由で直接接続

```bash
# 踏み台ホストのインスタンスIDを取得
INSTANCE_ID=$(aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  --query 'Reservations[*].Instances[*].[InstanceId]' \
  --output text \
  --profile your-profile)

# SSM Session Manager経由で接続
aws ssm start-session --target $INSTANCE_ID --profile your-profile

# 踏み台ホスト内でRDSに接続
PGPASSWORD=your-password psql \
  -h markmail-dev-db.xxxxx.region.rds.amazonaws.com \
  -U markmail \
  -d markmail
```

##### 方法2: SSM Send Command経由でコマンド実行（推奨）

```bash
# RDSエンドポイントを取得
RDS_ENDPOINT=$(aws rds describe-db-instances \
  --query 'DBInstances[?contains(DBInstanceIdentifier, `markmail-dev`)].Endpoint.Address' \
  --output text \
  --profile your-profile)

# データベースパスワードを取得
DB_PASSWORD=$(aws secretsmanager get-secret-value \
  --secret-id markmail-dev-db-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq -r '.password')

# ローカル環境からStripe Product/Price IDを確認
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "SELECT name, stripe_product_id, stripe_price_id FROM subscription_plans ORDER BY id;"
```

#### Stripe Product/Price IDの更新

踏み台ホストのインスタンスIDを確認後、以下のコマンドでRDSのデータを更新します：

```bash
# 踏み台ホストのインスタンスIDを確認
aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  "Name=instance-state-name,Values=running" \
  --query 'Reservations[*].Instances[*].[InstanceId,State.Name]' \
  --output table \
  --profile your-profile

# Stripe Product/Price IDを更新（実際のIDに置き換えてください）
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "export PGPASSWORD=\"your-database-password\"",
    "psql -h markmail-dev-db.cdkcmikuab4d.ap-northeast-1.rds.amazonaws.com -U markmail -d markmail -c \"UPDATE subscription_plans SET stripe_product_id = '"'"'prod_SWyCui2xJbDuJq'"'"', stripe_price_id = '"'"'price_1RbuGGGf4OK1sM7g1lyAE3yi'"'"' WHERE name = '"'"'pro'"'"';\"",
    "psql -h markmail-dev-db.cdkcmikuab4d.ap-northeast-1.rds.amazonaws.com -U markmail -d markmail -c \"UPDATE subscription_plans SET stripe_product_id = '"'"'prod_SX1g8ARlwQ5lDL'"'"', stripe_price_id = '"'"'price_1RbxckGf4OK1sM7gGWmThB6S'"'"' WHERE name = '"'"'business'"'"';\"",
    "psql -h markmail-dev-db.cdkcmikuab4d.ap-northeast-1.rds.amazonaws.com -U markmail -d markmail -c \"UPDATE subscription_plans SET stripe_product_id = '"'"'prod_SX1hkJ0kOqKujs'"'"', stripe_price_id = '"'"'price_1RbxeOGf4OK1sM7gitDS8jZ8'"'"' WHERE name = '"'"'free'"'"';\""
  ]' \
  --profile your-profile \
  --query 'Command.CommandId' \
  --output text

# コマンドの実行結果を確認
aws ssm get-command-invocation \
  --command-id <上記で出力されたCommandId> \
  --instance-id i-xxxxxxxxxxxxxxxxx \
  --profile your-profile | jq '{Status: .Status, StandardOutput: .StandardOutputContent}'

# 設定が正しく反映されたか確認
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "export PGPASSWORD=\"your-database-password\"",
    "psql -h markmail-dev-db.cdkcmikuab4d.ap-northeast-1.rds.amazonaws.com -U markmail -d markmail -c \"SELECT name, stripe_product_id, stripe_price_id FROM subscription_plans ORDER BY id;\""
  ]' \
  --profile your-profile \
  --query 'Command.CommandId' \
  --output text
```

### 3. Webhookエンドポイントの設定

#### Route 53ドメインを使用した設定（推奨）

AWS環境ではRoute
53で設定されたドメインが利用可能です。SSL付きのドメインを使用することで、より安全な通信が可能になります。

```bash
# 開発環境のドメイン
WEBHOOK_URL="https://dev.markmail.engineers-hub.ltd/api/stripe/webhook"
```

#### Stripe CLIを使用したWebhookエンドポイントの作成

```bash
# Stripe CLIを使用してWebhookエンドポイントを作成
stripe webhook_endpoints create \
  --url https://dev.markmail.engineers-hub.ltd/api/stripe/webhook \
  --enabled-events checkout.session.completed \
  --enabled-events customer.subscription.created \
  --enabled-events customer.subscription.updated \
  --enabled-events customer.subscription.deleted \
  --api-key sk_test_xxxxxxxxxxxxxxxx

# 出力例：
# {
#   "id": "we_1RcbtqGf4OK1sM7gdCklXdOw",
#   "object": "webhook_endpoint",
#   "enabled_events": [
#     "checkout.session.completed",
#     "customer.subscription.created",
#     "customer.subscription.updated",
#     "customer.subscription.deleted"
#   ],
#   "livemode": false,
#   "secret": "whsec_IpLMMBQw5wM4wMzo9ttBOdUl7XA6oDtY",
#   "status": "enabled",
#   "url": "https://dev.markmail.engineers-hub.ltd/api/stripe/webhook"
# }
```

#### 新しい署名シークレットでAWS Secrets Managerを更新

```bash
# Webhookエンドポイント作成時に生成された署名シークレットを使用
aws secretsmanager update-secret \
  --secret-id markmail-dev-stripe-secret \
  --secret-string '{
    "STRIPE_SECRET_KEY": "sk_test_xxxxxxxxxxxxxxxx",
    "STRIPE_PUBLISHABLE_KEY": "pk_test_xxxxxxxxxxxxxxxx",
    "STRIPE_WEBHOOK_SECRET": "whsec_新しく生成された署名シークレット"
  }' \
  --profile your-profile

# ECSサービスを再デプロイして新しいシークレットを反映
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --force-new-deployment \
  --profile your-profile
```

#### ALBのエンドポイントURL確認（ドメインを使用しない場合）

```bash
# ALBのDNS名を取得
ALB_DNS=$(aws elbv2 describe-load-balancers \
  --names markmail-dev-alb \
  --query 'LoadBalancers[0].DNSName' \
  --output text \
  --profile your-profile)

echo "Webhook URL (HTTP): http://$ALB_DNS/api/stripe/webhook"
```

**注意**:
HTTPを使用する場合、セキュリティリスクがあるため、本番環境では必ずHTTPS（ドメイン経由）を使用してください。

### 4. テストと検証

#### ログの確認

```bash
# バックエンドログの監視
aws logs tail /ecs/markmail-dev-backend --follow --profile your-profile

# Stripeイベントに絞って確認
aws logs filter-log-events \
  --log-group-name /ecs/markmail-dev-backend \
  --filter-pattern "stripe" \
  --start-time $(date -u -d '5 minutes ago' +%s)000 \
  --profile your-profile
```

#### エンドツーエンドテスト

1. アプリケーションにアクセス
   - SSL付きドメイン（推奨）: https://dev.markmail.engineers-hub.ltd
   - ALB DNS名:
     http://markmail-dev-alb-xxxxxxxxx.ap-northeast-1.elb.amazonaws.com
2. テストユーザーでログイン
3. サブスクリプションページ（`/subscriptions`）へ移動
4. アップグレードしたいプランを選択
5. 「アップグレード」ボタンをクリック
6. Stripe Checkoutページでテストカード情報を入力：
   - カード番号: `4242 4242 4242 4242`
   - 有効期限: 12/34
   - CVC: 123
   - 郵便番号: 12345
7. 決済完了後、以下を確認：
   - `/subscriptions/success` ページが表示される
   - `/subscriptions` ページに戻ると、プランが更新されている
   - バックエンドログでWebhookイベントの処理を確認

### 5. トラブルシューティング（AWS環境）

#### ECSタスクが起動しない

```bash
# タスクの状態を確認
aws ecs describe-tasks \
  --cluster markmail-dev \
  --tasks $(aws ecs list-tasks --cluster markmail-dev --service-name markmail-dev-backend --query 'taskArns[0]' --output text --profile your-profile) \
  --profile your-profile
```

#### Secrets Managerの値を確認

```bash
# 現在の値を確認（注意：APIキーが表示されます）
aws secretsmanager get-secret-value \
  --secret-id markmail-dev-stripe-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq '.'
```

#### RDS接続情報の取得

```bash
# RDSエンドポイント
aws rds describe-db-instances \
  --query 'DBInstances[?contains(DBInstanceIdentifier, `markmail-dev`)].Endpoint.Address' \
  --output text \
  --profile your-profile

# データベースパスワード
aws secretsmanager get-secret-value \
  --secret-id markmail-dev-db-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq -r '.password'
```

#### Webhookエンドポイントの管理

```bash
# 作成されたWebhookエンドポイントの一覧を確認
stripe webhook_endpoints list --api-key sk_test_xxxxxxxxxxxxxxxx

# 特定のWebhookエンドポイントの詳細を確認
stripe webhook_endpoints retrieve we_xxxxxxxxxxxxx --api-key sk_test_xxxxxxxxxxxxxxxx

# Webhookエンドポイントを削除（必要な場合）
stripe webhook_endpoints delete we_xxxxxxxxxxxxx --api-key sk_test_xxxxxxxxxxxxxxxx

# 最近のStripeイベントを確認
stripe events list --limit 5 --api-key sk_test_xxxxxxxxxxxxxxxx

# 特定のイベントの詳細を確認
stripe events retrieve evt_xxxxxxxxxxxxx --api-key sk_test_xxxxxxxxxxxxxxxx
```

#### データベースでサブスクリプション状態を確認

```bash
# 踏み台ホスト経由でRDSのサブスクリプションデータを確認
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "export PGPASSWORD=\"your-database-password\"",
    "psql -h markmail-dev-db.cdkcmikuab4d.ap-northeast-1.rds.amazonaws.com -U markmail -d markmail -c \"SELECT u.email, sp.name as plan_name, us.status, us.stripe_subscription_id FROM user_subscriptions us JOIN users u ON u.id = us.user_id JOIN subscription_plans sp ON sp.id = us.plan_id;\""
  ]' \
  --profile your-profile \
  --query 'Command.CommandId' \
  --output text
```

## 本番環境への移行

### 1. 環境変数の更新

```bash
# 本番用の.env（ローカル開発用）
STRIPE_SECRET_KEY=sk_live_xxxxxxxxxxxxxxxx  # 本番APIキー
STRIPE_PUBLISHABLE_KEY=pk_live_xxxxxxxxxxxxxxxx  # 本番公開可能キー
STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxxxxxx  # 本番Webhookシークレット
```

### 2. Webhookエンドポイントの設定

1. Stripeダッシュボードで本番環境に切り替え
2. 開発者 > Webhook > エンドポイントを追加
3. エンドポイントURL: `https://yourdomain.com/api/stripe/webhook`
4. イベントタイプを選択：
   - `checkout.session.completed`
   - `customer.subscription.created`
   - `customer.subscription.updated`
   - `customer.subscription.deleted`
5. 署名シークレットを `.env` に設定

### 3. 本番用商品の作成

テスト環境と同様に、本番環境でも商品を作成し、データベースを更新します。

## セキュリティに関する注意事項

1. **APIキーの管理**

   - APIキーは絶対にコミットしない
   - 環境変数または安全な秘密管理システムを使用

2. **Webhook署名の検証**

   - 必ず署名を検証してリクエストの正当性を確認
   - リプレイ攻撃を防ぐためタイムスタンプもチェック

3. **HTTPS必須**
   - 本番環境では必ずHTTPSを使用
   - Webhookエンドポイントも同様

## 参考リンク

- [Stripe公式ドキュメント](https://docs.stripe.com)
- [Stripe API リファレンス](https://docs.stripe.com/api)
- [Stripe CLI ドキュメント](https://docs.stripe.com/stripe-cli)
- [テスト用カード番号](https://docs.stripe.com/testing#cards)
