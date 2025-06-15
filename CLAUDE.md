# CLAUDE.md

このファイルはClaude Code
(claude.ai/code)がこのリポジトリで作業する際のガイダンスを提供します。This file
provides guidance for Claude Code when working with this repository.

## 🔴 エラー発生時は必ずこのセクションを確認すること / ALWAYS CHECK THIS SECTION WHEN ERRORS OCCUR

### テストが失敗した場合の対処法 / How to Handle Test Failures

1. **絶対にやってはいけないこと / NEVER DO THESE**

   - ❌ `git push --no-verify` でテストをスキップ / Skip tests with
     `git push --no-verify`
   - ❌ `#[ignore]` でテストを無効化 / Disable tests with `#[ignore]`
   - ❌ テストに合わせてビジネスロジックを変更 / Change business logic to pass
     tests

2. **必ず行うこと / ALWAYS DO THESE**
   - ✅ エラーメッセージを読んで原因を特定 / Read error messages and identify
     the cause
   - ✅ テストDBの問題なら / If it's a test DB issue: `DROP DATABASE` →
     `CREATE DATABASE` → `sqlx migrate run`
   - ✅ コードの問題なら: バグを修正 / If it's a code issue: Fix the bug
   - ✅ 全てのテストが通ることを確認してからプッシュ / Ensure all tests pass
     before pushing

### プッシュ時にエラーが発生した場合 / When Push Errors Occur

1. **pre-pushフックでテストが失敗 / pre-push hook test failures**

   - ❌ 絶対に `--no-verify` を使わない / NEVER use `--no-verify`
   - ✅ 上記の「テストが失敗した場合の対処法」を実行 / Follow the test failure
     handling above

2. **権限エラー / Permission errors**
   - ✅ `gh auth login` で認証を更新 / Update authentication with
     `gh auth login`

## ⚡ 最重要事項 - 絶対に行ってはいけないこと / CRITICAL - NEVER DO THESE

### 1. 既存のマイグレーションファイルの削除・変更 / Never Delete or Modify Existing Migration Files

- データベースマイグレーションファイル（`backend/migrations/*.sql`）は絶対に削除・変更しない /
  NEVER delete or modify database migration files
- 新しいマイグレーションが必要な場合は、新しいタイムスタンプで追加ファイルを作成する /
  Create new migration files with new timestamps
- 既に適用されたマイグレーションは変更不可能 / Applied migrations are immutable

### 2. テストの無効化 / Never Disable Tests

- テストが失敗する場合は、テストを削除・無効化せず、コードを修正する / Fix code
  instead of disabling tests
- `#[ignore]`や`skip`の使用は禁止 / Using `#[ignore]` or `skip` is forbidden
- **テストを通すためにロジックを変更する愚行は絶対に禁止** / **NEVER change
  business logic to make tests pass**
- テストは既存のロジックを検証するものであり、テストに合わせてロジックを変更してはならない /
  Tests verify existing logic, don't change logic to fit tests
- **既存の正常に動いているテストを消すな！** / **NEVER delete working tests!**

### 3. 直接的なデータベース操作 / Never Manipulate Database Directly

- `DROP TABLE`、`DROP DATABASE`などの破壊的操作は絶対に実行しない（テストDB除く） /
  NEVER execute destructive operations (except test DB)
- データベーススキーマの変更は必ずマイグレーションファイル経由で行う / Always
  use migration files for schema changes

### 4. 環境変数・シークレットの露出 / Never Expose Secrets

- `.env`ファイルの内容をコミット・表示しない / Never commit or display `.env`
  contents
- APIキーやパスワードをハードコーディングしない / Never hardcode API keys or
  passwords

### 5. 過信を招く表現の使用禁止 / Never Use Overconfident Language

- **「完璧」という言葉を絶対に使わない** - 実装後に必ず問題が発生する / **NEVER
  use the word "perfect"** - issues always arise after implementation
- 「問題ありません」「大丈夫です」などの断定的な表現を避ける / Avoid definitive
  expressions like "no problem"
