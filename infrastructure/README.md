# MarkMail Infrastructure (AWS CDK)

このディレクトリには、MarkMailのAWSインフラストラクチャをCDKで管理するコードが含まれています。

## 🎯 概要

AWS CDKを使用して以下のリソースを作成・管理します：

- **AWS SES** - メール送信サービス
  - Configuration Set（送信設定）
  - Email Identity（ドメイン/メールアドレス検証）
  - DKIM設定
- **Amazon SNS** - 通知サービス
  - バウンス通知トピック
  - 苦情通知トピック
- **Amazon S3** - ストレージ
  - バウンスメールの保存（オプション）
- **IAM** - アクセス管理
  - SES送信用ユーザー
  - 必要な権限ポリシー

## 📋 前提条件

1. **AWS CLI**がインストールされ、認証情報が設定されていること

   ```bash
   aws configure
   ```

2. **Node.js** (v18以上) と **npm**がインストールされていること

3. **AWS CDK**がグローバルにインストールされていること
   ```bash
   npm install -g aws-cdk
   ```

## 🚀 デプロイ手順

### 1. 依存関係のインストール

```bash
cd infrastructure
npm install
```

### 2. 環境変数の設定（オプション）

```bash
# カスタムドメインを使用する場合
export SES_DOMAIN=mail.example.com

# 通知メールアドレス
export NOTIFICATION_EMAIL=admin@example.com

# AWSアカウントとリージョン
export AWS_ACCOUNT_ID=123456789012
export AWS_REGION=ap-northeast-1
```

### 3. CDKのブートストラップ（初回のみ）

```bash
npm run cdk bootstrap
```

### 4. スタックのデプロイ

```bash
# 差分確認
npm run diff

# デプロイ
npm run deploy
```

### 5. デプロイ後の設定

デプロイが完了したら、以下の手順を実行してください：

1. **ドメイン/メールアドレスの検証**

   - AWS SESコンソールで送信元ドメインまたはメールアドレスを検証

2. **本番環境アクセスのリクエスト**（必要な場合）

   - SESサンドボックスから本番環境への移行申請

3. **アプリケーション設定の更新**

   ```bash
   # backend/.env に以下を追加
   AWS_ACCESS_KEY_ID=<出力されたアクセスキーID>
   AWS_SECRET_ACCESS_KEY=<出力されたシークレットアクセスキー>
   AWS_REGION=ap-northeast-1
   AWS_SES_FROM_EMAIL=noreply@example.com
   AWS_SES_CONFIGURATION_SET=markmail-configuration-set
   ```

4. **DNS設定**（カスタムドメインの場合）
   - 出力されたDKIMレコードをDNSに追加

## 📊 作成されるリソース

### Configuration Set

- 名前: `markmail-configuration-set`
- イベント追跡: 送信、バウンス、苦情、配信、拒否など
- CloudWatchメトリクス: 有効

### SNSトピック

- バウンス通知: `markmail-bounce-notifications`
- 苦情通知: `markmail-complaint-notifications`

### S3バケット

- 名前: `markmail-emails-{account}-{region}`
- 用途: バウンスメールの保存
- ライフサイクル: 90日後に自動削除

### IAMユーザー

- 名前: `markmail-ses-user`
- 権限:
  - SES送信権限
  - Configuration Set管理権限
  - SNSトピック読み取り権限

## 🔧 カスタマイズ

`lib/infrastructure-stack.ts`を編集して、以下をカスタマイズできます：

- リソース名
- S3バケットの保存期間
- SNS通知の設定
- IAM権限の範囲

## 🗑️ リソースの削除

```bash
npm run destroy
```

⚠️ **注意**:
S3バケットは`RETAIN`ポリシーが設定されているため、手動で削除する必要があります。

## 📝 コマンド一覧

```bash
# TypeScriptのビルド
npm run build

# CDKスタックの合成（CloudFormationテンプレート生成）
npm run synth

# 差分確認
npm run diff

# デプロイ
npm run deploy

# 削除
npm run destroy

# CDKコマンドの直接実行
npm run cdk -- <command>
```

## 🔍 トラブルシューティング

### "Stack is in DELETE_FAILED state"

```bash
# CloudFormationコンソールで手動削除するか
aws cloudformation delete-stack --stack-name MarkMailInfrastructureStack
```

### "Bootstrap stack required"

```bash
npm run cdk bootstrap aws://<account-id>/<region>
```

### アクセスキーが表示されない

CloudFormationのOutputsタブまたは以下のコマンドで確認：

```bash
aws cloudformation describe-stacks \
  --stack-name MarkMailInfrastructureStack \
  --query "Stacks[0].Outputs"
```
