# MarkMail

エンジニア向けマークダウンベースメールマーケティングツール

## 🎯 プロジェクト概要

**MarkMail**は、エンジニアが慣れ親しんだマークダウン記法を使ってメールテンプレートを作成し、効率的なメールマーケティングを実現するツールです。

### 技術スタック

- **バックエンド**: Rust (Axum + SQLx + Tokio)
- **フロントエンド**: Svelte + SvelteKit + TypeScript
- **データベース**: PostgreSQL
- **キャッシュ**: Redis
- **メール送信**: AWS SES / SendGrid
- **認証**: JWT + OAuth2
- **自動整形**: lefthook + Prettier + rustfmt
- **デプロイ**: Docker + Railway/Fly.io

## 🏗️ システムアーキテクチャ

以下は MarkMail のシステム全体のアーキテクチャ図です：

<div align="center">
  <img src="docs/architecture-simple.svg" alt="MarkMail システムアーキテクチャ" width="100%">
</div>

<details>
<summary>詳細なアーキテクチャ図</summary>

![詳細アーキテクチャ](docs/architecture.svg)

</details>

<details>
<summary>テキストベースのアーキテクチャ図</summary>

```
┌─────────────────────────────────────────────────────────────────────┐
│                    MarkMail システムアーキテクチャ                    │
└─────────────────────────────────────────────────────────────────────┘

┌───────────────┐    API Calls    ┌───────────────┐    API Calls    ┌───────────────┐
│   Frontend    │ ──────────────► │   Backend     │ ──────────────► │ External      │
│  (SvelteKit)  │                 │   (Rust)      │                 │ Services      │
│               │                 │               │                 │               │
│ • UI          │                 │ • Axum API    │                 │ • AWS SES     │
│ • MD Editor   │                 │ • JWT Auth    │                 │ • SendGrid    │
│ • Preview     │                 │ • Email Mgr   │                 │ • GitHub API  │
│ • TypeScript  │                 │ • Template    │                 │ • S3 Storage  │
│ • Tailwind    │                 │ • Campaign    │                 │               │
│               │                 │ • User Mgr    │                 │               │
│ Port: 5173    │                 │ Port: 3000    │                 │               │
└───────────────┘                 └───────┬───────┘                 └───────────────┘
                                          │
                                      SQL/Cache
                                          │
                                          ▼
                    ┌─────────────────────────────────────┐
                    │            Data Layer               │
                    │                                     │
                    │  ┌─────────────┐  ┌─────────────┐  │
                    │  │ PostgreSQL  │  │    Redis    │  │
                    │  │    (DB)     │  │  (Cache)    │  │
                    │  └─────────────┘  └─────────────┘  │
                    └─────────────────────────────────────┘

┌───────────────┐
│ Development   │
│ Tools         │
│               │
│ • Docker      │
│ • MailHog     │
│ • Railway     │
│ • GitHub      │
│   Actions     │
└───────────────┘
```

</details>

### アーキテクチャの特徴

- **マイクロサービス指向**: フロントエンド、バックエンド、データ層が分離された設計
- **高パフォーマンス**: Rust と SvelteKit による高速なレスポンス
- **スケーラブル**: Redis キャッシュと PostgreSQL による高いスケーラビリティ
- **セキュア**: JWT 認証と多層防御によるセキュリティ
- **開発体験**: Docker と自動整形による快適な開発環境

## 🚀 クイックスタート

### 前提条件

- Docker & Docker Compose
- Rust (1.75+)
- Node.js (18+)

### ローカル開発環境のセットアップ

1. **プロジェクトのクローン**

```bash
git clone https://github.com/your-org/markmail.git
cd markmail
```

2. **環境変数の設定**

```bash
cp env.example .env
# .env ファイルを編集して必要な値を設定
```

3. **自動整形のセットアップ（重要！）**

```bash
./scripts/setup-lefthook.sh
```

これで `git commit` 時に自動整形が実行されるようになります。

4. **Docker Compose で開発環境起動**

```bash
docker-compose up -d
```

5. **データベースマイグレーション**

