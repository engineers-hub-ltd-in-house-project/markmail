# ADR-001: Salesforce OAuth2実装への移行

## Status

提案中（Proposed）

## Context

現在のSalesforce統合は、Salesforce
CLIを使用してアクセストークンを取得している。この方法には以下の問題がある：

1. **トークン有効期限の問題**

   - アクセストークンは約2時間で期限切れになる
   - CLIはリフレッシュトークンを内部管理するため、アプリケーションから直接アクセスできない
   - 手動でトークンを更新する必要がある

2. **AWS環境での制約**

   - AWS ECS/Fargateでの本番環境では、CLIのインストールと管理が困難
   - コンテナ環境でのCLI認証フローが複雑

3. **運用上の課題**
   - トークン期限切れによるサービス中断
   - 手動介入が必要で自動化が困難

## Decision

Salesforce CLIへの依存を排除し、OAuth2標準フローを直接実装する。

### 採用する技術スタック

- **OAuth2ライブラリ**: `oauth2` v5.0.0（Rustの標準的なOAuth2実装）
- **HTTPクライアント**: `reqwest`
  v0.11（非同期対応、本プロジェクトで既に使用中）
- **認証フロー**: OAuth 2.0 Web Server Flow with Refresh Token

### 実装方針

1. **完全なOAuth2フローの実装**

   - Authorization Code Grant with Refresh Token
   - 自動トークンリフレッシュ機能
   - エラー時の自動リトライ

2. **セキュリティ対策**

   - CSRF保護（stateパラメータ）
   - SSRF対策（リダイレクト無効化）
   - トークンの暗号化保存

3. **段階的移行**
   - 既存のCLI実装と共存可能な設計
   - フィーチャーフラグによる切り替え

## Consequences

### 利点

1. **安定性の向上**

   - 自動トークンリフレッシュによりサービス中断を防止
   - エラーハンドリングの強化

2. **運用の自動化**

   - 手動介入不要
   - AWS環境に適した実装

3. **保守性の向上**
   - 標準的なOAuth2実装
   - 外部CLIへの依存削除

### 欠点

1. **実装の複雑性**

   - OAuth2フローの完全実装が必要
   - Salesforce Connected Appの設定が必要

2. **移行リスク**
   - 既存の統合への影響
   - 新規バグの可能性

### 軽減策

1. **段階的移行**

   - 開発環境での十分なテスト
   - 既存実装との並行稼働期間の設定

2. **監視とアラート**
   - トークンリフレッシュの成功/失敗を監視
   - エラー時の通知設定

## Implementation Plan

### Phase 1: 基本実装（1週間）

- OAuth2クライアントの実装
- 認証エンドポイントの作成
- トークン管理機能

### Phase 2: 自動更新機能（1週間）

- リフレッシュトークンの実装
- エラーハンドリング
- リトライロジック

### Phase 3: 移行準備（1週間）

- 既存実装との統合
- フィーチャーフラグの実装
- テスト環境での検証

### Phase 4: 本番展開（1週間）

- ステージング環境での検証
- 本番環境への段階的展開
- モニタリング設定

## References

- [oauth2 crate documentation](https://docs.rs/oauth2/latest/oauth2/)
- [Salesforce OAuth 2.0 Web Server Flow](https://help.salesforce.com/s/articleView?id=sf.remoteaccess_oauth_web_server_flow.htm)
- [Salesforce Refresh Token Flow](https://help.salesforce.com/s/articleView?id=sf.remoteaccess_oauth_refresh_token_flow.htm)
- RFC 6749: The OAuth 2.0 Authorization Framework

## Appendix

### Salesforce Connected App設定

```
OAuth Settings:
- Enable OAuth Settings: ✓
- Callback URL: https://{your-domain}/api/crm/oauth/salesforce/callback
- Selected OAuth Scopes:
  - Access and manage your data (api)
  - Perform requests on your behalf at any time (refresh_token, offline_access)
- Refresh Token Policy: Refresh token is valid until revoked
- Permitted Users: All users may self-authorize
```

### 環境変数

```env
# Salesforce OAuth2設定
SALESFORCE_CLIENT_ID=your_client_id
SALESFORCE_CLIENT_SECRET=your_client_secret
SALESFORCE_REDIRECT_URI=https://your-domain/api/crm/oauth/salesforce/callback
```