- 常に「～と思われます」「～はずです」のような慎重な表現を使う / Always use
  cautious expressions
- 実装完了後も潜在的な問題の可能性を常に意識する / Always be aware of potential
  issues

## 🛠️ 必須開発コマンド

### バックエンド (Rust)

```bash
# 開発
cd backend
cargo run                          # 開発サーバー起動 (ポート3000)
cargo watch -c -w src -w .env -x run  # 自動リロード開発サーバー起動 ⭐ 推奨
./watch.sh                         # 上記と同じ（スクリプト版）
cargo test                         # 全テスト実行
cargo test test_name               # 特定のテスト実行
cargo clippy -- -D warnings        # リンター実行
cargo fmt                          # コードフォーマット

# cargo-watchのインストール（初回のみ）
cargo install cargo-watch

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

### AI機能の設定

```bash
# .envファイルに以下を追加
AI_PROVIDER=openai                 # または 'anthropic'
OPENAI_API_KEY=sk-xxxx            # OpenAI APIキー
ANTHROPIC_API_KEY=sk-ant-xxxx     # Anthropic APIキー

# AI機能へのアクセス
# 1. ナビゲーションメニューの「AI機能」をクリック
# 2. 以下の3つの機能が利用可能：
#    - マーケティングシナリオ生成
#    - コンテンツ生成
#    - 件名最適化
```

## 🏗️ アーキテクチャ概要

### システム全体の構成

アプリケーションは関心の分離を明確にした設計：

- **フロントエンド**: SvelteKit SPAでクライアントサイドルーティング（SSR無効）
- **バックエンド**: Rust/Axum REST APIでJWT認証
- **データベース**: PostgreSQLとSQLxでコンパイル時クエリ検証
- **インフラ**: AWS CDKでInfrastructure as Code
- **バックグラウンド処理**: Tokioによる非同期ワーカー

### バックエンドアーキテクチャ (Rust)

```
backend/src/
├── api/           # HTTPエンドポイントハンドラー（ルート定義）
├── database/      # データベースクエリ関数（リポジトリ層）
├── models/        # ドメインモデルとリクエスト/レスポンス型
├── services/      # ビジネスロジック層
├── workers/       # バックグラウンドワーカー
├── middleware/    # 認証、CORS、ロギングミドルウェア
├── utils/         # 共有ユーティリティ（JWT、パスワードハッシュ、バリデーション）
└── ai/            # AI機能モジュール ⭐ NEW
    ├── providers/ # プロバイダー実装（OpenAI、Anthropic）
    ├── services/  # AIサービス（シナリオビルダー、コンテンツ生成）
    └── models/    # AI関連のデータモデルとプロンプト
```

**主要パターン**:

- 全APIルートはAxumの`from_fn`ミドルウェアで認証
- データベースクエリはSQLxでコンパイル時検証
- サービス層がビジネスロジックを処理、ハンドラーは薄く保つ
- モデルはデータベースエンティティとAPIコントラクトの両方を定義
- エラーハンドリングはカスタムエラー型で適切なHTTPステータスコード
- バックグラウンドワーカーは独立したTokioタスクで実行

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
- `sequence_enrollments` → `sequences`, `subscribers` (多対一)
- `sequence_step_logs` → `sequence_enrollments`, `sequence_steps` (多対一)

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

### シーケンス自動化システム

- バックグラウンドワーカーが60秒間隔で実行待ちステップを処理
- トリガーベースの自動エンロールメント（フォーム送信、購読者作成等）
- ステップタイプ:
  email（メール送信）、wait（待機）、condition（条件分岐）、tag（タグ付け）
- フォーム送信から購読者作成・シーケンス登録まで完全自動化

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

## 🚫 よくある愚行と防止策 / Common Mistakes and Prevention

### 1. テストを通すためにロジックを変更する / Changing Logic to Pass Tests

- ❌ 悪い例 /
  Bad: テストが失敗したので、テストに合わせてビジネスロジックを変更 / Change
  business logic to match tests
- ✅ 良い例 /
  Good: ロジックが正しい場合はテストを修正、バグがある場合はロジックを修正 / Fix
  tests if logic is correct, fix logic if it has bugs

### 2. テストをスキップしてプッシュ / Skipping Tests to Push

- ❌ **最悪の例 / WORST**: `git push --no-verify` でテストをスキップ / Skip
  tests with `git push --no-verify`
- ❌ 悪い例 / Bad: テストが失敗したので `#[ignore]` を追加 / Add `#[ignore]`
  when tests fail
