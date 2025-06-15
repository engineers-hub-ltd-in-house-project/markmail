# MarkMail

マークダウンベースのマーケティングオートメーションツール

👉 **開発ロードマップ**: [ROADMAP.md](./ROADMAP.md)  
👉 **開発規約**: [DEVELOPMENT.md](./DEVELOPMENT.md)

## 🤖 AI開発者向けガイドライン

このプロジェクトで作業する際は、以下の原則に従ってください：

1. **既存コードの尊重**: 既に実装済みの機能やパターンを優先的に使用
2. **テストファースト**: 新機能は必ずテストを先に作成
3. **型安全性**: TypeScriptの型定義を必ず追加
4. **エラーハンドリング**: 適切なエラーメッセージとステータスコードを返す
5. **セキュリティ**: 認証が必要なエンドポイントには必ず認証ミドルウェアを適用

## 🎯 プロジェクト概要

MarkMailは、AI駆動のマーケティングオートメーションツールです。プロンプト一つで完全なマーケティングファネルを自動構築し、Markdown形式でAIが理解しやすいメールを作成します。エンジニアファーストな設計で、API経由での無限の拡張性を提供します。

### 🚀 AI機能ハイライト

- **AIマーケティングシナリオ自動生成**: 業界・目的を指定するだけで、シーケンス・テンプレート・フォームを自動構築
- **AIコンテンツアシスタント**: メール文面の自動生成・最適化、件名のA/Bテスト提案
- **インテリジェントセグメンテーション**: 購読者の行動を機械学習で分析し、最適なタイミングで配信
- **スマートオートメーション**: AIがシーケンスの最適化を継続的に提案
- **多言語対応（開発中）**: 日本語・英語でのAI生成コンテンツに対応

#### 🤖 AI機能のセットアップ

AI機能を使用するには、`.env`ファイルに以下の設定を追加してください：

```env
# AI Provider Configuration
AI_PROVIDER=openai  # または 'anthropic'

# OpenAI Configuration
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxx
OPENAI_MODEL=gpt-4  # オプション（デフォルト: gpt-4）

# Anthropic Configuration
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxxxxx
ANTHROPIC_MODEL=claude-3-opus-20240229  # オプション
```

**APIキーの取得方法：**

- OpenAI: https://platform.openai.com/api-keys
- Anthropic: https://console.anthropic.com/account/keys

### 技術スタック

- **バックエンド**: Rust (Axum + SQLx + Tokio)
- **フロントエンド**: Svelte + SvelteKit + TypeScript
- **データベース**: PostgreSQL
- **キャッシュ**: Redis
- **メール送信**: AWS SES / MailHog（開発環境）
- **認証**: JWT + リフレッシュトークン
- **インフラストラクチャ**: AWS CDK v2 (TypeScript)
- **コンテナ**: Docker (ECS Fargate)
- **CI/CD**: GitHub Actions + AWS CodePipeline
- **自動整形**: lefthook + Prettier + rustfmt
- **テスト**: Vitest + Jest + Cargo test

## 🏗️ アーキテクチャ概要

```
Frontend (SvelteKit) → Backend (Rust/Axum) → Database (PostgreSQL)
                                           → Cache (Redis)
                                           → Email (AWS SES/MailHog)
```

詳細は [docs/architecture.svg](docs/architecture.svg) を参照。

## 🚀 クイックスタート

```bash
# 1. セットアップ
git clone https://github.com/engineers-hub-ltd-in-house-project/markmail.git
cd markmail
cp env.example .env
./scripts/setup-lefthook.sh

# 2. 起動
docker-compose up -d

# 3. マイグレーション
cd backend
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/markmail"
sqlx migrate run
```

**アクセスURL**:

- フロントエンド: http://localhost:5173
- API: http://localhost:3000
- メール確認: http://localhost:8025

## 🛠️ 開発コマンド

