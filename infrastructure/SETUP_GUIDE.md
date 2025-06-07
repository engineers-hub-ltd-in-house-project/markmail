# AWS SSO セットアップ & デプロイ手順

## 1. AWS SSO プロファイル設定

```bash
cd infrastructure
./setup-aws-profile.sh
```

スクリプトが `aws configure sso` ウィザードを起動します。以下の情報を入力:

1. **SSO session name**: 任意の名前 (例: markmail-sso)
2. **SSO start URL**: https://your-sso-portal.awsapps.com/start
3. **SSO region**: us-east-1 (通常)
4. **SSO registration scopes**: デフォルト (Enterキー)
5. ブラウザが開いたらSSOポータルにログイン
6. AWSアカウントとIAMロールを選択
7. **CLI default client Region**: ap-northeast-1
8. **CLI default output format**: json
9. **CLI profile name**: 最初に入力したプロファイル名が表示される

## 2. デプロイ実行

```bash
# プロファイルを環境変数に設定
export AWS_PROFILE=markmail-dev

# デプロイスクリプト実行
./deploy.sh
```

## 3. デプロイ後の設定

### Backend設定の更新

デプロイ完了後、表示されたアクセスキーを `backend/.env` に追加:

```env
# AWS SES設定（本番環境用）
EMAIL_PROVIDER=aws_ses
AWS_REGION=ap-northeast-1
AWS_ACCESS_KEY_ID=<表示されたアクセスキーID>
AWS_SECRET_ACCESS_KEY=<シークレットアクセスキーはAWSコンソールで取得>
AWS_SES_FROM_EMAIL=noreply@example.com
AWS_SES_CONFIGURATION_SET=markmail-configuration-set
```

### メールアドレス検証

1. [AWS SESコンソール](https://console.aws.amazon.com/ses/)にアクセス
2. 「Verified identities」を選択
3. 「Create identity」をクリック
4. 送信元メールアドレスを追加して検証

## 4. 動作確認

```bash
cd ../backend
cargo run
```

フロントエンドでキャンペーン送信を実行し、AWS
SESでメールが送信されることを確認。