- ✅ 良い例 / Good: テストが失敗した原因を調査し、問題を解決してからプッシュ /
  Investigate failure cause and fix before pushing

### 3. マイグレーションファイルの削除・変更 / Deleting or Modifying Migration Files

- ❌ 悪い例 / Bad: エラーが出たので既存のマイグレーションファイルを削除 / Delete
  existing migration files when errors occur
- ❌ 悪い例 / Bad: 既存のマイグレーションファイルを直接編集 / Edit existing
  migration files directly
- ✅ 良い例 / Good: 新しいタイムスタンプで修正用のマイグレーションを追加 / Add
  new migration with new timestamp

### 4. エラーを握りつぶす / Suppressing Errors

- ❌ 悪い例 / Bad: `unwrap()`でエラーが出たので`.unwrap_or_default()`に変更 /
  Change to `.unwrap_or_default()` when `unwrap()` fails
- ✅ 良い例 / Good: エラーの原因を調査し、適切なエラーハンドリングを実装 /
  Investigate error cause and implement proper handling

### 5. 作業ディレクトリの混乱 / Working Directory Confusion

- ❌ 悪い例 / Bad: 現在のディレクトリを確認せずにコマンド実行 / Execute commands
  without checking current directory
- ✅ 良い例 / Good:
  `pwd`で常に現在位置を確認、適切なディレクトリに移動してから作業 / Always check
  with `pwd` and navigate to correct directory

## 🚨 新規サービス実装時の必須チェックリスト

新しいサービスファイルを作成する際は、**必ず既存のサービスファイルのパターンを参照**してください。

### 1. API URLの構築パターンを統一する

```typescript
// ❌ 悪い例：独自のパターンを作る
const API_BASE = import.meta.env.VITE_API_URL || "http://localhost:3000";
const response = await fetch(`${API_BASE}/api${path}`, ...);

// ✅ 良い例：既存のパターンに従う
const API_BASE_URL = "/api";
const response = await fetch(`${API_BASE_URL}${path}`, ...);
```

### 2. 新規ファイル作成前の確認手順

1. **既存の類似ファイルを検索**

   ```bash
   # 例：新しいサービスを作る前に
   find . -name "*Service.ts" -o -name "*service.ts"
   ```

2. **既存のパターンを確認**

   ```bash
   # 例：API呼び出しパターンを確認
   grep -r "fetch.*api" --include="*.ts"
   ```

3. **最も類似したファイルをベースにする**
   - `api.ts` のような基本的なサービスファイルを参考にする
   - 独自のパターンを発明しない

### 3. 環境変数の使用を避ける

- フロントエンドでは相対パスを使用（`/api`）
- 環境依存の設定は最小限に
- 既存のサービスが環境変数を使っていない場合は使わない

### 4. コードレビューチェックリスト

- [ ] 既存のサービスファイルと同じパターンを使用しているか
- [ ] API URLの構築方法が統一されているか
- [ ] エラーハンドリングが一貫しているか
- [ ] 認証トークンの扱いが統一されているか
- [ ] TypeScriptの型定義が適切か

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

## 🔧 新機能実装の推奨手順

### 1. データベース制約の事前確認

新機能を実装する前に、必ずデータベースの制約を確認する：

```bash
# テーブル構造と制約の確認
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "\d テーブル名"

# 特に以下を確認
# - CHECK制約（許可される値）
# - UNIQUE制約（重複を許さないカラム）
# - 外部キー制約
# - データ型（特にUUID vs INTEGER）
```

### 2. バックエンドとフロントエンドの型整合性

実装前に以下を確認：

