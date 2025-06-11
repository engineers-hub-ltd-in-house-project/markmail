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

## 11. 将来の拡張性

1. **機能拡張**

   - A/Bテスト機能
   - 詳細な分析・レポート
   - Webhook連携
   - 他サービスとの統合（Zapier等）

2. **技術的拡張**
   - マルチテナント対応
   - 国際化（i18n）
   - モバイルアプリ

## 12. 用語定義

- **キャンペーン**: 一度に複数の購読者に送信するメール配信
- **シーケンス**: トリガーに基づいて自動的に送信される一連のメール
- **ステップ**: シーケンス内の個別のアクション（メール送信、遅延など）
- **エンロールメント**: 購読者がシーケンスに登録された状態
- **トリガー**: シーケンスを開始するイベント（登録、フォーム送信など）
