# Salesforce Integration Scripts

このディレクトリには、MarkMailとSalesforceの統合に関するユーティリティスクリプトが含まれています。

## スクリプト一覧

### 1. create_markmail_form.py

エンジニアスキルシートフォームをMarkMail APIで作成するスクリプト

**使用方法:**

```bash
python3 create_markmail_form.py
```

**必要な設定:**

- MarkMail APIエンドポイント (デフォルト: http://localhost:3000/api)
- ログイン情報を環境変数またはスクリプト内で設定

### 2. publish_form.py

作成したフォームを公開状態に変更するスクリプト

**使用方法:**

```bash
python3 publish_form.py
```

### 3. check_field_permissions.py

Salesforceのカスタムフィールドに対するフィールドレベルセキュリティを確認

**使用方法:**

```bash
python3 check_field_permissions.py
```

**前提条件:**

- Salesforce CLIがインストールされていること
- `sf org list` でmarkmail-orgが設定されていること

### 4. check_picklist_values.py

Salesforceの選択リストフィールドの値を確認

**使用方法:**

```bash
python3 check_picklist_values.py
```

### 5. submit_form_test.py

テストデータを使用してフォーム送信をテスト

**使用方法:**

```bash
python3 submit_form_test.py
```

### 6. update_sf_token_docker.py

Salesforceのアクセストークンを更新（Docker経由）

**使用方法:**

```bash
# 最新のトークンを取得
sf org display -o markmail-org --json | jq -r '.result.accessToken'

# スクリプトでトークンを更新
python3 update_sf_token_docker.py
```

## 注意事項

- これらのスクリプトにはセンシティブな情報（APIキー、パスワード等）を含めないでください
- 実行前に必要な環境変数を設定してください
- Salesforce CLIの認証が有効であることを確認してください
