#!/bin/bash

# 自動デプロイ用スクリプト（yes応答付き）
set -e

export AWS_PROFILE=yusuke.sato
export NOTIFICATION_EMAIL="yusuke.sato@engineers-hub.ltd"
export GITHUB_OWNER="engineers-hub-ltd-in-house-project"
export GITHUB_REPO="markmail"

cd /home/yusuke/engineers-hub.ltd/in-house-project/markmail/infrastructure

# デプロイ実行
npx cdk deploy -c environment=dev --require-approval never