```bash
# フォーマット
npm run format           # 全体
npm run format:backend   # Rust
npm run format:frontend  # Svelte/TS

# テスト
cd backend && cargo test
cd frontend && npm test

# リンター
npm run lint
```

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

# 開発サーバー起動（自動リロード付き）⭐ NEW
cargo watch -c -w src -w .env -x run
# または
./watch.sh

# テスト実行
cargo test

# リンター実行
cargo clippy

# フォーマット
cargo fmt
```

#### 🔄 cargo-watch による自動リロード開発

`cargo-watch`をインストールすることで、ファイル変更時の自動再起動が可能になります：

```bash
# インストール
cargo install cargo-watch

# 使用方法
cargo watch -c -w src -w .env -x run
```

オプション説明：

- `-c` : 実行前に画面をクリア
- `-w src` : srcディレクトリを監視
- `-w .env` : .envファイルも監視
- `-x run` : cargo runを実行

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

### 💎 料金プラン

- **Free** (¥0/月): 個人・スタートアップ向け
  - コンタクト数: 100名まで
  - 月間送信数: 1,000通まで
  - 基本機能のみ
- **Pro** (¥4,980/月): 成長企業向け
  - コンタクト数: 10,000名まで
  - 月間送信数: 100,000通まで
  - 全機能利用可能
- **Business** (¥19,800/月): エンタープライズ向け
  - コンタクト数: 100,000名まで
  - 月間送信数: 1,000,000通まで
  - 優先サポート付き

### ✅ 実装済み

- [x] プロジェクト構造の作成
- [x] Rust バックエンドの基本セットアップ
- [x] Svelte フロントエンドの基本セットアップ
- [x] Docker 開発環境
- [x] API エンドポイントの定義
- [x] データモデルの定義
- [x] **自動整形システム (lefthook)**
- [x] **VS Code 開発環境設定**
- [x] **認証システム (JWT) - API 実装・テスト済み**
  - [x] ユーザー登録・ログイン API
  - [x] JWT トークン発行・検証（24 時間有効）
  - [x] リフレッシュトークン（30 日間有効）
  - [x] 認証ミドルウェア（Axum from_fn）
  - [x] パスワードハッシュ化（bcrypt）
  - [x] データベーステーブル（users, refresh_tokens）
- [x] **プロフィール管理 API（取得・更新）- API 実装済み**
- [x] **テンプレート管理機能（バックエンド）- API 実装・テスト済み**
  - [x] データベーステーブル設計・作成（templates）
  - [x] CRUD API 実装（作成・取得・更新・削除・一覧）
  - [x] マークダウンから HTML への変換機能
  - [x] テンプレート変数システム（{{variable_name}}形式）
  - [x] プレビュー API（変数置換 + HTML 変換）
  - [x] メール用 CSS スタイリング
  - [x] マークダウン構文検証機能
- [x] **テンプレート管理機能（フロントエンド）- 基本機能実装済み**
  - [x] TypeScript 型定義（Template、CreateTemplateRequest 等）
  - [x] テンプレート一覧画面（/templates）
  - [x] テンプレート作成画面（/templates/new）
  - [x] マークダウンエディター（リアルタイムプレビュー付き）
  - [x] テンプレート変数管理 UI
  - [x] Svelte TypeScript プリプロセッサ設定修正
- [x] **キャンペーン管理機能（バックエンド）- API 実装・テスト済み**

  - [x] データベーステーブル設計・作成（campaigns）
  - [x] CRUD API 実装（作成・取得・更新・削除・一覧）
  - [x] キャンペーンスケジュール機能
  - [x] キャンペーン送信状態管理
  - [x] キャンペーンプレビュー機能
  - [x] 統計情報モデル（送信数・開封率・クリック率）

- [x] **購読者管理機能（バックエンド）- API 実装・テスト済み**
  - [x] データベーステーブル設計・作成（subscribers）
  - [x] CRUD API 実装（作成・取得・更新・削除・一覧）
  - [x] CSV一括インポート機能
  - [x] タグ管理機能
  - [x] ステータス管理（active、unsubscribed、bounced、complained）
  - [x] カスタムフィールド対応
- [x] **メール送信機能（バックエンド）- 実装済み**
  - [x] MailHog 対応（開発環境）
  - [x] AWS SES 対応（本番環境）
  - [x] 環境変数による送信方法の切り替え
  - [x] バッチ送信とレート制限
  - [x] プロバイダー抽象化（trait EmailProvider）
  - [x] テンプレート変数の動的置換
  - [x] 送信エラーハンドリング
  - [x] ドメイン検証とDKIM設定

### 🚧 開発中

- [x] **テンプレート管理機能（フロントエンド）- 追加機能**
  - [x] テンプレート編集画面
  - [x] テンプレート詳細表示画面
  - [x] 認証画面の実装（ログイン・ユーザー登録）
  - [ ] エラーハンドリングの改善
- [x] **キャンペーン管理機能（フロントエンド）**
  - [x] キャンペーン一覧画面
  - [x] キャンペーン作成・編集画面
  - [x] キャンペーンスケジュール設定UI
  - [x] キャンペーン送信・プレビュー機能
- [x] **購読者管理・インポート機能（フロントエンド）**
  - [x] 購読者一覧画面
  - [x] 購読者詳細・編集画面
  - [x] CSV一括インポート機能
  - [x] タグ管理機能
- [x] **メール送信機能（フロントエンド統合）**
  - [x] キャンペーン送信ボタンとAPI連携
  - [x] 送信進捗表示
  - [x] エラーハンドリング
  - [x] 送信成功/失敗の状態表示
- [x] **フォーム機能（バックエンド・フロントエンド）- 完全実装済み**
  - [x] フォーム作成・編集・削除API
  - [x] フォームフィールド動的生成（テキスト、メール、テキストエリア、選択等）
  - [x] フォーム公開機能（認証不要の送信エンドポイント）
  - [x] フォーム送信データ管理・一覧取得
  - [x] フロントエンド統合（フォーム管理画面）
  - [x] バリデーション機能
  - [x] フォーム設定（送信後メッセージ等）
  - [x] フォーム送信から購読者自動作成
- [x] **シーケンス機能（バックエンド・フロントエンド）- 完全実装済み**
  - [x] シーケンス作成・編集・削除API
  - [x] シーケンスステップ管理（メール送信、待機、条件分岐対応）
  - [x] トリガー設定（手動、登録、フォーム送信等）
  - [x] シーケンス実行状態管理（enrollments）
  - [x] テンプレート連携（ステップごとのテンプレート指定）
  - [x] 詳細設定（遅延時間、条件、アクション設定）
  - [x] フロントエンド統合（シーケンス管理画面）
  - [x] シーケンス自動化システム（バックグラウンドワーカー）
  - [x] トリガーベースの自動エンロールメント
  - [x] ステップ実行エンジン（メール送信、待機、条件分岐、タグ付け）
- [x] **AWS インフラストラクチャ（CDK v2）**
  - [x] ネットワーク層（VPC、サブネット、セキュリティグループ）
  - [x] コンテナ基盤（ECS Cluster、ECR、Fargate）
  - [x] データベース層（RDS Aurora PostgreSQL Serverless v2）
  - [x] アプリケーションロードバランサー（ALB）
  - [x] 監視とロギング（CloudWatch、Container Insights）
  - [x] CI/CDパイプライン（CodePipeline、CodeBuild）
  - [x] AWS SES設定（ドメイン検証、DKIM、送信権限）
  - [x] シークレット管理（Secrets Manager）
  - [x] インフラストラクチャテスト（Jest）

### 🎉 最近の更新

- [x] **AI機能の実装（2025-06-15）** 🤖 NEW
  - [x] マルチプロバイダー対応（OpenAI GPT-4、Anthropic Claude）
  - [x] AIマーケティングシナリオ自動生成API
  - [x] AIコンテンツ生成・最適化API
  - [x] 件名最適化エンジン（予測開封率付き）
  - [x] フロントエンドUI統合（シナリオ生成、コンテンツ生成、件名最適化）
  - [x] プロバイダー抽象化によるフォールバック機構
  - [x] トークンカウント機能とレート制限対応
  - [ ] **出力言語選択機能（開発中）** - 日本語/英語での生成が可能
- [x] **開発環境の改善（2025-06-15）** 🛠️ NEW
  - [x] cargo-watchによる自動リロード開発環境
  - [x] watch.shスクリプトの追加
- [x] **シーケンス自動化システムの完全実装（2025-06-12）**
  - [x] バックグラウンドワーカーによる自動実行
  - [x] フォーム送信から購読者作成・シーケンス登録の完全自動化
  - [x] トリガーベースの動的エンロールメント
  - [x] AWS SES経由でのメール配信確認
  - [x] エラー耐性の高い非同期処理アーキテクチャ
- [x] **フォーム・シーケンス機能の完全実装**
  - [x] フォーム機能: API実装からフロントエンド統合まで完了
  - [x] シーケンス機能: 高度なメール自動化機能の完全実装
  - [x] 包括的テストスイート: axumパターンでの統合テスト実装
  - [x] データベーススキーマ:
        forms、sequences、sequence_steps、form_submissions、sequence_enrollments、sequence_step_logs テーブル追加
- [x] **テストスイートの大幅改善**
  - [x] バックエンドテスト: 49/49 成功 (5つはignored)
  - [x] インフラテスト: 76/76 成功
  - [x] フロントエンドテスト: 31/31 成功 (10つはskipped)
  - [x] axumパターンでのテスト統一（forms、templates、sequences）
  - [x] テスト用データベース分離とクリーンアップ機能
  - [x] CLAUDE.md追加（AI開発者向けガイドライン）

### 📋 今後の予定

- [ ] **サブスクリプション機能の実装** 🔴 最優先
  - [ ] データベース設計・基本機能
  - [ ] 制限チェックシステム
  - [ ] Stripe決済統合
  - [ ] プラン選択・アップグレード画面
  - [ ] 使用量トラッキング
- [ ] **E2Eテスト基盤の確立**
  - [ ] Playwrightの導入と設定
  - [ ] CI/CDパイプラインへの統合
  - [ ] クリティカルユーザージャーニーのテスト作成
  - [ ] テストデータ管理システム
  - [ ] 並列実行環境の構築
- [x] **AI機能の実装** ✅ COMPLETED
  - [x] AIマーケティングシナリオ自動生成
    - [x] プロンプトインターフェース
    - [x] 業界別テンプレート
    - [x] シーケンス・フォーム・テンプレート自動生成
  - [x] AIコンテンツアシスタント
    - [x] メール文面自動生成
    - [x] 件名最適化
    - [x] パーソナライゼーション提案
  - [ ] インテリジェントセグメンテーション（Phase 2）
    - [ ] 購読者行動分析
    - [ ] エンゲージメントスコアリング
    - [ ] チャーン予測
- [ ] **テストカバレッジの改善**
  - [ ] 既存テストのギャップ分析
  - [ ] 不足している統合テストの追加
  - [ ] APIテストの網羅性向上
  - [ ] エラーハンドリングテスト
- [ ] **フォーム埋め込み・公開機能**
  - [ ] フォーム埋め込みスクリプト生成
  - [ ] iframeエンベッド機能
  - [ ] JavaScriptウィジェット提供
  - [ ] カスタムCSS対応
- [ ] **ダブルオプトイン機能**
  - [ ] 確認メール自動送信
  - [ ] 確認リンク生成・管理
  - [ ] 確認状態の追跡
  - [ ] 再送信機能
- [ ] **本番環境デプロイ**
  - [ ] AWS環境へのCDKデプロイ
  - [ ] ドメイン設定とSSL証明書
  - [ ] 本番用環境変数の設定
  - [ ] SESサンドボックスモードからの移行申請
- [ ] **メール送信機能（追加機能）**
  - [ ] 送信ログの保存と表示
  - [ ] 配信停止処理
  - [ ] バウンス・苦情処理
  - [ ] 送信結果の詳細レポート
- [ ] **外部システム連携機能**
  - [ ] Salesforce インテグレーション
  - [ ] HubSpot 連携
  - [ ] Zapier Webhook 対応
  - [ ] REST API for 外部システム
- [ ] 分析・レポート機能
- [ ] A/B テスト機能（AI駆動）
- [ ] API レート制限

## 🧪 テスト

### テスト戦略

MarkMailでは包括的なテスト戦略を採用し、高品質なソフトウェアを提供します：

#### 1. **E2Eテスト（Playwright）** 🆕

- **目的**: 実際のユーザー体験を網羅的に検証
- **カバレッジ**: クリティカルユーザージャーニーの100%
- **主要なテストシナリオ**:
  - ユーザー登録・ログインフロー
  - テンプレート作成・編集・削除
  - キャンペーン作成・送信
  - 購読者管理・インポート
  - フォーム作成・公開・送信
  - シーケンス設定・実行
- **実行タイミング**: PR時（スモーク）、デプロイ前（フル）、夜間（回帰）

#### 2. **統合テスト**

- **バックエンド**: APIエンドポイントの結合テスト（目標: 90%以上）
- **フロントエンド**: コンポーネント間の連携テスト
- **データベース**: マイグレーション整合性テスト

#### 3. **ユニットテスト**

- **バックエンド**: ビジネスロジック単体テスト（目標: 80%以上）
- **フロントエンド**: コンポーネント単体テスト（目標: 70%以上）
- **共通ユーティリティ**: 100%カバレッジ

### テスト原則

**重要**: テストはプロジェクトの品質を保証する重要な要素です。以下の原則を必ず守ってください：

1. テストが失敗する場合は、テスト条件を緩和するのではなく、コードを修正してテストを通過させること
2. 新機能の実装時は必ず対応するテストを追加すること
3. バグ修正時は、そのバグを検出するテストを追加すること
4. テストカバレッジは定期的に確認し、向上させること

詳細は [DEVELOPMENT.md](./DEVELOPMENT.md) を参照してください。

### テスト実行結果

現在のテストカバレッジ:

- **バックエンド**:
  49個のテストが成功（認証、テンプレート、キャンペーン、購読者、フォーム、シーケンス、メール送信）
- **インフラストラクチャ**: 76個のテストが成功（CDKスタック、コンストラクト）
- **フロントエンド**:
  31個のテストが成功（API、ストア、ユーティリティ、コンポーネント）

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

### 認証 ✅（API 実装・テスト済み）

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

### プロフィール ✅（API 実装済み）

- `GET /api/users/profile` - プロフィール取得
  - レスポンス: ユーザー情報（ID、メール、名前、アバター等）
- `PUT /api/users/profile` - プロフィール更新
  - リクエスト: `{"name": "新しい名前", "avatar_url": "https://..."}`
  - レスポンス: 更新されたユーザー情報

### テンプレート ✅（API 実装・テスト済み）

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

### マークダウン処理 ✅（API 実装・テスト済み）

- `POST /api/markdown/render` - マークダウンを HTML に変換
  - リクエスト:
    `{"markdown": "# マークダウンテキスト", "variables": {"key": "value"}}`
  - レスポンス:
    `{"html": "変換されたHTML", "extracted_variables": ["変数一覧"]}`
- `POST /api/markdown/validate` - マークダウン構文検証
  - リクエスト: `{"markdown": "# マークダウンテキスト"}`
  - レスポンス:
    `{"valid": true, "errors": [], "extracted_variables": ["変数一覧"]}`

### キャンペーン ✅（API 実装・テスト済み）

- `GET /api/campaigns` - キャンペーン一覧
  - パラメータ:
    `?limit=50&offset=0&status=draft&sort_by=created_at&sort_order=DESC`
  - レスポンス: キャンペーン一覧、総数、ページング情報
- `POST /api/campaigns` - キャンペーン作成
  - リクエスト:
    `{"name": "キャンペーン名", "description": "説明", "subject": "メール件名", "template_id": "テンプレートID"}`
  - レスポンス: 作成されたキャンペーン情報
- `GET /api/campaigns/:id` - キャンペーン詳細取得
  - レスポンス: キャンペーン詳細情報と統計データ
- `PUT /api/campaigns/:id` - キャンペーン更新
  - リクエスト: 更新したいフィールドのみ
  - レスポンス: 更新されたキャンペーン情報
- `DELETE /api/campaigns/:id` - キャンペーン削除
  - レスポンス: 削除確認メッセージ
- `POST /api/campaigns/:id/send` - キャンペーン送信
  - レスポンス: 送信開始確認メッセージ
- `POST /api/campaigns/:id/schedule` - キャンペーンスケジュール
  - リクエスト: `{"scheduled_at": "2025-07-01T12:00:00Z"}`
  - レスポンス: 更新されたキャンペーン情報
- `GET /api/campaigns/:id/preview` - キャンペーンプレビュー
  - レスポンス: `{"html": "変換されたHTML"}`

### フォーム ✅（API実装・フロントエンド統合完了）

- `GET /api/forms` - フォーム一覧取得
  - レスポンス: フォーム一覧（ユーザー所有のもののみ）
- `POST /api/forms` - フォーム作成
  - リクエスト:
    `{"name": "お問い合わせフォーム", "description": "説明", "slug": "contact", "markdown_content": "# フォーム", "form_fields": [{"field_type": "text", "name": "name", "label": "お名前", "required": true}], "settings": {}}`
  - レスポンス: 作成されたフォーム情報
- `GET /api/forms/:id` - フォーム詳細取得
  - レスポンス: フォーム詳細情報（所有者のみアクセス可能）
- `PUT /api/forms/:id` - フォーム更新
  - リクエスト: 更新したいフィールドのみ
  - レスポンス: 更新されたフォーム情報
- `DELETE /api/forms/:id` - フォーム削除
  - レスポンス: 削除確認メッセージ
- `GET /api/forms/:id/submissions` - フォーム送信データ取得
  - パラメータ: `?limit=20&offset=0`
  - レスポンス: `{"submissions": [...], "total": 100, "limit": 20, "offset": 0}`
- `GET /api/forms/:id/public` - 公開フォーム取得（認証不要）
  - レスポンス: 公開されたフォーム情報
- `POST /api/forms/:id/submit` - フォーム送信（認証不要）
  - リクエスト: `{"data": {"name": "田中太郎", "email": "tanaka@example.com"}}`
  - レスポンス: 送信完了確認

### シーケンス ✅（API実装・フロントエンド統合完了）

- `GET /api/sequences` - シーケンス一覧取得
  - レスポンス: シーケンス一覧（ユーザー所有のもののみ）
- `POST /api/sequences` - シーケンス作成
  - リクエスト:
    `{"name": "ウェルカムシーケンス", "description": "新規登録者向け", "trigger_type": "registration", "trigger_config": {}}`
  - レスポンス: 作成されたシーケンス情報
- `GET /api/sequences/:id` - シーケンス詳細取得
  - レスポンス: シーケンス詳細情報（所有者のみアクセス可能）
- `PUT /api/sequences/:id` - シーケンス更新
  - リクエスト: 更新したいフィールドのみ
  - レスポンス: 更新されたシーケンス情報
- `DELETE /api/sequences/:id` - シーケンス削除
  - レスポンス: 削除確認メッセージ
- `GET /api/sequences/:id/full` - ステップ付きシーケンス取得
  - レスポンス: `{"sequence": {...}, "steps": [{...}]}`
- `POST /api/sequences/:id/steps` - シーケンスステップ作成
  - リクエスト:
    `{"name": "ウェルカムメール", "step_order": 1, "step_type": "send_email", "delay_value": 0, "delay_unit": "hours", "template_id": "...", "subject": "ようこそ！"}`
  - レスポンス: 作成されたステップ情報
- `PUT /api/sequences/:sequence_id/steps/:step_id` - シーケンスステップ更新
  - リクエスト: 更新したいフィールドのみ
  - レスポンス: 更新されたステップ情報
- `DELETE /api/sequences/:sequence_id/steps/:step_id` - シーケンスステップ削除
  - レスポンス: 削除確認メッセージ

### 購読者 ✅（フロントエンド実装済み）

- `GET /api/subscribers` - 購読者一覧
  - パラメータ:
    `?limit=50&offset=0&search=query&tag=tag&status=active&sort_by=created_at&sort_order=DESC`
  - レスポンス: 購読者一覧、総数、ページング情報
- `POST /api/subscribers` - 購読者追加
  - リクエスト:
    `{"email": "user@example.com", "name": "名前", "tags": ["タグ1", "タグ2"], "custom_fields": {"key": "value"}}`
  - レスポンス: 作成された購読者情報
- `GET /api/subscribers/:id` - 購読者詳細取得
  - レスポンス: 購読者詳細情報
- `PUT /api/subscribers/:id` - 購読者更新
  - リクエスト: 更新したいフィールドのみ
  - レスポンス: 更新された購読者情報
- `DELETE /api/subscribers/:id` - 購読者削除
  - レスポンス: 削除確認メッセージ
- `POST /api/subscribers/import` - CSV 一括インポート
  - リクエスト: マルチパートフォームデータ (file: CSVファイル, tag: 共通タグ)
  - レスポンス:
    `{"message": "インポート完了", "imported": 10, "failed": 0, "errors": []}`
- `GET /api/subscribers/tags` - 利用可能なタグ一覧
  - レスポンス: `{"tags": ["タグ1", "タグ2", "タグ3"]}`

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
- **テスト失敗時**: テスト条件を緩和せずに根本的な問題を修正してください

### CI/CD パイプライン

プロジェクトでは以下のCI/CDパイプラインが設定されています：

1. **コミット前チェック（pre-commit）**:
   - コードフォーマット
   - リンターチェック
2. **プッシュ前チェック（pre-push）**:
   - バックエンドテスト実行
   - フロントエンドテスト実行
3. **継続的インテグレーション（GitHub Actions）**:
   - ビルド検証
   - 全テスト実行
   - コードカバレッジレポート生成

**重要**: テストが失敗した場合は、テストを無効化したり条件を緩和するのではなく、根本的な問題を修正してください。

## 📄 ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。

## 🙋‍♂️ サポート

質問や問題がある場合は、[Issues](https://github.com/engineers-hub-ltd-in-house-project/markmail/issues)
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

### メール送信システム（開発中）

MarkMail のメール送信システムは、開発環境と本番環境で異なるプロバイダーをサポートします：

#### サポートするメールプロバイダー

1. **MailHog（開発環境）**

   - ローカル開発用のSMTPサーバー
   - Docker Composeで自動起動
   - Web UIでメール確認可能（http://localhost:8025）
   - 実際のメール送信なし

2. **AWS SES（本番環境）** ✅
   - 高い配信性能と信頼性
   - 詳細な配信統計
   - バウンス・苦情処理の自動化
   - リージョン: 東京（ap-northeast-1）
   - Configuration Set による送信イベント追跡
   - ドメイン検証とDKIM署名によるメール認証
   - SPFレコードによるなりすまし防止

#### メール送信アーキテクチャ

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Campaign      │     │   Email         │     │  Mail Provider  │
│   Service       │────▶│   Service       │────▶│                 │
│                 │     │                 │     │  - MailHog      │
│ • キャンペーン   │     │ • テンプレート   │     │  - AWS SES      │
│   送信管理      │     │   レンダリング   │     │                 │
│ • バッチ処理    │     │ • 変数置換      │     └─────────────────┘
│ • レート制限    │     │ • 送信処理      │
└─────────────────┘     └─────────────────┘
        │                       │
        │                       │
        ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│   Send Log      │     │   Subscriber    │
│   Database      │     │   Database      │
│                 │     │                 │
│ • 送信履歴      │     │ • 配信状況      │
│ • エラーログ    │     │ • 配信停止      │
└─────────────────┘     └─────────────────┘
```