```bash
cd backend
cargo install sqlx-cli

# DATABASE_URL環境変数を設定（重要！）
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/markmail"

# マイグレーション実行
sqlx migrate run
```

**注意**:
SQLx のコンパイル時には`DATABASE_URL`環境変数が必要です。VSCode を使用している場合は、ターミナルを再起動するか、`.env`ファイルから環境変数を読み込んでください。

### アクセス先

- **フロントエンド**: http://localhost:5173
- **バックエンド API**: http://localhost:3000
- **MailHog (メール確認)**: http://localhost:8025

## ✨ 自動整形機能

### 🪝 Git Hooks による自動整形

**lefthook**を使用して、コミット時に自動的にコードを整形します：

- **git commit 時**:
  - Rust コード → `cargo fmt` で整形
  - フロントエンドコード → `prettier` で整形
  - リンターチェック → `cargo clippy` & `eslint`
- **git push 時**:
  - テスト自動実行

### 🎨 手動整形コマンド

```bash
# 全体のフォーマット
npm run format

# バックエンドのみ（Rust）
npm run format:backend

# フロントエンドのみ（Svelte/TypeScript）
npm run format:frontend

# リンター
npm run lint
```

### 🔧 VS Code 自動整形

VS Code を使用している場合、以下が自動で設定されます：

- **保存時自動整形**: ファイル保存時に自動フォーマット
- **ペースト時自動整形**: コードペースト時に自動フォーマット
- **推奨拡張機能**: Rust Analyzer、Svelte、Prettier 等

## 📁 プロジェクト構造

```
markmail/
├── backend/                 # Rust バックエンド
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/             # API ルート定義
│   │   ├── services/        # ビジネスロジック
│   │   ├── models/          # データモデル
│   │   ├── database/        # DB関連
│   │   ├── middleware/      # ミドルウェア
│   │   └── utils/           # ユーティリティ
│   ├── Cargo.toml
│   ├── rustfmt.toml         # Rust フォーマット設定
│   └── Dockerfile.dev
├── frontend/                # Svelte フロントエンド
│   ├── src/
│   │   ├── routes/          # SvelteKit ルート
│   │   └── lib/             # 共有コンポーネント
│   ├── package.json
│   ├── .eslintrc.cjs        # ESLint 設定
│   └── Dockerfile.dev
├── .vscode/                 # VS Code 設定
│   ├── settings.json        # エディタ設定
│   └── extensions.json      # 推奨拡張機能
├── lefthook.yml             # Git hooks 設定
├── .prettierrc              # Prettier 設定
├── docker-compose.yml       # ローカル開発環境
└── README.md
```

## 🔧 開発コマンド

### バックエンド (Rust)

```bash
cd backend

# 開発サーバー起動
cargo run

# テスト実行
cargo test

# リンター実行
cargo clippy

# フォーマット
cargo fmt
```

### フロントエンド (Svelte)

```bash
cd frontend

# 依存関係インストール
npm install

# 開発サーバー起動
npm run dev

# ビルド
npm run build

# 型チェック
npm run check

# テスト実行
npm run test

# フォーマット
npm run format

# リンター
npm run lint
```

## 🌟 主な機能

### ✅ 実装済み

- [x] プロジェクト構造の作成
- [x] Rust バックエンドの基本セットアップ
- [x] Svelte フロントエンドの基本セットアップ
- [x] Docker 開発環境
- [x] API エンドポイントの定義
- [x] データモデルの定義
- [x] **自動整形システム (lefthook)**
- [x] **VS Code 開発環境設定**
- [x] **認証システム (JWT) - 完全動作確認済み**
  - [x] ユーザー登録・ログイン API
  - [x] JWT トークン発行・検証（24 時間有効）
  - [x] リフレッシュトークン（30 日間有効）
  - [x] 認証ミドルウェア（Axum from_fn）
  - [x] パスワードハッシュ化（bcrypt）
  - [x] データベーステーブル（users, refresh_tokens）
