# CLAUDE.md

このファイルはClaude Code
(claude.ai/code)がこのリポジトリで作業する際のガイダンスを提供します。

## ⚡ 最重要事項 - 絶対に行ってはいけないこと

### 1. 既存のマイグレーションファイルの削除・変更

- データベースマイグレーションファイル（`backend/migrations/*.sql`）は絶対に削除・変更しない
- 新しいマイグレーションが必要な場合は、新しいタイムスタンプで追加ファイルを作成する
- 既に適用されたマイグレーションは変更不可能

### 2. テストの無効化

- テストが失敗する場合は、テストを削除・無効化せず、コードを修正する
- `#[ignore]`や`skip`の使用は禁止
- **テストを通すためにロジックを変更する愚行は絶対に禁止**
- テストは既存のロジックを検証するものであり、テストに合わせてロジックを変更してはならない
- **既存の正常に動いているテストを消すな！**

### 3. 直接的なデータベース操作

- `DROP TABLE`、`DROP DATABASE`などの破壊的操作は絶対に実行しない
- データベーススキーマの変更は必ずマイグレーションファイル経由で行う

### 4. 環境変数・シークレットの露出

- `.env`ファイルの内容をコミット・表示しない
- APIキーやパスワードをハードコーディングしない

## 🛠️ 必須開発コマンド

### バックエンド (Rust)

```bash
# 開発
cd backend
cargo run                          # 開発サーバー起動 (ポート3000)
cargo test                         # 全テスト実行
cargo test test_name               # 特定のテスト実行
cargo clippy -- -D warnings        # リンター実行
cargo fmt                          # コードフォーマット

# データベース
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/markmail"
sqlx migrate run                   # マイグレーション実行
sqlx migrate add migration_name    # 新規マイグレーション作成
cargo sqlx prepare                 # オフラインコンパイル用のsqlx-data.json更新
```

### フロントエンド (SvelteKit)

```bash
# 開発
cd frontend
npm run dev                        # 開発サーバー起動 (ポート5173)
npm run build                      # 本番ビルド
npm test                          # 全テスト実行
npm test -- --run                  # テストを一度だけ実行
npm run check                      # 型チェック
npm run lint                       # ESLint実行
npm run format                     # コードフォーマット
```

### インフラストラクチャ (AWS CDK)

```bash
cd infrastructure
npm test                           # インフラテスト実行
npm run build                      # TypeScriptコンパイル
npm run deploy                     # AWSへデプロイ
cdk synth                         # CloudFormationテンプレート生成
```

### プロジェクト全体のコマンド

```bash
# プロジェクトルートから
docker-compose up -d               # 全サービス起動 (PostgreSQL, Redis, MailHog)
npm run format                     # コードベース全体をフォーマット
npm run lint                      # コードベース全体をリント
./scripts/setup-lefthook.sh       # Gitフックのセットアップ
```

## 🏗️ アーキテクチャ概要

### システム全体の構成

アプリケーションは関心の分離を明確にした設計：

- **フロントエンド**: SvelteKit SPAでクライアントサイドルーティング（SSR無効）
- **バックエンド**: Rust/Axum REST APIでJWT認証
- **データベース**: PostgreSQLとSQLxでコンパイル時クエリ検証
- **インフラ**: AWS CDKでInfrastructure as Code

### バックエンドアーキテクチャ (Rust)

```
backend/src/
├── api/           # HTTPエンドポイントハンドラー（ルート定義）
├── database/      # データベースクエリ関数（リポジトリ層）
├── models/        # ドメインモデルとリクエスト/レスポンス型
├── services/      # ビジネスロジック層
├── middleware/    # 認証、CORS、ロギングミドルウェア
└── utils/         # 共有ユーティリティ（JWT、パスワードハッシュ、バリデーション）
```

**主要パターン**:

- 全APIルートはAxumの`from_fn`ミドルウェアで認証
- データベースクエリはSQLxでコンパイル時検証
- サービス層がビジネスロジックを処理、ハンドラーは薄く保つ
- モデルはデータベースエンティティとAPIコントラクトの両方を定義
- エラーハンドリングはカスタムエラー型で適切なHTTPステータスコード

### フロントエンドアーキテクチャ (SvelteKit)

```
frontend/src/
├── routes/        # SvelteKitページとAPIルート
├── lib/
│   ├── services/  # APIクライアントサービス
│   ├── stores/    # Svelteストア（認証、グローバル状態）
│   └── types/     # TypeScript型定義
└── tests/         # srcの構造をミラーリングしたテストファイル
```

**主要パターン**:

- `+layout.js`で`ssr = false`と`prerender = false`によるSPAモード
- 全APIコールはサービス層経由で適切なエラーハンドリング
- 認証状態は`authStore`でlocalStorageに永続化
- フォームコンポーネントは作成と編集で共通ロジック
- TypeScript型はバックエンドAPIコントラクトと一致

### データベーススキーマ

主要テーブルと関係:

- `users` → `templates`, `campaigns`, `subscribers`, `forms`, `sequences`
- `campaigns` → `templates` (多対一)
- `forms` → `form_fields` (一対多)
- `sequences` → `sequence_steps` (一対多)
- `sequence_steps` → `templates` (多対一)
- `form_submissions` → `forms` (多対一)