#### 環境変数設定

```env
# メール送信設定
EMAIL_PROVIDER=mailhog  # mailhog | aws_ses
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_FROM=noreply@markmail.dev
SMTP_FROM_NAME=MarkMail
SMTP_USERNAME=
SMTP_PASSWORD=

# AWS SES設定（本番環境）
AWS_REGION=ap-northeast-1
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
AWS_SES_FROM_EMAIL=noreply@example.com
AWS_SES_CONFIGURATION_SET=markmail-configuration-set

# メール送信制限
EMAIL_RATE_LIMIT=14  # 秒あたりの送信数（AWS SESのデフォルト）
EMAIL_BATCH_SIZE=50  # バッチ送信のサイズ
```

#### メール送信フロー

1. **キャンペーン送信開始**: APIエンドポイント `/api/campaigns/:id/send`
   を呼び出し
2. **購読者リスト取得**: 対象となる購読者をデータベースから取得
3. **バッチ処理**: 大量送信時は `EMAIL_BATCH_SIZE` ごとに分割
4. **テンプレートレンダリング**: 各購読者用に変数を置換してHTML生成
5. **メール送信**: 設定されたプロバイダー経由で送信
6. **送信ログ記録**: 成功/失敗をデータベースに記録
7. **統計更新**: キャンペーンの送信数、エラー数を更新