- [x] **プロフィール管理 API（取得・更新）- 動作確認済み**
- [x] **テンプレート管理機能（バックエンド）- 完全動作確認済み**
  - [x] データベーステーブル設計・作成（templates）
  - [x] CRUD API 実装（作成・取得・更新・削除・一覧）
  - [x] マークダウンから HTML への変換機能
  - [x] テンプレート変数システム（{{variable_name}}形式）
  - [x] プレビュー機能（変数置換 + HTML 変換）
  - [x] メール用 CSS スタイリング
  - [x] マークダウン構文検証機能

### 🚧 開発中

- [ ] **テンプレート管理機能（フロントエンド）**
  - [ ] テンプレート一覧画面
  - [ ] テンプレート作成・編集画面
  - [ ] マークダウンエディター（リアルタイムプレビュー付き）
  - [ ] テンプレート変数管理 UI

### 📋 今後の予定

- [ ] メール送信機能（AWS SES/SendGrid 統合）
- [ ] キャンペーン管理システム
- [ ] 購読者管理・インポート機能
- [ ] GitHub 連携（README 直接インポート）
- [ ] 分析・レポート機能
- [ ] A/B テスト機能
- [ ] API レート制限
- [ ] メールプレビュー機能
- [ ] スケジュール送信
- [ ] Webhook 統合

## 🧪 テスト

### バックエンドテスト

```bash
cd backend
cargo test
```

### フロントエンドテスト

```bash
cd frontend
npm run test
```

### API テスト例（curl）

```bash
# ユーザー登録
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123",
    "name": "テストユーザー"
  }'

# ログイン
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'

# トークン更新（リフレッシュトークンを使用）
curl -X POST http://localhost:3000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "YOUR_REFRESH_TOKEN_HERE"
  }'

# テンプレート作成（認証必要）
curl -X POST http://localhost:3000/api/templates \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "ウェルカムメール",
    "subject_template": "{{company_name}}へようこそ、{{user_name}}さん！",
    "markdown_content": "# ようこそ {{user_name}} さん！\n\n{{company_name}}へのご登録ありがとうございます。"
  }'

# テンプレートプレビュー（認証必要）
curl -X POST http://localhost:3000/api/templates/TEMPLATE_ID/preview \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "variables": {
      "user_name": "田中太郎",
      "company_name": "株式会社MarkMail"
    }
  }'

# マークダウンレンダリング（認証必要）
curl -X POST http://localhost:3000/api/markdown/render \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "markdown": "# テスト {{name}} さん",
    "variables": {"name": "太郎"}
  }'
```

## 📚 API ドキュメント

### 認証 ✅（動作確認済み）

- `POST /api/auth/register` - ユーザー登録
  - リクエスト:
    `{"email": "user@example.com", "password": "password123", "name": "ユーザー名"}`
  - レスポンス: JWT トークン、リフレッシュトークン、ユーザー情報
- `POST /api/auth/login` - ログイン
  - リクエスト: `{"email": "user@example.com", "password": "password123"}`
  - レスポンス: JWT トークン、リフレッシュトークン、ユーザー情報
- `POST /api/auth/refresh` - トークン更新
  - リクエスト: `{"refresh_token": "..."}`
  - レスポンス: 新しい JWT トークン、新しいリフレッシュトークン

### プロフィール ✅（動作確認済み）

- `GET /api/users/profile` - プロフィール取得
  - レスポンス: ユーザー情報（ID、メール、名前、アバター等）
- `PUT /api/users/profile` - プロフィール更新
  - リクエスト: `{"name": "新しい名前", "avatar_url": "https://..."}`
  - レスポンス: 更新されたユーザー情報

### テンプレート ✅（動作確認済み）

- `GET /api/templates` - テンプレート一覧取得
  - パラメータ: `?limit=50&offset=0`
  - レスポンス: テンプレート一覧、総数、ページング情報
- `POST /api/templates` - テンプレート作成
  - リクエスト:
    `{"name": "テンプレート名", "subject_template": "件名テンプレート", "markdown_content": "# マークダウン", "variables": {"key": "value"}, "is_public": false}`
  - レスポンス: 作成されたテンプレート情報
- `GET /api/templates/:id` - テンプレート取得
  - レスポンス: テンプレート詳細情報