## 📋 重要な開発上の注意事項

### データベースマイグレーション

- **既存のマイグレーションファイルは絶対に変更しない** - 一度適用されたら不変
- 新規マイグレーションは常にタイムスタンプ付き: `sqlx migrate add description`
- マイグレーション後は`cargo sqlx prepare`でオフラインコンパイルデータを更新

### テスト哲学

- **失敗するテストを無効化しない** - 根本原因を修正する
- テスト命名: `test_feature_scenario` (例:
  `test_create_campaign_with_invalid_template`)
- バックエンドテストは自動クリーンアップ付きの分離されたテストデータベース使用
- フロントエンドテストはAPIへの依存を避けるためモックサービス使用

### 認証フロー

1. ログインでJWT（24時間）+リフレッシュトークン（30日）を返す
2. フロントエンドはauthStore経由でlocalStorageにトークン保存
3. APIリクエストは`Authorization: Bearer <token>`ヘッダーを含む
4. 401レスポンスで自動ログアウト
5. 保護されたルートはレンダリング前に認証状態をチェック

### フォームビルダーシステム

フォームは複雑なフィールド構造を持つ:

- バックエンドは`form_fields`（スネークケース）を使用
- フロントエンドコンポーネントは`form.form_fields`を使用（`form.fields`ではない）
- フィールドタイプ: text, email, textarea, select, radio, checkbox等
- 公開フォームは認証なしで`/forms/[id]/public`でアクセス可能

### メールサービスアーキテクチャ

- プロバイダー抽象化traitでMailHog（開発）とAWS SES（本番）を切り替え
- 環境変数`EMAIL_PROVIDER`でプロバイダーを制御
- 本番用のレート制限付きバッチ送信
- テンプレート変数は`{{variable_name}}`構文を使用

### よくある落とし穴

1. **SvelteKitの動的ルート**: プリレンダリングできない、SPAモードを使用
2. **開発時のCORS**: バックエンドはlocalhost:5173を許可、本番は同一ドメイン
3. **SQLxオフラインモード**: スキーマ変更後は`cargo sqlx prepare`を実行
4. **Lefthookフォーマット**: コミット時に自動実行、`--no-verify`でバイパスしない

## 🚀 AWSデプロイメントノート

### ビルド設定

- フロントエンドはSPA用の`fallback: "index.html"`でstatic adapterを使用
- Dockerfileは`.svelte-kit/output`ではなく`/app/build`からコピー
- VITE_API_URL環境変数はAPIエンドポイント用にビルド時に設定

### インフラストラクチャスタック

- コンテナ化されたサービス用のECS Fargate
- RDS Aurora PostgreSQL Serverless v2
- パスベースルーティング付きApplication Load Balancer
- ロギングとモニタリング用のCloudWatch
- GitHubからのCI/CD用CodePipeline

### 環境変数

設定必須の重要な変数:

- `DATABASE_URL`: PostgreSQL接続文字列
- `JWT_SECRET`: JWT署名用シークレット
- `VITE_API_URL`: フロントエンドAPIエンドポイント（ビルド時）
- `EMAIL_PROVIDER`: mailhogまたはaws_ses
- `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`: SES用

## 🚫 よくある愚行と防止策

### 1. テストを通すためにロジックを変更する

- ❌ 悪い例: テストが失敗したので、テストに合わせてビジネスロジックを変更
- ✅ 良い例: ロジックが正しい場合はテストを修正、バグがある場合はロジックを修正

### 2. マイグレーションファイルの削除・変更

- ❌ 悪い例: エラーが出たので既存のマイグレーションファイルを削除
- ✅ 良い例: 新しいタイムスタンプで修正用のマイグレーションを追加

### 3. エラーを握りつぶす

- ❌ 悪い例: `unwrap()`でエラーが出たので`.unwrap_or_default()`に変更
- ✅ 良い例: エラーの原因を調査し、適切なエラーハンドリングを実装

### 4. 作業ディレクトリの混乱

- ❌ 悪い例: 現在のディレクトリを確認せずにコマンド実行
- ✅ 良い例: `pwd`で常に現在位置を確認、適切なディレクトリに移動してから作業

## 📝 コミットメッセージ規約

```
<type>: <subject>

<body>
```

タイプ:

- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメントのみの変更
- `style`: コードの意味に影響しない変更
- `refactor`: リファクタリング
- `test`: テストの追加・修正
- `chore`: ビルドプロセスやツールの変更

## 🔧 トラブルシューティング

### データベース接続エラー

```bash
# PostgreSQLが起動しているか確認
docker-compose ps

# 起動していない場合
docker-compose up -d postgres

# マイグレーションの実行
cd backend
sqlx migrate run
```

### ビルドエラー

```bash
# Rust
cargo clean
cargo build

# Frontend
cd frontend
rm -rf node_modules .svelte-kit
npm install
```

### 不要ファイルのクリーンアップ

```bash
# 未追跡ファイルの確認
git clean -n

# 未追跡ファイルとディレクトリの削除
git clean -fd
```

このファイルの指示に従い、安全で高品質なコード開発を行ってください。