1. **バックエンドのモデル定義** (`backend/src/models/`)

   - フィールド名（snake_case）
   - データ型（UUID、String、i32等）
   - 必須/オプショナルフィールド

2. **データベーススキーマ** (`backend/migrations/`)

   - カラム名と型
   - 制約（CHECK、UNIQUE等）
   - デフォルト値

3. **フロントエンドの型定義** (`frontend/src/lib/types/`)
   - バックエンドと一致する型定義
   - IDは通常`string`（UUID）
   - ステータスやタイプのenum値が一致

### 3. API実装時の確認事項

1. **エンドポイントの確認**

   ```bash
   # backend/src/api/mod.rs でルーティングを確認
   grep -n "route.*api" backend/src/api/mod.rs
   ```

2. **特殊なエンドポイントの把握**

   - 詳細取得: `/api/resources/:id` vs `/api/resources/:id/full`
   - ネストしたリソース: `/api/resources/:id/sub-resources`

3. **レスポンス形式の確認**
   - 単一オブジェクト vs ラッパーオブジェクト
   - ページネーション形式

### 4. よくある実装ミスと対策

#### ❌ 型の不一致

```typescript
// 悪い例
type Status = 'active' | 'inactive'; // DBは 'draft' も含む

// 良い例 - DBの制約を先に確認
type Status = 'draft' | 'active' | 'inactive';
```

#### ❌ フィールド名の不一致

```typescript
// 悪い例
trigger_conditions?: Record<string, any>;  // DBは trigger_config

// 良い例 - バックエンドのモデルと一致
trigger_config?: Record<string, any>;
```

#### ❌ 重複エラーの未考慮

```typescript
// 悪い例
step_order: steps.length + 1; // 削除後に重複する可能性

// 良い例
step_order: Math.max(...steps.map(s => s.step_order)) + 1;
```

#### ❌ Enum型の文字列比較

```rust
// 悪い例
if sequence.trigger_type == TriggerType::FormSubmission {
    // 型エラー: String != TriggerType
}

// 良い例 - as_str()メソッドを使用
if sequence.trigger_type == TriggerType::FormSubmission.as_str() {
    // 正常に動作
}
```

#### ❌ 非同期タスクのエラーハンドリング

```rust
// 悪い例
tokio::spawn(async move {
    process_sequences().await; // エラーが握りつぶされる
});

// 良い例 - エラーをログ出力
tokio::spawn(async move {
    if let Err(e) = process_sequences().await {
        error!("シーケンス処理エラー: {}", e);
    }
});
```

### 5. デバッグ手順

1. **エラー発生時はまずログを確認**

   - ブラウザのコンソール
   - バックエンドのターミナル出力

2. **データベースの実データ確認**

   ```bash
   docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "SELECT * FROM table_name;"
   ```

3. **API通信の確認**
   - ブラウザの開発者ツール > Network タブ
   - リクエスト/レスポンスのペイロード確認

## ⚠️ AWS CDKデプロイ時の必須事項 / Critical Requirements for AWS CDK Deployment

### ドメイン環境変数の設定が必須 / Domain Environment Variables are REQUIRED

**問題**: ドメイン環境変数が設定されていない場合、以下の深刻な問題が発生します：

1. ALBStackがHTTPSリスナーを作成しない
2. ECSServiceStackがHTTPSリスナーを参照しようとして失敗
3. 両スタックが相互依存でUPDATE_ROLLBACK_COMPLETE状態になる
4. MonitoringStackなど依存スタックもデプロイできなくなる

**解決策**: CDKデプロイ前に必ず環境変数を設定

```bash
# 開発環境の場合
export DEV_DOMAIN=dev.markmail.engineers-hub.ltd

# ステージング環境の場合
export STAGING_DOMAIN=staging.markmail.engineers-hub.ltd

# 本番環境の場合
export PROD_DOMAIN=markmail.engineers-hub.ltd

# デプロイコマンド実行
npm run cdk -- deploy StackName --profile your-profile
```

**絶対にやってはいけないこと**:

