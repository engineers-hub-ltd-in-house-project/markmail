# アーキテクチャ決定記録

このドキュメントでは、Salesforce統合における主要な技術的決定事項を記録します。

## ADR-001: Salesforce OAuth2実装方針

### ステータス

承認済み（2025-07-30）

### コンテキスト

MarkMailにSalesforce
CRM統合を追加する必要があり、安全で拡張可能な認証メカニズムが必要でした。

### 決定事項

1. **OAuth2 Web Server Flowの採用**

   - ユーザー認証情報を直接扱わない
   - Salesforceの標準フロー
   - リフレッシュトークンによる長期アクセス

2. **直接HTTPリクエストの実装**

   - `oauth2`クレートではなく`reqwest`を使用
   - 理由: Salesforce固有のフィールド（instance_url）への対応
   - より柔軟なエラーハンドリングが可能

3. **トークン管理戦略**
   - PostgreSQLにトークンを保存
   - 自動リフレッシュメカニズム
   - instance_urlの保存と使用

### 結果

**メリット**:

- Salesforce APIの全機能にアクセス可能
- セキュアな認証フロー
- 自動トークンリフレッシュ

**デメリット**:

- カスタム実装の保守が必要
- oauth2標準ライブラリの恩恵を受けられない

## ADR-002: フォーム統合アーキテクチャ

### ステータス

承認済み（2025-07-29）

### コンテキスト

フォーム送信データを自動的にSalesforceリードとして作成する必要がありました。

### 決定事項

1. **非同期処理の採用**

   ```rust
   // フォーム送信とリード作成を分離
   tokio::spawn(async move {
       create_salesforce_lead(data).await;
   });
   ```

2. **フィールドマッピング設計**

   - 設定可能なマッピング
   - カスタムフィールドのサポート
   - 型安全性の確保

3. **エラーハンドリング**
   - フォーム送信は成功させる
   - CRM作成エラーは別途記録
   - リトライメカニズム

### 結果

- フォーム送信のUXが向上
- CRM障害がフォーム機能に影響しない
- 柔軟なフィールドマッピング

## ADR-003: マルチCRMサポート設計

### ステータス

計画中

### コンテキスト

将来的にHubSpot、Pipedrive等の他CRMもサポートする可能性があります。

### 決定事項

1. **プロバイダー抽象化**

   ```rust
   trait CRMProvider {
       async fn create_lead(&self, data: LeadData) -> Result<String, Error>;
       async fn update_lead(&self, id: &str, data: LeadData) -> Result<(), Error>;
   }
   ```

2. **統一データモデル**

   - 共通フィールドの定義
   - プロバイダー固有フィールドの処理
   - 変換ロジックの分離

3. **設定管理**
   - プロバイダーごとの設定
   - ユーザーレベルの切り替え
   - 複数プロバイダーの同時使用

### 結果

- 新しいCRMの追加が容易
- 既存コードへの影響を最小化
- 統一的なAPI

## ADR-004: セキュリティ設計

### ステータス

実装済み

### コンテキスト

CRM統合では機密性の高いデータを扱うため、厳格なセキュリティが必要です。

### 決定事項

1. **シークレット管理**

   - 環境変数からAWS Secrets Managerへ
   - 自動ローテーション対応
   - 監査ログ

2. **アクセス制御**

   - ユーザーごとのトークン管理
   - JWTによる認証
   - ロールベースアクセス制御

3. **データ保護**
   - トークンの暗号化検討
   - HTTPSのみの通信
   - ログからの機密情報除外

### 結果

- 企業レベルのセキュリティ
- コンプライアンス対応
- 監査可能性

## ADR-005: パフォーマンス戦略

### ステータス

部分的実装

### コンテキスト

大量のフォーム送信に対応できるスケーラブルなシステムが必要です。

### 決定事項

1. **バッチ処理**

   - Salesforce Composite APIの活用
   - 最大200件の一括処理
   - エラー処理の個別化

2. **キャッシング**

   - フィールドメタデータのキャッシュ
   - ピックリスト値のキャッシュ
   - Redisによる実装

3. **レート制限対応**
   - API使用量の追跡
   - 自動バックオフ
   - キューイングシステム

### 結果

- 高スループット対応
- API制限の有効活用
- 安定したパフォーマンス

## ADR-006: テスト戦略

### ステータス

実装済み

### コンテキスト

外部APIに依存するコードの信頼性を確保する必要があります。

### 決定事項

1. **モックサーバーの使用**

   ```rust
   #[cfg(test)]
   mod tests {
       use mockito::{mock, Matcher};

       #[tokio::test]
       async fn test_create_lead() {
           let _m = mock("POST", "/services/data/v59.0/sobjects/Lead")
               .with_status(201)
               .with_body(r#"{"id": "00Q123", "success": true}"#)
               .create();
       }
   }
   ```

2. **統合テスト**

   - Sandboxアカウントの使用
   - CI/CDでの自動実行
   - テストデータの自動クリーンアップ

3. **E2Eテスト**
   - フォーム送信からリード作成まで
   - 定期的な本番環境テスト
   - アラート設定

### 結果

- 高い信頼性
- リグレッションの早期発見
- 継続的な品質保証

## ADR-007: エラーハンドリングとリトライ

### ステータス

計画中

### コンテキスト

ネットワークエラーや短期的な障害に対する耐性が必要です。

### 決定事項

1. **指数バックオフ**

   ```rust
   retry::retry_with_index(Fixed::from_millis(100).take(3), |current_try| {
       match create_lead(&data).await {
           Ok(result) => OperationResult::Ok(result),
           Err(e) if is_retryable(&e) => OperationResult::Retry(e),
           Err(e) => OperationResult::Err(e),
       }
   })
   ```

2. **エラー分類**

   - 短期的エラー（リトライ可能）
   - 永続的エラー（リトライ不可）
   - 認証エラー（トークンリフレッシュ）

3. **デッドレターキュー**
   - 失敗したリクエストの保存
   - 手動リトライ機能
   - エラー分析

### 結果

- 高可用性
- 自動復旧
- 問題の可視化

## ADR-008: 監視とロギング

### ステータス

実装済み

### コンテキスト

本番環境での問題を迅速に検出し、解決する必要があります。

### 決定事項

1. **構造化ログ**

   ```rust
   use tracing::{info, error, instrument};

   #[instrument(skip(pool, lead_data))]
   async fn create_lead(pool: &PgPool, lead_data: LeadData) {
       info!(lead_email = %lead_data.email, "Creating lead");
   }
   ```

2. **メトリクス収集**

   - API呼び出し回数
   - 成功/失敗率
   - レスポンスタイム

3. **アラート設定**
   - エラー率の閾値
   - API制限の警告
   - トークン期限切れ

### 結果

- 問題の早期発見
- パフォーマンスの可視化
- 迅速なトラブルシューティング

## 今後の検討事項

1. **Webhook統合**

   - Salesforceからのリアルタイム更新
   - 双方向同期

2. **データ同期**

   - 定期的な同期ジョブ
   - 差分更新

3. **高度なマッピング**

   - 条件付きマッピング
   - データ変換ルール

4. **分析機能**
   - リード品質スコアリング
   - コンバージョン追跡

## 関連ドキュメント

- [OAuth2実装仕様](./oauth2-implementation-spec.md)
- [Salesforce統合計画](./salesforce-integration-plan.md)
- [インフラストラクチャ設計](../../infrastructure/README.md)
