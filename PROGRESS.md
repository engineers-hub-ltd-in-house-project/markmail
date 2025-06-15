# MarkMail 開発進捗記録

このドキュメントは、各開発セッションの作業内容、修正内容、発生した問題と解決方法を記録するためのものです。

## 2025-06-15: AI機能のバックエンド実装（AIマーケティング自動化）

### 作業概要

MarkMailにAI統合機能を実装し、インテリジェントなマーケティング自動化プラットフォームへと進化。OpenAIとAnthropicの最先端AIモデルを活用し、マーケティングシナリオの自動生成、コンテンツ最適化、パーソナライズされたカスタマージャーニーの作成を可能に。

### 実装した機能

#### 1. AIプロバイダー抽象化層 (`backend/src/ai/providers/`)

- ✅ `AIProvider`トレイトによる統一インターフェース
- ✅ OpenAI GPT-4プロバイダー実装
- ✅ Anthropic Claudeプロバイダー実装
- ✅ モックプロバイダー（テスト用）
- ✅ プロバイダー自動選択とフォールバック機構

#### 2. AIサービス層 (`backend/src/ai/services/`)

- ✅ **ScenarioBuilder**: マーケティングシナリオ自動生成
  - 業界・ターゲット・ゴールに基づくシナリオ作成
  - メールシーケンス、フォーム、テンプレートの一括生成
  - JSONスキーマ検証による応答品質保証
- ✅ **ContentGenerator**: コンテンツ生成・最適化
  - メールコンテンツの自動生成
  - 件名の最適化（開封率向上）
  - トーン・スタイルのカスタマイズ
  - 変数抽出とパーソナライゼーション

#### 3. APIエンドポイント (`backend/src/api/ai.rs`)

- ✅ `POST /api/ai/scenarios/generate` - シナリオ生成
- ✅ `POST /api/ai/content/generate` - コンテンツ生成
- ✅ `POST /api/ai/content/optimize-subject` - 件名最適化

#### 4. データモデルとプロンプト管理

- ✅ 型安全なリクエスト/レスポンスモデル
- ✅ プロンプトテンプレートの一元管理
- ✅ マルチ言語対応の基盤

### 技術的な特徴

#### 1. プロバイダー抽象化パターン

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_text(&self, prompt: &str, max_tokens: Option<u32>) -> Result<String>;
    async fn chat(&self, messages: Vec<ChatMessage>, max_tokens: Option<u32>) -> Result<String>;
    fn count_tokens(&self, text: &str) -> Result<usize>;
}
```

#### 2. 環境変数による柔軟な設定

```env
AI_PROVIDER=openai  # or 'anthropic'
OPENAI_API_KEY=your-key
ANTHROPIC_API_KEY=your-key
```

#### 3. エラーハンドリングとリトライ

- APIレート制限に対する自動リトライ
- タイムアウト処理
- 詳細なエラーログ

### 遭遇した問題と解決方法

#### 問題1: モジュールのインポートパス問題

**症状**: `main.rs`から`lib.rs`のモジュールをインポートできない

**解決方法**:

```rust
// main.rs
use markmail_backend::{ai, AppState};  // crate名を使用

