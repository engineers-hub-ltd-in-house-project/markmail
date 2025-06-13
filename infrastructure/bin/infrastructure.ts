#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { NetworkStack } from '../lib/stacks/network-stack';
import { DatabaseStack } from '../lib/stacks/database-stack';
import { ECRStack } from '../lib/stacks/ecr-stack';
import { ECSClusterStack } from '../lib/stacks/ecs-cluster-stack';
import { Route53Stack } from '../lib/stacks/route53-stack';
import { ALBStack } from '../lib/stacks/alb-stack';
import { ECSServiceStack } from '../lib/stacks/ecs-service-stack';
import { CICDStack } from '../lib/stacks/cicd-stack';
import { MonitoringStack } from '../lib/stacks/monitoring-stack';
import { BastionStack } from '../lib/stacks/bastion-stack';

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
    domainName: process.env.DEV_DOMAIN || process.env.PROD_DOMAIN,
    notificationEmail: process.env.NOTIFICATION_EMAIL || 'admin@example.com',
    githubOwner: process.env.GITHUB_OWNER || 'engineers-hub-ltd-in-house-project',
    githubRepo: process.env.GITHUB_REPO || 'markmail',
    githubBranch: process.env.GITHUB_BRANCH || 'dev',
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

// Stack 1: Network Infrastructure
const networkStack = new NetworkStack(app, `MarkMail-${environmentName}-NetworkStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  description: `MarkMail Network Infrastructure Stack for ${environmentName} environment`,
});

// Stack 2: Database Infrastructure (depends on Network)
const databaseStack = new DatabaseStack(app, `MarkMail-${environmentName}-DatabaseStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  vpc: networkStack.vpc,
  rdsSecurityGroup: networkStack.rdsSecurityGroup,
  cacheSecurityGroup: networkStack.cacheSecurityGroup,
  dbInstanceClass: (config as any).dbInstanceClass,
  dbInstanceSize: (config as any).dbInstanceSize,
  description: `MarkMail Database Infrastructure Stack for ${environmentName} environment`,
});
databaseStack.addDependency(networkStack);

// Stack 3: ECR Repositories
const ecrStack = new ECRStack(app, `MarkMail-${environmentName}-ECRStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  description: `MarkMail ECR Repositories Stack for ${environmentName} environment`,
});

// Stack 4: ECS Cluster (depends on Network and Database for secrets)
const ecsClusterStack = new ECSClusterStack(app, `MarkMail-${environmentName}-ECSClusterStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  vpc: networkStack.vpc,
  dbSecret: databaseStack.dbSecret,
  description: `MarkMail ECS Cluster Stack for ${environmentName} environment`,
});
ecsClusterStack.addDependency(networkStack);
ecsClusterStack.addDependency(databaseStack);

// Stack 4.5: Route 53 (optional, only if domain is configured)
let route53Stack: Route53Stack | undefined;
if (config.domainName) {
  route53Stack = new Route53Stack(app, `MarkMail-${environmentName}-Route53Stack`, {
    env: { account, region },
    environmentName: config.environmentName,
    domainName: config.domainName,
    description: `MarkMail Route 53 Stack for ${environmentName} environment`,
  });
}

// Stack 5: Application Load Balancer (depends on Network and optionally Route53)
const albStack = new ALBStack(app, `MarkMail-${environmentName}-ALBStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  vpc: networkStack.vpc,
  albSecurityGroup: networkStack.albSecurityGroup,
  domainName: config.domainName,
  hostedZone: route53Stack?.hostedZone,
  description: `MarkMail Application Load Balancer Stack for ${environmentName} environment`,
});
albStack.addDependency(networkStack);
if (route53Stack) {
  albStack.addDependency(route53Stack);
}

// Stack 6: ECS Service (depends on all previous stacks)
const ecsServiceStack = new ECSServiceStack(app, `MarkMail-${environmentName}-ECSServiceStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  vpc: networkStack.vpc,
  ecsSecurityGroup: networkStack.ecsSecurityGroup,
  cluster: ecsClusterStack.cluster,
  taskExecutionRole: ecsClusterStack.taskExecutionRole,
  taskRole: ecsClusterStack.taskRole,
  logGroup: ecsClusterStack.logGroup,
  backendRepo: ecrStack.backendRepo,
  frontendRepo: ecrStack.frontendRepo,
  database: databaseStack.database,
  dbSecret: databaseStack.dbSecret,
  cacheCluster: databaseStack.cacheCluster,
  loadBalancer: albStack.loadBalancer,
  httpsListener: albStack.httpsListener,
  httpListener: albStack.httpListener,
  desiredCount: config.desiredCount,
  cpu: config.cpu,
  memoryLimitMiB: config.memoryLimitMiB,
  description: `MarkMail ECS Service Stack for ${environmentName} environment`,
});
ecsServiceStack.addDependency(ecrStack);
ecsServiceStack.addDependency(ecsClusterStack);
ecsServiceStack.addDependency(albStack);