- ❌ 環境変数なしでCDKデプロイを実行
- ❌ AWS CLIで手動でリソースを作成・修正
- ❌ スタック間の依存関係を無視した操作

## 🔧 AWS RDS操作方法 / How to Operate AWS RDS

### RDSへの接続方法 / How to Connect to RDS

AWS環境のRDSはセキュリティ要件により直接接続できません。以下の方法で接続します：

#### 1. 踏み台ホスト（Bastion Host）経由での接続 / Connection via Bastion Host

```bash
# 踏み台ホストの作成 / Create bastion host
cd infrastructure
CREATE_BASTION=true npm run cdk -- deploy MarkMail-dev-BastionStack --profile your-profile

# 踏み台ホストのインスタンスIDを取得 / Get bastion host instance ID
aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  --query 'Reservations[*].Instances[*].[InstanceId]' \
  --output text \
  --profile your-profile

# SSM Session Manager経由で接続 / Connect via SSM Session Manager
aws ssm start-session \
  --target i-xxxxxxxxxxxxx \
  --profile your-profile

# 踏み台ホスト内からRDSに接続 / Connect to RDS from bastion host
PGPASSWORD=your-password psql \
  -h your-rds-endpoint.rds.amazonaws.com \
  -U markmail \
  -d markmail
```

#### 2. SSM Send Commandでのリモート実行 / Remote Execution via SSM Send Command

```bash
# コマンドを実行 / Execute command
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=["your-command-here"]' \
  --profile your-profile \
  --query 'Command.CommandId' \
  --output text

# 実行結果を確認 / Check execution result
aws ssm get-command-invocation \
  --command-id command-id-here \
  --instance-id i-xxxxxxxxxxxxx \
  --profile your-profile
```

### データベースマイグレーション / Database Migration

#### ECS経由での自動マイグレーション / Automatic Migration via ECS

アプリケーション起動時に自動的にマイグレーションが実行されます：

```bash
# ECSサービスを強制的に再デプロイ / Force redeploy ECS service
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --force-new-deployment \
  --profile your-profile
```

#### 手動マイグレーション / Manual Migration

踏み台ホスト経由で手動実行する場合：

```bash
# 踏み台ホストでマイグレーションを実行 / Run migration on bastion host
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "git clone https://github.com/your-repo/markmail.git",
    "cd markmail/backend",
    "export DATABASE_URL=\"postgresql://user:pass@endpoint:5432/dbname\"",
    "sqlx migrate run"
  ]' \
  --profile your-profile
```

### データベースのリセット / Database Reset

⚠️ **警告 / WARNING**: 本番環境では絶対に実行しないでください / NEVER execute in
production

```bash
# 接続を強制終了してデータベースを再作成 / Terminate connections and recreate database
aws ssm send-command \
  --instance-ids i-xxxxxxxxxxxxx \
  --document-name "AWS-RunShellScript" \
  --parameters 'commands=[
    "export PGPASSWORD=\"your-password\"",
    "psql -h endpoint -U markmail -d postgres -c \"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '"'"'markmail'"'"' AND pid <> pg_backend_pid();\"",
    "psql -h endpoint -U markmail -d postgres -c \"DROP DATABASE IF EXISTS markmail;\"",
    "psql -h endpoint -U markmail -d postgres -c \"CREATE DATABASE markmail;\""
  ]' \
  --profile your-profile
```

### マイグレーションバージョンの不一致を解決 / Resolve Migration Version Mismatch

ローカルとAWS環境でマイグレーションのチェックサムが異なる場合：

1. **マイグレーション履歴を確認 / Check migration history**

   ```sql
   SELECT version, checksum FROM _sqlx_migrations ORDER BY version;
   ```

2. **チェックサムを更新 / Update checksum**

   ```sql
   UPDATE _sqlx_migrations
   SET checksum = 'new-checksum-here'
   WHERE version = 'version-number';
   ```

3. **特定のマイグレーションを削除して再実行 / Delete and rerun specific
   migration**
   ```sql
   DELETE FROM _sqlx_migrations WHERE version = 'version-number';
   ```

### トラブルシューティング / Troubleshooting

