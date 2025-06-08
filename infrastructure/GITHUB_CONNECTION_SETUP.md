# GitHub Connection セットアップガイド

## エラーの原因

CodePipelineのソースステージで以下のエラーが発生しています：
```
Connection markmail-dev-github is not available
```

これは、AWS CodeConnections（旧CodeStar Connections）のGitHub接続が承認されていないことが原因です。

## 解決手順

### 1. AWS コンソールにログイン

```bash
# AWS SSOでログイン（既に設定済みの場合）
aws sso login --profile markmail-dev
```

### 2. CodeConnectionsで接続を承認

1. [AWS Console](https://console.aws.amazon.com/) にアクセス
2. リージョンが `ap-northeast-1` (東京) になっていることを確認
3. サービス検索で「Developer Tools」または「CodeConnections」を検索
4. 左メニューから「Connections」を選択
5. `markmail-dev-github` という名前の接続を見つける
   - ステータスが「PENDING」になっているはず
6. 接続名をクリックして詳細ページへ
7. 「Update pending connection」ボタンをクリック
8. GitHubの認証画面が開くので、以下を実行：
   - GitHubにログイン
   - AWS Connector GitHub Appをインストール（初回のみ）
   - リポジトリへのアクセスを許可
     - Organization: `engineers-hub-ltd-in-house-project`
     - Repository: `markmail`
9. 認証が完了すると、ステータスが「AVAILABLE」に変わる

### 3. CodePipelineの再実行

接続が承認されたら、CodePipelineを再実行します：

1. AWS Console で「CodePipeline」を検索
2. `markmail-dev-pipeline` を選択
3. 「Release change」ボタンをクリックして手動実行

### 4. 動作確認

パイプラインが正常に動作することを確認：
- Sourceステージ: GitHubからコード取得
- Buildステージ: Dockerイメージのビルド
- Deployステージ: ECSへのデプロイ

## トラブルシューティング

### 接続が見つからない場合

```bash
# CI/CDスタックのデプロイ状況確認
aws cloudformation describe-stacks \
  --stack-name MarkMail-dev-CICDStack \
  --profile markmail-dev \
  --query 'Stacks[0].Outputs[?OutputKey==`GitHubConnectionArn`].OutputValue' \
  --output text
```

### 権限エラーの場合

IAMロールに以下の権限が必要です：
- `codeconnections:UseConnection`
- `codeconnections:GetConnection`

### 再デプロイが必要な場合

```bash
cd infrastructure
npm run cdk deploy MarkMail-dev-CICDStack -- --profile markmail-dev
```

## 参考情報

- [AWS CodeConnections ドキュメント](https://docs.aws.amazon.com/codestar-connections/latest/userguide/connections.html)
- [GitHub App のインストール](https://docs.aws.amazon.com/codestar-connections/latest/userguide/update-github-connection.html)