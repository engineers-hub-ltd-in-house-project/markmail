# Salesforce Integration Documentation

このドキュメントは、MarkMailのSalesforce統合に関する包括的なガイドです。各トピックごとに整理されており、実装からトラブルシューティングまでをカバーしています。

## 📚 ドキュメント構成

### 1. [認証とOAuth2](./01-authentication.md)

- OAuth2フローの実装
- トークン管理とリフレッシュ
- 環境別の設定方法
- AWS環境での設定

### 2. [リード管理](./02-lead-management.md)

- リード作成APIの使用方法
- フォーム送信からのリード生成
- カスタムフィールドの設定
- 権限とアクセス制御

### 3. [開発環境セットアップ](./03-development-setup.md)

- ローカル環境の構築
- Docker環境での開発
- 必要な環境変数
- Salesforceアプリの設定

### 4. [テストツールとスクリプト](./04-testing-tools.md)

- フォーム送信テストスクリプト
- OAuth2フローテストツール
- デバッグ用ユーティリティ
- 自動テストの実行方法

### 5. [AWS環境デプロイ](./05-aws-deployment.md)

- Secrets Managerの設定
- RDSへのデータ投入
- 環境変数の管理
- CodePipelineでのデプロイ

### 6. [トラブルシューティング](./06-troubleshooting.md)

- よくあるエラーと解決方法
- OAuth2認証の問題
- API権限エラー
- ログの確認方法

### 7. [アーキテクチャ決定記録](./07-architecture-decisions.md)

- ADR-001: OAuth2実装方針
- 技術選定の理由
- 今後の拡張計画

## 🚀 クイックスタート

### 初回セットアップ

1. **Salesforceアプリの作成**

   ```bash
   # ドキュメントを参照
   cat docs/salesforce/03-development-setup.md
   ```

2. **環境変数の設定**

   ```bash
   # .envファイルの設定
   cp .env.example .env
   # 必要な値を設定
   ```

3. **OAuth2認証の実行**

   ```bash
   # 認証URLの生成とコールバック処理
   python scripts/salesforce-integration/testing/oauth2_flow.py
   ```

4. **フォーム送信テスト**
   ```bash
   # テストフォームからリード作成
   python scripts/salesforce-integration/testing/submit_form_test_dev.py
   ```

## 📁 スクリプトディレクトリ構成

```
scripts/salesforce-integration/
├── README.md                    # スクリプト全体の説明
├── aws-deployment/              # AWS環境用スクリプト
│   ├── create_salesforce_secret.sh
│   ├── insert_crm_integration_rds.sh
│   └── update_aws_secrets.sh
├── form-management/             # フォーム管理スクリプト
│   ├── create_markmail_form.py
│   └── publish_form.py
├── testing/                     # テスト用スクリプト
│   ├── oauth2_flow.py
│   ├── submit_form_test.py
│   └── test_oauth_curl.sh
└── utilities/                   # ユーティリティスクリプト
    ├── check_field_permissions.py
    └── check_picklist_values.py
```

## 🔗 関連リソース

- [Salesforce REST API Documentation](https://developer.salesforce.com/docs/atlas.en-us.api_rest.meta/api_rest/)
- [OAuth 2.0 Web Server Flow](https://help.salesforce.com/s/articleView?id=sf.remoteaccess_oauth_web_server_flow.htm)
- [MarkMail Backend CRM Module](../../backend/src/crm/)

## 📝 更新履歴

- 2025-08-03: ドキュメント構成の整理と統合
- 2025-07-30: AWS環境での実装完了
- 2025-07-28: 初回実装とドキュメント作成
