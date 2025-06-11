# MarkMail 開発進捗記録

このドキュメントは、各開発セッションの作業内容、修正内容、発生した問題と解決方法を記録するためのものです。

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
