#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { MarkMailInfrastructureStack } from '../lib/infrastructure-stack';

const app = new cdk.App();

// 環境変数から設定を取得
const environmentName = app.node.tryGetContext('environment') || 'dev';
const account = app.node.tryGetContext('account') || process.env.CDK_DEFAULT_ACCOUNT;
const region =
  app.node.tryGetContext('region') || process.env.CDK_DEFAULT_REGION || 'ap-northeast-1';

// 環境別の設定
const config = {
  dev: {
    environmentName: 'dev',
    domainName: undefined, // 開発環境ではドメインなし
    notificationEmail: process.env.NOTIFICATION_EMAIL || 'admin@example.com',
    githubOwner: process.env.GITHUB_OWNER || 'engineers-hub-ltd-in-house-project',
    githubRepo: process.env.GITHUB_REPO || 'markmail',
    githubBranch: 'develop',
    desiredCount: 1,
    cpu: 512,
    memoryLimitMiB: 1024,
  },
  staging: {
    environmentName: 'staging',
    domainName: process.env.STAGING_DOMAIN,
    notificationEmail: process.env.NOTIFICATION_EMAIL || 'admin@example.com',
    githubOwner: process.env.GITHUB_OWNER || 'engineers-hub-ltd-in-house-project',
    githubRepo: process.env.GITHUB_REPO || 'markmail',
    githubBranch: 'staging',
    desiredCount: 2,
    cpu: 512,
    memoryLimitMiB: 1024,
  },
  prod: {
    environmentName: 'prod',
    domainName: process.env.PROD_DOMAIN,
    notificationEmail: process.env.NOTIFICATION_EMAIL || 'admin@example.com',
    githubOwner: process.env.GITHUB_OWNER || 'engineers-hub-ltd-in-house-project',
    githubRepo: process.env.GITHUB_REPO || 'markmail',
    githubBranch: 'main',
    desiredCount: 3,
    cpu: 1024,
    memoryLimitMiB: 2048,
    dbInstanceClass: cdk.aws_ec2.InstanceClass.T3,
    dbInstanceSize: cdk.aws_ec2.InstanceSize.SMALL,
  },
}[environmentName as 'dev' | 'staging' | 'prod'];

if (!config) {
  throw new Error(`Unknown environment: ${environmentName}. Use 'dev', 'staging', or 'prod'.`);
}

new MarkMailInfrastructureStack(app, `MarkMail-${environmentName}-Stack`, {
  env: { account, region },
  ...config,
});
