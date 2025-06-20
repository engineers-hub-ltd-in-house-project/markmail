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

## 本番環境への移行

### 1. 環境変数の更新

```bash
# 本番用の.env
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