#### 踏み台ホストが見つからない / Bastion host not found

```bash
# インスタンスの状態を確認 / Check instance status
aws ec2 describe-instances \
  --filters "Name=tag:Name,Values=markmail-dev-bastion" \
  "Name=instance-state-name,Values=running,stopped" \
  --query 'Reservations[*].Instances[*].[InstanceId,State.Name]' \
  --output table \
  --profile your-profile
```

#### RDSエンドポイントの確認 / Check RDS endpoint

```bash
aws rds describe-db-instances \
  --query 'DBInstances[*].[DBInstanceIdentifier,Endpoint.Address]' \
  --output table \
  --profile your-profile
```

#### データベースパスワードの取得 / Get database password

```bash
aws secretsmanager get-secret-value \
  --secret-id markmail-dev-db-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq -r '.password'
```

### 重要な注意事項 / Important Notes

- **踏み台ホストは一時的なリソース** / Bastion host is a temporary resource
- **使用後は削除を検討** / Consider deletion after use
- **本番環境では特に慎重に操作** / Be extra careful in production
- **データベースのバックアップを確認** / Verify database backups exist

## 🔐 AWS Secrets Manager でのAI API キー管理 / Managing AI API Keys with AWS Secrets Manager

### AI用シークレットの初期設定 / Initial Setup for AI Secrets

AWS環境では、AI関連のAPIキー（OPENAI_API_KEY、ANTHROPIC_API_KEY等）はSecrets
Managerで管理されます。

#### 1. シークレットの更新 / Update Secrets

```bash
# シークレットの内容を更新 / Update secret values
aws secretsmanager update-secret \
  --secret-id markmail-dev-ai-secret \
  --secret-string '{
    "OPENAI_API_KEY": "sk-xxxxxxxxxxxxxxxxxxxxxxxx",
    "ANTHROPIC_API_KEY": "sk-ant-xxxxxxxxxxxxxxxx",
    "AI_PROVIDER": "openai",
    "OPENAI_MODEL": "gpt-4",
    "ANTHROPIC_MODEL": "claude-3-opus-20240229"
  }' \
  --profile your-profile
```

#### 2. シークレットの確認 / Verify Secrets

```bash
# 現在の値を確認（注意：実際のAPIキーが表示されます） / Verify current values (WARNING: displays actual API keys)
aws secretsmanager get-secret-value \
  --secret-id markmail-dev-ai-secret \
  --query 'SecretString' \
  --output text \
  --profile your-profile | jq '.'
```

#### 3. ECSサービスの再デプロイ / Redeploy ECS Service

シークレットを更新した後は、ECSサービスを再デプロイして新しい値を反映させます：

```bash
# バックエンドサービスを強制的に再デプロイ / Force redeploy backend service
aws ecs update-service \
  --cluster markmail-dev \
  --service markmail-dev-backend \
  --force-new-deployment \
  --profile your-profile
```

#### 4. シークレットのローテーション / Secret Rotation

定期的にAPIキーをローテーションすることを推奨します：

```bash
# 新しいAPIキーでシークレットを更新 / Update secret with new API key
aws secretsmanager update-secret \
  --secret-id markmail-dev-ai-secret \
  --secret-string '{
    "OPENAI_API_KEY": "sk-new-key-xxxxxxxxxxxxxxxx",
    "ANTHROPIC_API_KEY": "sk-ant-new-key-xxxxxxxxx",
    "AI_PROVIDER": "openai",
    "OPENAI_MODEL": "gpt-4",
    "ANTHROPIC_MODEL": "claude-3-opus-20240229"
  }' \
  --profile your-profile
```

### 注意事項 / Important Notes

- **環境ごとにシークレットは分離** / Secrets are separated by environment (dev,
  staging, prod)
- **シークレット名の規則** / Secret naming convention:
  `markmail-{environment}-ai-secret`
- **ECSタスクは自動的に最新のシークレットを取得** / ECS tasks automatically
  fetch the latest secrets
- **ローカル開発では`.env`ファイルを使用** / Use `.env` file for local
  development

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
