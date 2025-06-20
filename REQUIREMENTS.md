# MarkMail 要件定義書

## 1. システム概要

MarkMailは、マークダウンベースのメールマーケティング自動化ツールです。

### 1.1 主要機能

1. **認証・ユーザー管理**

   - JWT認証（アクセストークン: 24時間、リフレッシュトークン: 30日）
   - ユーザー登録・ログイン・ログアウト
   - パスワードリセット機能

2. **テンプレート管理**

   - マークダウン形式のメールテンプレート作成・編集
   - 変数サポート（`{{variable_name}}`形式）
   - HTMLプレビュー機能
   - テンプレートの共有（キャンペーン・シーケンスで利用）

3. **キャンペーン管理**

   - 一括メール配信キャンペーンの作成・管理
   - ステータス管理（draft → scheduled → sending → completed）
   - 配信予約機能
   - 配信結果の追跡

4. **購読者管理**

   - 連絡先の登録・編集・削除
   - タグによる分類
   - カスタムフィールド（JSON形式）
   - CSVインポート機能
   - 購読解除管理

5. **フォームビルダー**

   - ドラッグ&ドロップでフォーム作成
   - 複数のフィールドタイプサポート
   - 公開URLでの外部アクセス
   - フォーム送信データの管理
   - 送信後のアクション設定

6. **メールシーケンス（自動化）**
   - 複数ステップの自動メール配信
   - トリガーベースの開始（登録時、フォーム送信時など）
   - 遅延・条件分岐サポート
   - 購読者ごとの進行状況追跡

## 2. 技術要件

### 2.1 アーキテクチャ

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Frontend   │────▶│   Backend   │────▶│  Database   │
│  (SvelteKit)│     │   (Rust)    │     │ (PostgreSQL)│
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │Email Service│
                    │(SES/MailHog)│
                    └─────────────┘