#### エラーハンドリング

- **一時的エラー**: 自動リトライ（最大3回）
- **恒久的エラー**: エラーログに記録して次の購読者へ
- **レート制限**: `EMAIL_RATE_LIMIT` に基づいて送信速度を調整
- **バウンス処理**: AWS SES のSNS通知を受信して自動処理

### AWS インフラストラクチャ設定

MarkMailはAWS CDKを使用してSES関連のインフラストラクチャを管理しています：

#### インフラストラクチャ構成

- **AWS SES Configuration Set**: メール送信イベントの追跡
- **SNSトピック**: バウンス・苦情通知の受信
- **S3バケット**: バウンスメールの保存（90日後自動削除）
- **IAMユーザー**: SES送信専用のアクセス権限

#### セットアップ手順

1. **AWS CDKのインストール**

   ```bash
   npm install -g aws-cdk
   ```

2. **インフラストラクチャのデプロイ**

   ```bash
   cd infrastructure
   npm install
   npm run deploy
   ```

3. **ドメイン検証（必須）**

   - AWS SESコンソールでドメインまたはメールアドレスを検証
   - DNSレコード（DKIM、SPF）を設定

4. **環境変数の更新**
   - デプロイ後に表示されるアクセスキーを`.env`に設定

#### サンドボックスモードについて

新しいAWS SESアカウントはサンドボックスモードで開始されます：

- 検証済みのメールアドレスにのみ送信可能
- 1日あたり200通、1秒あたり1通の制限
- 本番環境への移行はAWSサポートへ申請が必要
