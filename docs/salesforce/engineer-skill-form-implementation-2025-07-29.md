# エンジニアスキルシートフォーム実装記録

## 実施日時

2025年7月29日

## 概要

HTMLのエンジニアスキルシートフォームと同様の機能をMarkMailで実装し、フォーム送信時に自動的にSalesforceリードを作成する仕組みを構築しました。

## 実装内容

### 1. バックエンド拡張

#### カスタムフィールドマッピング定数の追加

`backend/src/models/crm.rs` に以下を追加：

```rust
// Salesforce Lead カスタムフィールドマッピング定数
pub mod salesforce_field_mapping {
    // プログラミング言語スキルフィールドマッピング
    pub const PROGRAMMING_LANGUAGE_FIELDS: &[(&str, &str)] = &[
        ("java", "00NIR00000FTrIJ"),
        ("python", "00NIR00000FTrIO"),
        ("javascript_typescript", "00NIR00000FTrIT"),
        ("c_cpp", "00NIR00000FTrIY"),
        ("csharp", "00NIR00000FTrId"),
        ("php", "00NIR00000FTrIi"),
        ("go", "00NIR00000FTrIn"),
        ("ruby", "00NIR00000FTrIs"),
        ("swift", "00NIR00000FTrIx"),
        ("kotlin", "00NIR00000FTrJ2"),
    ];

    // フレームワーク・技術スタックフィールドマッピング
    pub const TECH_STACK_FIELDS: &[(&str, &str)] = &[
        ("react", "00NIR00000FTrNZ"),
        ("nextjs", "00NIR00000FTrNe"),
        // ... 他のフィールド
    ];

    // その他のカスタムフィールドマッピング
    pub const OTHER_FIELDS: &[(&str, &str)] = &[
        ("github_url", "00NIR00000FTrJC"),
        ("portfolio_url", "00NIR00000FTrJH"),
        ("experience", "00NIR00000FTrJM"),
        ("newsletter_opt_in", "00NIR00000FTrVO"),
        ("state", "State"), // 標準フィールド
    ];
}
```

#### CrmLead::from_form_submission メソッドの拡張

カスタムフィールドのマッピング処理を追加（backend/src/models/crm.rs:266-300）

#### Salesforceプロバイダーの拡張

`backend/src/services/crm_service/salesforce.rs` に以下を追加：

- `build_lead_params` メソッド：カスタムフィールドを含むパラメータを構築
- `create_lead` メソッドの更新：カスタムフィールドに対応

### 2. フォーム作成

#### フォーム構成

作成したフォームID: `92c55a20-85cf-4418-a127-5ccdf39c4c0f`

**基本情報フィールド:**

- 姓 (last_name) - text, required
- 名 (first_name) - text, required
- メール (email) - email, required
- 会社名 (company) - text
- 都道府県 (state) - text

**プログラミング言語スキル (select):**

- 10言語（Java, Python, JavaScript/TypeScript等）
- 選択肢：未経験、基礎知識、実務経験あり、実務経験豊富（メイン言語として使用）、専門性が高い（技術選定・指導可能）

**技術スタック (checkbox):**

- 27種類の技術（React, AWS, Docker等）

**その他:**

- 業務経験・アピールポイント (experience) - textarea
- GitHub URL (github_url) - url
- ポートフォリオURL (portfolio_url) - url
- お知らせ受信 (newsletter_opt_in) - checkbox

### 3. Salesforce設定確認

すべてのカスタムフィールドに適切なフィールドレベルセキュリティが設定されていることを確認しました。

## 発生した問題と解決方法

### 1. Salesforceセッショントークンの期限切れ

**エラー**: `INVALID_SESSION_ID`

**解決方法**:

```bash
# 新しいトークンを取得
sf org display -o markmail-org --json | jq -r '.result.accessToken'

# データベースを更新（Dockerコンテナ経由）
docker exec markmail-postgres-1 psql -U markmail -d markmail_dev -c "UPDATE ..."
```

### 2. 選択リストの値の不一致

**エラー**: `INVALID_OR_NULL_FOR_RESTRICTED_PICKLIST`

**原因**: 送信した値「実務経験豊富」と、Salesforceの選択リスト値「実務経験豊富（メイン言語として使用）」が完全一致していなかった

**解決方法**: 正確な選択リスト値を使用するように修正

## 作成したツール・スクリプト

### 1. フォーム作成スクリプト

`scripts/salesforce-integration/create_markmail_form.py`

- MarkMail APIを使用してエンジニアスキルシートフォームを作成

### 2. フォーム公開スクリプト

`scripts/salesforce-integration/publish_form.py`

- 作成したフォームをpublishedステータスに更新

### 3. フィールド権限確認スクリプト

`scripts/salesforce-integration/check_field_permissions.py`

- Salesforceのカスタムフィールドに対するフィールドレベルセキュリティを確認

### 4. 選択リスト値確認スクリプト

`scripts/salesforce-integration/check_picklist_values.py`

- プログラミング言語フィールドの選択リスト値を確認

### 5. フォーム送信テストスクリプト

`scripts/salesforce-integration/submit_form_test.py`

- API経由でテストデータを送信してフォームの動作を確認

### 6. Salesforceトークン更新スクリプト

`scripts/salesforce-integration/update_sf_token_docker.py`

- Dockerコンテナ経由でSalesforceのアクセストークンを更新

## 動作確認結果

テストフォーム送信により、以下が正常に動作することを確認：

1. MarkMailでフォーム送信を受信
2. 購読者として登録
3. Salesforceリードを自動作成（すべてのカスタムフィールドも含む）

**作成されたリードの例:**

- ID: 00QIR00000dPmzS2AS
- 名前: 田中 太郎
- 会社: テスト株式会社
- Java: 実務経験豊富（メイン言語として使用）
- Python: 専門性が高い（技術選定・指導可能）

## 今後の注意事項

1. **セッショントークンの管理**:
   Salesforceのトークンは定期的に期限切れになるため、更新が必要
2. **選択リストの値**: Salesforceの選択リスト値と完全一致する必要がある
3. **カスタムフィールドの権限**: 新しいカスタムフィールドを追加する場合は、フィールドレベルセキュリティの設定が必要

## 関連ファイル

- `/backend/src/models/crm.rs` - カスタムフィールドマッピング定義
- `/backend/src/services/crm_service/salesforce.rs` - Salesforceプロバイダー実装
- `/scripts/salesforce-integration/` - 各種ユーティリティスクリプト

## 公開フォームURL

http://localhost:5173/forms/92c55a20-85cf-4418-a127-5ccdf39c4c0f/public