- `PUT /api/templates/:id` - テンプレート更新
  - リクエスト: 更新したいフィールドのみ
  - レスポンス: 更新されたテンプレート情報
- `DELETE /api/templates/:id` - テンプレート削除
  - レスポンス: 削除確認メッセージ
- `POST /api/templates/:id/preview` - テンプレートプレビュー
  - リクエスト:
    `{"variables": {"user_name": "田中太郎", "company_name": "株式会社例"}}`
  - レスポンス: `{"html": "変換されたHTML", "subject": "変数置換済み件名"}`

### マークダウン処理 ✅（動作確認済み）

- `POST /api/markdown/render` - マークダウンを HTML に変換
  - リクエスト:
    `{"markdown": "# マークダウンテキスト", "variables": {"key": "value"}}`
  - レスポンス:
    `{"html": "変換されたHTML", "extracted_variables": ["変数一覧"]}`
- `POST /api/markdown/validate` - マークダウン構文検証
  - リクエスト: `{"markdown": "# マークダウンテキスト"}`
  - レスポンス:
    `{"valid": true, "errors": [], "extracted_variables": ["変数一覧"]}`

### キャンペーン（未実装）

- `GET /api/campaigns` - キャンペーン一覧
- `POST /api/campaigns` - キャンペーン作成
- `GET /api/campaigns/:id` - キャンペーン詳細
- `POST /api/campaigns/:id/send` - キャンペーン送信
- `POST /api/campaigns/:id/schedule` - キャンペーンスケジュール

### 購読者（未実装）

- `GET /api/subscribers` - 購読者一覧
- `POST /api/subscribers` - 購読者追加
- `POST /api/subscribers/import` - CSV 一括インポート

## 🎨 コーディング規約

### 自動整形設定

プロジェクトでは一貫したコードスタイルを保つため、以下のツールを使用しています：

- **Rust**: `rustfmt` + `clippy`
- **TypeScript/JavaScript**: `prettier` + `eslint`
- **Svelte**: `prettier` + `eslint-plugin-svelte`
- **JSON/YAML/Markdown**: `prettier`

### フォーマット設定

- **インデント**: スペース 2 文字（Rust は 4 文字）
- **行幅**: 100 文字
- **改行**: LF
- **セミコロン**: あり
- **クォート**: シングルクォート

## 🤝 コントリビューション

1. このリポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
   - **自動整形が実行されます！**
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

### 開発時の注意点

- **自動整形**: `git commit` 時に自動でコードが整形されます
- **リンターエラー**: コミット前にリンターエラーを修正してください
- **テスト**: `git push` 時にテストが自動実行されます

## 📄 ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。

## 🙋‍♂️ サポート

質問や問題がある場合は、[Issues](https://github.com/your-org/markmail/issues)
を作成してください。

---

**MarkMail** - エンジニアのためのメールマーケティングツール 🚀

### 🔥 特徴

- **自動整形**: コミット時に自動でコードが美しく整形
- **高速開発**: lefthook による高速な Git hooks
- **VS Code 最適化**: 保存時自動整形で快適な開発体験
- **一貫性**: チーム全体で統一されたコードスタイル

## 📖 実装詳細

### 認証システム

MarkMail の認証システムは以下の技術を使用しています：

- **JWT (JSON Web Token)**: アクセストークンの生成・検証
- **bcrypt**: パスワードのハッシュ化
- **リフレッシュトークン**: 長期認証のための 64 文字のランダムトークン
- **Axum ミドルウェア**: `from_fn`を使用した認証ミドルウェア

#### 認証フロー

1. **ユーザー登録**: メールアドレス、パスワード、名前を受け取り、パスワードを bcrypt でハッシュ化してデータベースに保存
2. **ログイン**: メールアドレスとパスワードを検証し、JWT アクセストークン（24 時間有効）とリフレッシュトークン（30 日間有効）を発行
3. **API 保護**: 認証が必要なエンドポイントは`Authorization: Bearer <token>`ヘッダーで JWT トークンを検証
4. **トークン更新**: リフレッシュトークンを使用して新しいアクセストークンを取得