// Stack 7: CI/CD Pipeline (depends on ECR and ECS Service)
const cicdStack = new CICDStack(app, `MarkMail-${environmentName}-CICDStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  githubOwner: config.githubOwner,
  githubRepo: config.githubRepo,
  githubBranch: config.githubBranch,
  backendRepo: ecrStack.backendRepo,
  frontendRepo: ecrStack.frontendRepo,
  backendService: ecsServiceStack.backendService,
  frontendService: ecsServiceStack.frontendService,
  domainName: config.domainName,
  description: `MarkMail CI/CD Pipeline Stack for ${environmentName} environment`,
});
cicdStack.addDependency(ecrStack);
cicdStack.addDependency(ecsServiceStack);

// Stack 8: Monitoring (depends on ALB and ECS Service)
const monitoringStack = new MonitoringStack(app, `MarkMail-${environmentName}-MonitoringStack`, {
  env: { account, region },
  environmentName: config.environmentName,
  notificationEmail: config.notificationEmail,
  backendService: ecsServiceStack.backendService,
  frontendService: ecsServiceStack.frontendService,
  loadBalancer: albStack.loadBalancer,
  domainName: config.domainName,
  description: `MarkMail Monitoring Stack for ${environmentName} environment`,
});
monitoringStack.addDependency(albStack);
monitoringStack.addDependency(ecsServiceStack);

// Bastion Stack (独立したスタック - 依存関係なし)
// 環境変数 CREATE_BASTION=true の場合のみ作成
if (process.env.CREATE_BASTION === 'true') {
  const bastionStack = new BastionStack(app, `MarkMail-${environmentName}-BastionStack`, {
    stackName: `MarkMail-${environmentName}-BastionStack`,
    env: { account, region },
    environmentName,
    description: `MarkMail Bastion Host Stack for ${environmentName} environment (TEMPORARY)`,
  });
  // 依存関係を明示的に追加しない - 独立したスタックとして扱う
  console.log(`\n[TEMPORARY] Bastion Stack: ${bastionStack.stackName}`);
}

// Output stack deployment order
console.log(`\n Stack deployment order for ${environmentName} environment:`);
console.log(`1. ${networkStack.stackName} - Network infrastructure (VPC, Security Groups)`);
console.log(`2. ${databaseStack.stackName} - Database infrastructure (RDS, ElastiCache)`);
console.log(`3. ${ecrStack.stackName} - ECR repositories`);
console.log(`4. ${ecsClusterStack.stackName} - ECS Cluster and IAM roles`);
if (route53Stack) {
  console.log(`5. ${route53Stack.stackName} - Route 53 Hosted Zone`);
  console.log(`6. ${albStack.stackName} - Application Load Balancer with SSL`);
} else {
  console.log(`5. ${albStack.stackName} - Application Load Balancer`);
}
console.log(
  `${route53Stack ? '7' : '6'}. ${ecsServiceStack.stackName} - ECS Service and Task Definition`
);
console.log(`${route53Stack ? '8' : '7'}. ${cicdStack.stackName} - CI/CD Pipeline`);
console.log(
  `${route53Stack ? '9' : '8'}. ${monitoringStack.stackName} - CloudWatch and SES monitoring\n`
);