```

### 2.2 バックエンド要件

- **言語・フレームワーク**: Rust + Axum
- **データベース**: PostgreSQL + SQLx
- **認証**: JWT（RS256アルゴリズム）
- **メール配信**: マルチプロバイダー対応（開発: MailHog、本番: AWS SES）
- **非同期処理**: Tokio

### 2.3 フロントエンド要件

- **フレームワーク**: SvelteKit（SPAモード）
- **言語**: TypeScript
- **スタイリング**: Tailwind CSS
- **状態管理**: Svelte Stores
- **認証**: LocalStorage + JWT

### 2.4 インフラストラクチャ要件

- **コンテナ化**: Docker
- **オーケストレーション**: AWS ECS Fargate
- **データベース**: AWS RDS Aurora Serverless v2
- **ロードバランサー**: AWS ALB
- **CI/CD**: AWS CodePipeline + GitHub
- **IaC**: AWS CDK (TypeScript)

## 3. データモデル

### 3.1 主要エンティティ

1. **Users**

   - id, email, password_hash, name, created_at, updated_at

2. **Templates**

   - id, user_id, name, subject, content, created_at, updated_at

3. **Campaigns**

   - id, user_id, name, template_id, status, scheduled_for, created_at,
     updated_at

4. **Subscribers**

   - id, user_id, email, name, tags, custom_fields, status, created_at,
     updated_at

5. **Forms**

   - id, user_id, name, description, form_fields, settings, is_published,
     created_at, updated_at

6. **Sequences**

   - id, user_id, name, description, trigger_type, trigger_conditions, status,
     created_at, updated_at

7. **SequenceSteps**

   - id, sequence_id, step_order, step_type, delay_minutes, template_id,
     conditions, created_at, updated_at

8. **SequenceEnrollments**
   - id, sequence_id, subscriber_id, current_step, status, enrolled_at,
     completed_at

### 3.2 リレーションシップ

- Users → Templates, Campaigns, Subscribers, Forms, Sequences (1:N)
- Campaigns → Templates (N:1)
- Sequences → SequenceSteps (1:N)
- SequenceSteps → Templates (N:1)
- Subscribers → SequenceEnrollments (1:N)

## 4. API仕様

### 4.1 認証API

- `POST /api/auth/register` - ユーザー登録
- `POST /api/auth/login` - ログイン
- `POST /api/auth/refresh` - トークンリフレッシュ
- `POST /api/auth/logout` - ログアウト

### 4.2 リソースAPI（認証必須）

- **Templates**: CRUD操作 + プレビュー
- **Campaigns**: CRUD操作 + 送信機能
- **Subscribers**: CRUD操作 + インポート + タグ管理
- **Forms**: CRUD操作 + 公開設定 + 送信データ取得
- **Sequences**: CRUD操作 + ステップ管理 + 登録管理

## 5. セキュリティ要件

1. **認証・認可**

   - JWT認証必須（公開フォーム以外）
   - ユーザーは自分のデータのみアクセス可能

2. **データ保護**

   - パスワードはbcryptでハッシュ化
   - SQLインジェクション対策（SQLx使用）
   - XSS対策（適切なエスケープ）

3. **通信**
   - HTTPS必須（本番環境）
   - CORS設定（開発: localhost:5173、本番: 同一ドメイン）

## 6. 新規実装要件（シーケンス機能）

### 6.1 バックグラウンドワーカー

**要件**:

- Rustベースのジョブキューシステムの導入
- Redis統合（AWS ElastiCache活用）
- 候補ライブラリ:
  - [Sidekiq-rs](https://github.com/sidekiq/sidekiq-rs) - Sidekiq互換
  - [Apalis](https://github.com/geofmureithi/apalis) - Rust製ジョブキュー
  - [Faktory](https://github.com/contribsys/faktory) + Rust client

**実装方針**:

1. 開発環境: Docker ComposeでRedis追加
2. 本番環境: AWS ElastiCache利用
3. ジョブタイプ:
   - シーケンスステップ実行
   - メール送信
   - 遅延実行

### 6.2 スケジューリングシステム

**要件**:

- 実行時刻をデータベースに保存
- 定期的なポーリング（1分間隔）
- 環境別の実装:
  - 開発: ワーカー内でのポーリング
  - 本番: AWS EventBridge + Lambda/ECS Task

**実装方針**:

1. `sequence_enrollments`テーブルに`next_step_at`カラム追加
2. ワーカーが定期的に実行待ちステップをチェック
3. 実行後、次のステップの実行時刻を計算・保存

### 6.3 テンプレート識別

**要件**:

- テンプレートがキャンペーン用かシーケンス用か識別可能に
- 既存テンプレートとの互換性維持

**実装方針**:

1. `templates`テーブルに`usage_type`カラム追加（'campaign', 'sequence', 'both'）
2. デフォルト値は'both'で既存データと互換性維持
3. UI上でフィルタリング機能提供

### 6.4 フロントエンド優先開発

**開発順序**:

1. **Phase 1: UI実装**

   - シーケンス一覧画面
   - シーケンス作成・編集画面
   - ステップビルダー（ビジュアルエディタ）
   - モックデータで動作確認

2. **Phase 2: バックエンド統合**

   - sequence_service.rs実装
   - APIとの接続
   - リアルタイム更新

3. **Phase 3: 自動化実装**
   - ワーカーシステム構築
   - トリガー実装
   - ステップ実行エンジン

## 7. パフォーマンス要件

1. **レスポンスタイム**

   - API: 95%タイルで200ms以内
   - ページロード: 3秒以内

2. **スケーラビリティ**

   - 同時接続ユーザー: 1000人
   - メール送信: 10,000通/時間

3. **可用性**
   - 稼働率: 99.9%（年間8.76時間のダウンタイム許容）

## 8. 運用要件

1. **監視**

   - AWS CloudWatchによるメトリクス収集
   - エラーログの集約
   - アラート設定

2. **バックアップ**

   - データベース: 日次自動バックアップ
   - 保持期間: 30日

3. **デプロイメント**
   - Blue/Greenデプロイメント
   - ロールバック機能

## 9. テスト要件

1. **単体テスト**

   - カバレッジ: 80%以上
   - 全ての公開API関数

2. **統合テスト**

   - API エンドポイントテスト
   - データベース操作テスト

3. **E2Eテスト**
   - 主要ユーザーフロー
   - クロスブラウザテスト

## 10. 制約事項

1. **技術的制約**

   - メール送信はプロバイダーのレート制限に従う
   - ファイルアップロードは10MB以下

2. **ビジネス制約**
   - 無料プランの制限（将来実装）
   - GDPR/個人情報保護法準拠

## 11. AI機能要件

### 11.1 AIマーケティングシナリオ自動生成

**概要**: プロンプト入力から完全なマーケティングファネルを自動構築

**機能要件**:

- ターゲット層とゴールをプロンプトで指定
- 業界別テンプレート選択（SaaS、EC、教育、不動産等）
- AIが以下を自動生成:
  - マルチステップシーケンス（5-10ステップ）
  - 各ステップのメールテンプレート
  - リードキャプチャフォーム
  - 条件分岐ロジック
  - 最適な送信タイミング
- 生成後の編集・カスタマイズ機能
- **出力言語選択**（日本語/英語でのシナリオ生成）

**技術要件**:

- LLM API統合（OpenAI GPT-4/Anthropic Claude）
- プロンプトテンプレートシステム
- JSONスキーマベースの生成結果検証
- **言語別システムプロンプト管理**

### 11.2 AIコンテンツアシスタント

**機能要件**:

- メールテンプレートの自動生成・改善
- 件名最適化（開封率向上）
- パーソナライゼーション変数の自動提案
- トーン調整（フォーマル/カジュアル）
- 多言語対応（日本語/英語）
- **出力言語選択機能**（ユーザーが希望する言語でAI出力を生成）

**技術要件**:

- リアルタイムコンテンツ生成API
- マークダウンパーサー統合
- A/Bテストバリエーション自動生成
- **多言語プロンプトテンプレート管理システム**
- **言語別のトークン使用量最適化**

### 11.3 インテリジェントセグメンテーション

**機能要件**:

- 購読者行動の機械学習分析
- 自動セグメント作成・更新
- エンゲージメントスコアリング（0-100）
- チャーン予測アラート
- セグメント別の最適配信時間予測

**技術要件**:

- 機械学習モデル統合（scikit-learn/TensorFlow）
- バッチ処理パイプライン
- リアルタイムスコアリング

### 11.4 スマートオートメーション最適化

**機能要件**:

- シーケンスパフォーマンスのAI分析
- ステップ順序の自動最適化提案
- 条件分岐の動的生成
- 送信タイミングの機械学習最適化
- 異常検知とアラート

### 11.5 技術アーキテクチャ

**バックエンド拡張**:

```
backend/src/
├── ai/
│   ├── mod.rs
│   ├── services/
│   │   ├── content_generator.rs    # コンテンツ生成
│   │   ├── scenario_builder.rs     # シナリオ構築
│   │   ├── segmentation.rs         # セグメンテーション
│   │   └── analytics.rs            # 分析・予測
│   ├── models/
│   │   ├── prompts.rs              # プロンプト管理
│   │   ├── ai_responses.rs         # レスポンス型
│   │   └── ml_models.rs            # MLモデル定義
│   └── providers/
│       ├── openai.rs               # OpenAI統合
│       ├── anthropic.rs            # Anthropic統合
│       └── ml_runtime.rs           # ML実行環境
```

**データベース拡張**:

```sql
-- AI生成コンテンツ管理
CREATE TABLE ai_generated_content (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    content_type VARCHAR(50) NOT NULL,
    prompt TEXT NOT NULL,
    generated_content TEXT NOT NULL,
    model_used VARCHAR(50),
    metadata JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AIシナリオテンプレート
CREATE TABLE ai_scenario_templates (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    industry VARCHAR(100) NOT NULL,
    goal VARCHAR(255) NOT NULL,
    template_data JSONB NOT NULL,
    success_metrics JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- MLモデルメタデータ
CREATE TABLE ml_models (
    id UUID PRIMARY KEY,
    model_name VARCHAR(255) NOT NULL,
    model_type VARCHAR(50) NOT NULL,
    version VARCHAR(20) NOT NULL,
    parameters JSONB,
    performance_metrics JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**API エンドポイント**:

- `POST /api/ai/scenarios/generate` - シナリオ自動生成（言語パラメータ対応）
- `POST /api/ai/content/generate` - コンテンツ生成（言語パラメータ対応）
- `POST /api/ai/content/improve` - 既存コンテンツ改善
- `POST /api/ai/segments/analyze` - セグメント分析
- `GET /api/ai/segments/suggestions` - セグメント提案
- `POST /api/ai/sequences/optimize` - シーケンス最適化
- `GET /api/ai/analytics/insights` - AIインサイト取得

### 11.6 AI倫理・セキュリティ要件

**プライバシー保護**:

- 個人情報のマスキング
- AI学習データからの除外オプション
- GDPR/個人情報保護法準拠

**コンテンツ安全性**:

- 生成コンテンツのフィルタリング
- ブランドガイドライン準拠チェック
- 人間によるレビューオプション

**透明性**:

- AI生成コンテンツの明示
- 意思決定プロセスの説明機能
- パフォーマンスメトリクスの可視化

## 12. 将来の拡張性

1. **追加AI機能**

   - 音声アシスタント統合
   - 画像生成（DALL-E/Stable Diffusion）
   - チャットボット自動生成
   - 予測分析ダッシュボード

2. **既存機能の拡張**

   - A/Bテスト機能（AI駆動）
   - 詳細な分析・レポート
   - Webhook連携
   - 他サービスとの統合（Zapier等）

3. **技術的拡張**
   - マルチテナント対応
   - 国際化（i18n）
   - モバイルアプリ
   - リアルタイムコラボレーション

## 13. サブスクリプション・料金プラン要件

### 13.1 料金プラン構成

#### Free（無料プラン）

**対象**: 個人・スタートアップ

**リソース制限**:

- コンタクト数: 100
- 月間メール送信数: 1,000
- キャンペーン数: 3
- テンプレート数: 5
- フォーム数: 3
- シーケンス数: 1
- シーケンスステップ数: 5
- フォーム送信数: 100/月
- ユーザー数: 1
- Webhook数: 0

**AI機能（制限付き）**:

- AI使用回数: 月10回まで
- シナリオ生成: 月3回まで
- コンテンツ生成: 月5回まで
- 件名最適化: 月2回まで

**利用不可機能**:

- API連携
- 高度な分析
- A/Bテスト
- カスタムドメイン
- 優先サポート

#### Pro（プロフェッショナルプラン）

**料金**: ¥4,980/月 **対象**: 成長企業

**リソース制限**:

- コンタクト数: 10,000
- 月間メール送信数: 100,000
- キャンペーン数: 50
- テンプレート数: 100
- フォーム数: 50
- シーケンス数: 20
- シーケンスステップ数: 50
- フォーム送信数: 10,000/月
- ユーザー数: 5
- Webhook数: 10

**AI機能（標準）**:

- AI使用回数: 月500回まで
- シナリオ生成: 月50回まで
- コンテンツ生成: 月300回まで
- 件名最適化: 月150回まで

**追加機能**:

- API連携（レート制限: 1000リクエスト/時）
- 高度な分析
- A/Bテスト
- カスタムMarkdownコンポーネント

#### Business（ビジネスプラン）

**料金**: ¥19,800/月 **対象**: エンタープライズ

**リソース制限**:

- コンタクト数: 100,000
- 月間メール送信数: 1,000,000
- キャンペーン数: 無制限
- テンプレート数: 無制限
- フォーム数: 無制限
- シーケンス数: 無制限
- シーケンスステップ数: 無制限
- フォーム送信数: 無制限
- ユーザー数: 無制限
- Webhook数: 無制限

**AI機能（無制限）**:

- 全AI機能を無制限で利用可能

**全機能利用可能**:

- API連携（レート制限なし）
- 高度な分析
- A/Bテスト
- カスタムドメイン
- ホワイトラベル
- 優先サポート

### 13.2 サブスクリプション技術要件

#### データベース設計

```sql
-- サブスクリプションプラン
CREATE TABLE subscription_plans (
    id UUID PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    price_monthly INTEGER NOT NULL,
    -- リソース制限
    contact_limit INTEGER,
    monthly_email_limit INTEGER,
    campaign_limit INTEGER,
    template_limit INTEGER,
    sequence_limit INTEGER,
    sequence_step_limit INTEGER,
    form_limit INTEGER,
    form_submission_limit INTEGER,
    user_limit INTEGER,
    webhook_limit INTEGER,
    -- AI制限
    ai_monthly_limit INTEGER,
    ai_scenario_limit INTEGER,
    ai_content_limit INTEGER,
    ai_subject_limit INTEGER,
    -- 機能フラグ
    custom_markdown_components BOOLEAN DEFAULT FALSE,
    ai_features BOOLEAN DEFAULT FALSE,
    advanced_analytics BOOLEAN DEFAULT FALSE,
    ab_testing BOOLEAN DEFAULT FALSE,
    api_access BOOLEAN DEFAULT FALSE,
    api_rate_limit INTEGER,
    priority_support BOOLEAN DEFAULT FALSE,
    custom_domain BOOLEAN DEFAULT FALSE,
    white_label BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ユーザーサブスクリプション
CREATE TABLE user_subscriptions (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    plan_id UUID REFERENCES subscription_plans(id),
    status VARCHAR(20) NOT NULL, -- active, cancelled, past_due, expired
    current_period_start TIMESTAMP NOT NULL,
    current_period_end TIMESTAMP NOT NULL,
    cancel_at_period_end BOOLEAN DEFAULT FALSE,
    stripe_subscription_id VARCHAR(255),
    stripe_customer_id VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 使用量記録
CREATE TABLE usage_records (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    resource_type VARCHAR(50) NOT NULL, -- email_sent, ai_usage, etc
    quantity INTEGER NOT NULL,
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI使用ログ
CREATE TABLE ai_usage_logs (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    feature_type VARCHAR(50) NOT NULL, -- scenario, content, subject
    prompt TEXT,
    response TEXT,
    tokens_used INTEGER,
    model_used VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 制限チェックミドルウェア

```rust
// サブスクリプション制限チェック
async fn check_subscription_limit(
    user_id: Uuid,
    resource_type: &str,
    db: &PgPool
) -> Result<bool, AppError>;

// AI使用回数チェック
async fn check_ai_usage_limit(
    user_id: Uuid,
    feature_type: &str,
    db: &PgPool
) -> Result<bool, AppError>;

// 機能アクセスチェック
async fn check_feature_access(
    user_id: Uuid,
    feature: &str,
    db: &PgPool
) -> Result<bool, AppError>;
```

### 13.3 決済システム統合（Stripe）

#### 必要なStripe機能

- Customer管理
- Subscription管理
- Payment Method管理
- Webhook処理
- 請求書・領収書発行

#### Webhook対応イベント

- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`
- `invoice.payment_succeeded`
- `invoice.payment_failed`

### 13.4 使用量トラッキング

#### リアルタイムトラッキング対象

- メール送信数
- AI API呼び出し回数
- APIリクエスト数
- ストレージ使用量

#### 月次リセット処理

- 毎月1日0時（JST）に使用量をリセット
- リセット前に使用量レポートを生成・保存

## 14. 用語定義

- **キャンペーン**: 一度に複数の購読者に送信するメール配信
- **シーケンス**: トリガーに基づいて自動的に送信される一連のメール
- **ステップ**: シーケンス内の個別のアクション（メール送信、遅延など）
- **エンロールメント**: 購読者がシーケンスに登録された状態
- **トリガー**: シーケンスを開始するイベント（登録、フォーム送信など）
- **サブスクリプション**: ユーザーが契約している料金プラン
- **使用量制限**: プランごとに設定されたリソース使用の上限
- **AI使用回数**: AI機能を呼び出した回数（月次カウント）
