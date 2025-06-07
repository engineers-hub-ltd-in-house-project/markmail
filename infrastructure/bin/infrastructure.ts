#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { MarkMailInfrastructureStack } from '../lib/infrastructure-stack';

const app = new cdk.App();

// 環境変数から設定を取得
const env = {
  account: process.env.CDK_DEFAULT_ACCOUNT || process.env.AWS_ACCOUNT_ID,
  region: process.env.CDK_DEFAULT_REGION || process.env.AWS_REGION || 'ap-northeast-1',
};

// スタックの作成
new MarkMailInfrastructureStack(app, 'MarkMailInfrastructureStack', {
  env,
  description: 'MarkMail Email Infrastructure - SES configuration and resources',

  // スタックのタグ
  tags: {
    Project: 'MarkMail',
    Environment: process.env.ENVIRONMENT || 'development',
    ManagedBy: 'CDK',
  },
});