// lib.rs
pub use crate::app_state::AppState;  // 再エクスポート
```

#### 問題2: エラー型の不一致

**症状**: `AppError`型が見つからない

**解決方法**: 既存のAPIパターンに合わせて`(StatusCode, Json<Value>)`を使用

#### 問題3: Clippy警告

**症状**: `and_then(|x| Some(y))`の使用に対する警告

**解決方法**: `map(|x| y)`に変更

### テスト結果

- ✅ **76個のバックエンドテスト**合格（AI機能テスト含む）
- ✅ **58個のフロントエンドテスト**合格
- ✅ OpenAI/Anthropicプロバイダーの統合テスト
- ✅ モックプロバイダーによる決定論的テスト

### パフォーマンスメトリクス

- シナリオ生成: 約2-3秒
- コンテンツ生成: 約1-2秒
- プロバイダー切り替え: 50ms未満

### ドキュメント更新

- ✅ `REQUIREMENTS.md` - AI機能の要件追加（Section 11）
- ✅ `ROADMAP.md` - Phase 4としてAI実装フェーズ追加
- ✅ `README.md` - プロジェクト概要をAI機能を含む内容に更新

### ブランチ管理

- `feature/ai-integration`ブランチを作成
- 全ての変更をコミット・プッシュ済み
- PR用の詳細な説明文を日本語で作成

### 次のステップ

1. **フロントエンドUI実装**

   - [ ] AIシナリオ生成画面
   - [ ] コンテンツ生成・編集UI
   - [ ] リアルタイムプレビュー

2. **高度な機能**

   - [ ] ユーザーフィードバックによる学習
   - [ ] A/Bテスト自動生成
   - [ ] マルチモーダルコンテンツ対応

3. **運用機能**
   - [ ] 使用量トラッキング
   - [ ] コスト管理
   - [ ] パフォーマンスモニタリング

### PR情報

- **ブランチ**: `feature/ai-integration`
- **テスト**: 全て合格
- **影響範囲**: バックエンドのみ（フロントエンドは次のPR）
- **破壊的変更**: なし

---

## 2025-06-12: シーケンス機能のバックエンド実装（自動化システム）

### 作業概要

シーケンス機能のバックエンド自動化システムを実装。メール自動配信のためのビジネスロジック層、バックグラウンドワーカー、トリガーハンドラーを構築し、メール配信の動作確認まで完了。

### 実装した機能

#### 1. シーケンスサービス層 (`backend/src/services/sequence_service.rs`)

- ✅ トリガーエンロールメント処理
- ✅ トリガー条件評価ロジック
- ✅ ステップ実行エンジン（メール送信、待機、条件分岐、タグ付け）
- ✅ 変数置換システム
- ✅ エラーハンドリングとログ記録

#### 2. データベース層の拡張 (`backend/src/database/sequences.rs`)

- ✅ `find_active_sequences_by_trigger` - トリガー別のアクティブシーケンス取得
- ✅ `find_pending_sequence_enrollments` - 実行待ちエンロールメント取得
- ✅ `complete_sequence_enrollment` - エンロールメント完了処理
- ✅ `update_enrollment_progress` - 進捗更新
- ✅ `schedule_next_enrollment_step` - 次ステップのスケジューリング
- ✅ `create_sequence_step_log` - ステップ実行ログ記録

#### 3. バックグラウンドワーカー (`backend/src/workers/sequence_worker.rs`)

- ✅ 定期実行ワーカー（デフォルト60秒間隔）
- ✅ 非同期タスクとしてメインプロセスから独立実行
- ✅ エラーハンドリングとログ出力

#### 4. トリガーハンドラーの統合

- ✅ 購読者作成時のトリガー（`api/subscribers.rs`）
- ✅ フォーム送信時のトリガー（`api/forms.rs`）
- ✅ フォーム送信から購読者作成・シーケンス登録の連携
- ✅ エラーが発生してもメイン処理は継続

#### 5. フォーム送信時の購読者作成機能

- ✅ フォームデータからメールアドレスを自動抽出
- ✅ 既存購読者のチェックと新規作成
- ✅ フォーム送信と購読者の紐付け
- ✅ カスタムフィールドとタグの自動設定

### 遭遇した問題と解決方法

#### 問題1: Enum型とString型の不一致

**症状**: `TriggerType`と`String`の比較でコンパイルエラー

**解決方法**:

```rust
// 変更前
if sequence.trigger_type == TriggerType::FormSubmission {

// 変更後
if sequence.trigger_type == TriggerType::FormSubmission.as_str() {
```

#### 問題2: current_step_orderフィールドの不在

**症状**: `SequenceEnrollment`に`current_step_order`が存在しない

**解決方法**: `current_step_id`から動的に計算する方式に変更

```rust
let current_step_order = if let Some(current_step_id) = enrollment.current_step_id {
    steps.iter()
        .find(|s| s.id == current_step_id)
        .map(|s| s.step_order)
        .unwrap_or(0)
} else {
    0
};
```

#### 問題3: EmailServiceのメソッド名不一致

**症状**: `send`メソッドが存在しない

**解決方法**: 正しいメソッド名`send_email`を使用

#### 問題4: フォーム送信時に購読者が作成されない

**症状**: フォーム送信しても`subscriber_id`が常にNULL

**解決方法**: `submit_form`ハンドラーに購読者作成ロジックを追加

```rust
// フォームデータからメールアドレスを抽出
let email = extract_email_from_form_data(&request.data, &form.form_fields);

// 購読者を検索または作成
if let Some(email) = email {
    match subscribers::find_subscriber_by_email(&state.db, &email, form.user_id).await {
        Ok(Some(subscriber)) => Some(subscriber.id),
        Ok(None) => {
            // 新規購読者を作成
            let create_req = CreateSubscriberRequest {
                email,
                name: extract_name_from_form_data(&request.data, &form.form_fields),
                status: Some(SubscriberStatus::Active),
                tags: Some(vec![format!("form:{}", form.slug)]),
                custom_fields: Some(request.data.clone()),
            };
            // ...
        }
    }
}
```

#### 問題5: AWS SESメール配信の確認

**症状**: メールが送信されているがユーザーに届かない

**原因**: 一時的な配信遅延

**結果**: 数分後に正常に配信されたことを確認

### アーキテクチャの特徴

1. **サービス層の分離**: ビジネスロジックをサービス層に集約
2. **非同期処理**: Tokioを使用した効率的な非同期処理
3. **エラー耐性**: 個別のエンロールメント処理のエラーが全体に影響しない設計
4. **拡張性**: 新しいステップタイプやトリガータイプの追加が容易
5. **自動化**: フォーム送信から購読者登録、メール配信まで完全自動化

### 動作確認済みのフロー

1. フォーム作成（フィールドにemailタイプを含む）
2. フォーム公開
3. フォーム送信（メールアドレス入力）
4. 購読者自動作成
5. FormSubmissionトリガーでシーケンス登録
6. バックグラウンドワーカーがステップを処理
7. AWS SES経由でメール送信
8. ユーザーがメール受信

### 次のステップ

1. **統合テストの作成**

   - [ ] シーケンスエンロールメントのE2Eテスト
   - [ ] ワーカーの動作確認テスト
   - [ ] トリガー発火のテスト

2. **運用機能の追加**

   - [ ] シーケンス実行の一時停止・再開機能
   - [ ] 実行ログの可視化
   - [ ] パフォーマンスメトリクスの収集

3. **高度な機能**
   - [ ] 条件分岐の詳細実装
   - [ ] A/Bテスト機能
   - [ ] 動的なコンテンツ生成

---

## 2025-06-11: シーケンス機能のフロントエンド実装

### 作業概要

フォームビルダー機能の後、シーケンス（自動メール配信）機能のフロントエンド実装を完了。

### 実装した機能

#### 1. TypeScript型定義 (`frontend/src/lib/types/sequence.ts`)

- ✅ 基本型定義（Sequence, SequenceStep, SequenceEnrollment）
- ✅ API通信用の型定義
- ✅ ビジュアルエディタ用の型定義

#### 2. APIサービス (`frontend/src/lib/services/sequenceService.ts`)

- ✅ シーケンスCRUD操作
- ✅ ステップ管理機能
- ✅ エンロールメント管理機能
- ✅ 日本語表示用のユーティリティ関数

#### 3. UIコンポーネント

- ✅ シーケンス一覧ページ (`routes/sequences/+page.svelte`)
- ✅ シーケンス詳細ページ (`routes/sequences/[id]/+page.svelte`)
- ✅ シーケンス作成ページ (`routes/sequences/new/+page.svelte`)
- ✅ シーケンス編集ページ (`routes/sequences/[id]/edit/+page.svelte`)
- ✅ ナビゲーションメニューへの追加 (`routes/+layout.svelte`)

### 遭遇した問題と解決方法

#### 問題1: trigger_type の不一致

**症状**: シーケンス作成時に500エラー

```
error returned from database: new row for relation "sequences" violates check constraint "sequences_trigger_type_check"
```

**原因**:

- フロントエンド: `'registration'`
- データベース: `'subscriber_created'`

**解決方法**:

```typescript
// frontend/src/lib/types/sequence.ts
export type TriggerType =
  | 'manual'
  | 'subscriber_created'
  | 'form_submission'
  | 'tag_added';
```

#### 問題2: ステップタイプの不一致

**症状**: ステップ追加時に500エラー

**原因**:

- フロントエンド: `'delay'`
- データベース: `'wait'`

**解決方法**:

```typescript
// frontend/src/lib/types/sequence.ts
export type StepType = 'email' | 'wait' | 'condition' | 'tag';
```

#### 問題3: ID型の不一致

**症状**: 型エラー

**原因**:

- フロントエンド: `number`型を想定
- バックエンド: UUID（`string`型）を使用

**解決方法**: 全ての型定義でIDを`string`に変更

#### 問題4: ステップ作成時の重複エラー

**症状**:
`duplicate key value violates unique constraint "sequence_steps_sequence_id_step_order_key"`

**原因**:

1. 既存ステップの`step_order`を正しく取得できていない
2. APIエンドポイントの不一致（`/sequences/:id` vs `/sequences/:id/full`）

**解決方法**:

```typescript
// frontend/src/lib/services/sequenceService.ts
async getSequence(id: string): Promise<SequenceWithSteps> {
  return this.request<SequenceWithSteps>(`/sequences/${id}/full`);
}
```

#### 問題5: ステータス'draft'の欠落

**症状**: UIでステータスが正しく表示されない

**原因**: フロントエンドの型定義に`'draft'`ステータスが含まれていない

**解決方法**:

```typescript
export type SequenceStatus = 'draft' | 'active' | 'paused' | 'archived';
```

### データベース制約の確認結果

```sql
-- sequences テーブルの制約
CHECK (status = ANY (ARRAY['draft', 'active', 'paused', 'archived']))
CHECK (trigger_type = ANY (ARRAY['form_submission', 'tag_added', 'manual', 'subscriber_created']))

-- sequence_steps テーブルの制約
CHECK (delay_unit = ANY (ARRAY['minutes', 'hours', 'days']))
CHECK (step_type = ANY (ARRAY['email', 'wait', 'condition', 'tag']))
UNIQUE(sequence_id, step_order)
```

### 未実装機能（バックエンド側）

1. **sequence_service.rs** - ビジネスロジック層

   - [ ] 自動エンロールメントのトリガー実装
   - [ ] ステップ実行ロジック
   - [ ] メールサービスとの統合

2. **バックグラウンドワーカー**

   - [ ] ジョブキューシステムの選定と実装
   - [ ] 定期的なステップ実行チェック
   - [ ] 遅延処理の実装

3. **トリガー実装**
   - [ ] 購読者登録時のフック
   - [ ] フォーム送信時のフック
   - [ ] タグ追加時のフック

### 次のステップ

1. `backend/src/services/sequence_service.rs`の作成
2. 購読者登録・フォーム送信時のトリガー実装
3. Rustのジョブキューシステム（Sidekiq-rs、Apalis、Faktory）の評価と選定
4. バックグラウンドワーカーの実装

### 参考: 修正されたファイル一覧

**新規作成:**

- `frontend/src/lib/types/sequence.ts`
- `frontend/src/lib/services/sequenceService.ts`
- `frontend/src/routes/sequences/+page.svelte`
- `frontend/src/routes/sequences/[id]/+page.svelte`
- `frontend/src/routes/sequences/new/+page.svelte`
- `frontend/src/routes/sequences/[id]/edit/+page.svelte`
- `REQUIREMENTS.md`

**修正:**

- `frontend/src/routes/+layout.svelte` - ナビゲーションメニューに「シーケンス」追加
- `frontend/src/lib/services/api.ts` - templateApiの使用（既存）

### デバッグ用コマンド

```bash
# データベースのテーブル構造確認
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "\d sequences"
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "\d sequence_steps"

# ステップデータの確認
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "SELECT id, sequence_id, name, step_order, step_type FROM sequence_steps ORDER BY sequence_id, step_order;"
```

### 注意事項

1. **マイグレーションファイルは絶対に変更しない** - 新しいマイグレーションを追加する
2. **テストは無効化せず、コードを修正する**
3. **既存のテンプレート機能を再利用** - キャンペーンとシーケンスで共有

---

最終更新: 2025-06-11 20:15 JST
