# 開発環境セットアップ

このドキュメントでは、Salesforce統合の開発環境をセットアップする手順を説明します。

## 前提条件

- Docker Composeがインストールされていること
- Salesforce Developer Accountまたは組織へのアクセス
- Node.js 18以上
- Rust 1.70以上

## ローカル環境構築

### 1. リポジトリのクローン

```bash
git clone https://github.com/engineers-hub-ltd-in-house-project/markmail.git
cd markmail
```

### 2. 環境変数の設定

```bash
# .envファイルを作成
cp .env.example .env
```

`.env`ファイルを編集して以下を設定:

```bash
# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/markmail

# JWT
JWT_SECRET=your-secure-jwt-secret-key

# Email (開発環境)
EMAIL_PROVIDER=mailhog

# Salesforce OAuth2
SALESFORCE_CLIENT_ID=your-consumer-key
SALESFORCE_CLIENT_SECRET=your-consumer-secret
SALESFORCE_REDIRECT_URL=http://localhost:3000/api/crm/oauth/salesforce/callback
SALESFORCE_USE_SANDBOX=true
```

### 3. Dockerサービスの起動

```bash
# PostgreSQL、Redis、MailHogを起動
docker-compose up -d
```

### 4. データベースセットアップ

```bash
cd backend

# データベース作成
docker exec markmail-postgres-1 psql -U postgres -c "CREATE DATABASE markmail;"

# マイグレーション実行
sqlx migrate run

# SQLxオフラインキャッシュ生成
cargo sqlx prepare
```

### 5. 開発サーバーの起動

バックエンド:

```bash
cd backend
cargo watch -c -w src -w .env -x run
# または
./watch.sh
```

フロントエンド:

```bash
cd frontend
npm install
npm run dev
```

## Salesforce組織の設定

### 1. Developer Accountの作成

1. [Salesforce Developer](https://developer.salesforce.com/signup)でアカウント作成
2. メールで認証を完了
3. ログインURLとユーザー名を保存

### 2. Connected Appの作成

1. **Setup → Apps → App Manager**
2. **New Connected App**をクリック
3. 以下を入力:

```yaml
Connected App Name: MarkMail Development
API Name: MarkMail_Development
Contact Email: your-email@example.com

Enable OAuth Settings: ✓

Callback URL:
  - http://localhost:3000/api/crm/oauth/salesforce/callback
  - https://dev.markmail.engineers-hub.ltd/api/crm/oauth/salesforce/callback

Selected OAuth Scopes:
  - Access and manage your data (api)
  - Perform requests on your behalf at any time (refresh_token, offline_access)
  - Access your basic information (id, profile, email, address, phone)
  - Access custom permissions (custom_permissions)

Require Secret for Web Server Flow: ✓
```

4. **Save**をクリック
5. **Manage Consumer Details**から:
   - Consumer Key (Client ID)
   - Consumer Secret (Client Secret) をコピーして`.env`に設定

### 3. カスタムフィールドの作成

1. **Setup → Object Manager → Lead**
2. **Fields & Relationships → New**
3. 以下のフィールドを作成:

| Field Label              | API Name                      | Data Type | Length | Required |
| ------------------------ | ----------------------------- | --------- | ------ | -------- |
| Engineer Skills          | Engineer_Skills\_\_c          | Text Area | 1000   | No       |
| Years of Experience      | Years_of_Experience\_\_c      | Number    | 18,0   | No       |
| Project Budget           | Project_Budget\_\_c           | Currency  | 16,2   | No       |
| Preferred Contact Method | Preferred_Contact_Method\_\_c | Picklist  | -      | No       |

ピックリスト値（Preferred Contact Method）:

- Email
- Phone
- Both

### 4. フィールドレベルセキュリティ

各カスタムフィールドに対して:

1. **Field Accessibility**をクリック
2. 統合ユーザープロファイルを選択
3. **Visible**と**Read-Only**をチェック解除（編集可能にする）
4. **Save**

## テスト用フォームの作成

### 1. MarkMailにログイン

```bash
# テスト用ユーザーでログイン
# デフォルト: admin@example.com / password123
```

### 2. エンジニアスキルフォームの作成

```python
# スクリプトで自動作成
cd scripts/salesforce-integration/form-management
python create_markmail_form_dev.py
```

または手動で:

1. Forms → Create New Form
2. 以下のフィールドを追加:
   - Name (Text, Required)
   - Email (Email, Required)
   - Company (Text, Required)
   - Phone (Text)
   - Engineer Skills (Textarea)
   - Years of Experience (Number)
   - Message (Textarea)

### 3. CRM統合の有効化

1. フォーム編集画面
2. **Settings**タブ
3. **CRM Integration**をON
4. **Save**

## 認証フローのテスト

### 1. OAuth2認証の開始

```bash
# 認証URLを生成してブラウザで開く
python scripts/salesforce-integration/testing/oauth2_flow.py
```

### 2. Salesforceログイン

1. Salesforceのログイン画面が表示される
2. 開発者アカウントでログイン
3. アプリケーションの許可
4. `http://localhost:3000/api/crm/oauth/success`にリダイレクト

### 3. トークンの確認

```bash
# データベースでトークンを確認
docker exec -it markmail-postgres-1 psql -U postgres -d markmail -c "SELECT * FROM crm_integrations;"
```

## フォーム送信テスト

### 1. 公開フォームURLの取得

```bash
# フォームを公開
python scripts/salesforce-integration/form-management/publish_form_dev.py

# URLが表示される
# 例: http://localhost:5173/forms/72406032-3f07-4da2-9621-4018373b857e/public
```

### 2. テスト送信

```bash
# 自動テストスクリプト
python scripts/salesforce-integration/testing/submit_form_test_dev.py
```

### 3. Salesforceで確認

1. Salesforceにログイン
2. **Leads**タブ
3. 新しいリードが作成されていることを確認

## Docker環境での開発

### Dockerfile更新時のトークン設定

```python
# Docker環境でのトークン更新
python scripts/salesforce-integration/utilities/update_sf_token_docker.py
```

## トラブルシューティング

### PostgreSQL接続エラー

```bash
# コンテナ状態確認
docker-compose ps

# ログ確認
docker logs markmail-postgres-1

# 再起動
docker-compose restart postgres
```

### SQLxコンパイルエラー

```bash
# オフラインキャッシュを再生成
cd backend
cargo sqlx prepare
```

### Salesforce認証エラー

1. Consumer Key/Secretが正しいか確認
2. Callback URLが完全一致しているか確認
3. Sandboxモードの設定を確認

### ポート競合

```bash
# 使用中のポートを確認
lsof -i :3000  # Backend
lsof -i :5173  # Frontend
lsof -i :5432  # PostgreSQL
```

## 開発ツール

### VS Code拡張機能

- rust-analyzer: Rust開発サポート
- Svelte for VS Code: Svelte開発サポート
- SQLTools: データベース接続
- Thunder Client: API テスト

### 便利なコマンド

```bash
# Rustフォーマット
cd backend && cargo fmt

# フロントエンドリント
cd frontend && npm run lint

# 全体フォーマット（Lefthook）
npm run format

# テスト実行
cd backend && cargo test -- --test-threads=1
cd frontend && npm test
```

## 次のステップ

1. [OAuth2認証の詳細](./01-authentication.md)
2. [リード管理の実装](./02-lead-management.md)
3. [テストツールの使用方法](./04-testing-tools.md)
