# Squarespace サブドメイン委譲ガイド

このガイドでは、Squarespaceで管理している `engineers-hub.ltd` ドメインから、`dev.markmail.engineers-hub.ltd` サブドメインをAWS Route 53に委譲する手順を説明します。

## 手順

### 1. Route 53 ホストゾーンのデプロイ

```bash
cd infrastructure

# 環境変数を設定（開発環境）
source ./setup-dev-domain.sh

# Route 53スタックのみをデプロイ
npx cdk deploy MarkMail-dev-Route53Stack --profile yusuke.sato
```

### 2. ネームサーバーの取得

デプロイ完了後、CloudFormationの出力からネームサーバーを確認：

```bash
aws cloudformation describe-stacks \
  --stack-name MarkMail-dev-Route53Stack \
  --query "Stacks[0].Outputs[?OutputKey=='NameServers'].OutputValue" \
  --output text \
  --profile yusuke.sato
```

または、AWSコンソールで確認：

1. CloudFormation → MarkMail-dev-Route53Stack
2. 「出力」タブ
3. 「NameServers」の値をコピー（カンマ区切りで4つのネームサーバー）

### 3. Squarespaceでの設定

1. Squarespaceにログイン
2. 「Settings」→「Domains」→「engineers-hub.ltd」を選択
3. 「Advanced」または「DNS Settings」をクリック
4. 「Add Custom Record」または「Add Record」をクリック
5. 以下のNSレコードを追加（4つ全て）：

```
Record Type: NS
Host: dev.markmail
Value: ns-XXX.awsdns-XX.org.
```

**注意事項:**

- 各ネームサーバーごとに個別のレコードを作成
- 末尾のドット（.）を含める
- TTLは3600（1時間）を推奨

### 4. DNS伝播の確認

設定後、DNS伝播を確認（通常5-30分かかります）：

```bash
# ネームサーバーが正しく設定されているか確認
dig NS dev.markmail.engineers-hub.ltd

# 期待される結果：Route 53のネームサーバーが表示される
```

### 5. 残りのスタックのデプロイ

DNS委譲が確認できたら、残りのスタックをデプロイ：

```bash
# ALBスタック（SSL証明書が自動作成されます）
npx cdk deploy MarkMail-dev-ALBStack --profile yusuke.sato

# または全スタックを順次デプロイ
./deploy-sequential.sh
```

## トラブルシューティング

### NSレコードが反映されない場合

1. Squarespaceの設定を再確認
2. 親ドメイン（engineers-hub.ltd）のネームサーバーが正しいか確認
3. DNS伝播に最大48時間かかる場合があります

### 証明書の検証が失敗する場合

ACMは自動的にDNS検証用のCNAMEレコードを作成します。
これらのレコードは自動的にRoute 53に追加されるため、通常は手動操作は不要です。

### 確認コマンド

```bash
# サブドメインのネームサーバー確認
dig NS dev.markmail.engineers-hub.ltd @8.8.8.8

# Route 53ホストゾーンの詳細確認
aws route53 get-hosted-zone \
  --id $(aws route53 list-hosted-zones-by-name \
    --dns-name dev.markmail.engineers-hub.ltd \
    --query "HostedZones[0].Id" \
    --output text) \
  --profile yusuke.sato
```
