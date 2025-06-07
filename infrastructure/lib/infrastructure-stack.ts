import * as cdk from 'aws-cdk-lib';
import type { Construct } from 'constructs';

// Import our modular constructs
import { NetworkConstruct } from './constructs/network';
import { DatabaseConstruct } from './constructs/database';
import { ContainerConstruct } from './constructs/container';
import { CICDConstruct } from './constructs/cicd';
import { MonitoringConstruct } from './constructs/monitoring';

export interface MarkMailInfrastructureStackProps extends cdk.StackProps {
  // 環境名 (dev, staging, prod)
  environmentName: string;
  // ドメイン名
  domainName?: string;
  // 通知用メールアドレス
  notificationEmail: string;
  // GitHubリポジトリ情報
  githubOwner: string;
  githubRepo: string;
  githubBranch: string;
  // データベース設定
  dbInstanceClass?: cdk.aws_ec2.InstanceClass;
  dbInstanceSize?: cdk.aws_ec2.InstanceSize;
  // ECS設定
  desiredCount?: number;
  cpu?: number;
  memoryLimitMiB?: number;
}

export class MarkMailInfrastructureStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: MarkMailInfrastructureStackProps) {
    super(scope, id, props);

    const {
      environmentName,
      domainName,
      notificationEmail,
      githubOwner,
      githubRepo,
      githubBranch,
      dbInstanceClass,
      dbInstanceSize,
      desiredCount,
      cpu,
      memoryLimitMiB,
    } = props;

    // Network Layer
    const network = new NetworkConstruct(this, 'Network', {
      environmentName,
    });

    // Database Layer
    const database = new DatabaseConstruct(this, 'Database', {
      environmentName,
      vpc: network.vpc,
      rdsSecurityGroup: network.rdsSecurityGroup,
      cacheSecurityGroup: network.cacheSecurityGroup,
      dbInstanceClass,
      dbInstanceSize,
    });

    // Container Layer
    const container = new ContainerConstruct(this, 'Container', {
      environmentName,
      vpc: network.vpc,
      ecsSecurityGroup: network.ecsSecurityGroup,
      database: database.database,
      dbSecret: database.dbSecret,
      cacheCluster: database.cacheCluster,
      domainName,
      desiredCount,
      cpu,
      memoryLimitMiB,
    });

    // CI/CD Layer
    const cicd = new CICDConstruct(this, 'CICD', {
      environmentName,
      githubOwner,
      githubRepo,
      githubBranch,
      backendRepo: container.backendRepo,
      frontendRepo: container.frontendRepo,
      fargateService: container.alb,
      domainName,
    });

    // Monitoring & Security Layer
    const monitoring = new MonitoringConstruct(this, 'Monitoring', {
      environmentName,
      notificationEmail,
      service: container.alb.service,
      loadBalancer: container.alb.loadBalancer,
      domainName,
    });

    // Outputs
    new cdk.CfnOutput(this, 'ALBEndpoint', {
      value: container.alb.loadBalancer.loadBalancerDnsName,
      description: 'ALB endpoint URL',
      exportName: `${this.stackName}-ALBEndpoint`,
    });

    new cdk.CfnOutput(this, 'DatabaseEndpoint', {
      value: database.database.dbInstanceEndpointAddress,
      description: 'RDS endpoint',
      exportName: `${this.stackName}-DatabaseEndpoint`,
    });

    new cdk.CfnOutput(this, 'CacheEndpoint', {
      value: database.cacheCluster.attrRedisEndpointAddress,
      description: 'ElastiCache endpoint',
      exportName: `${this.stackName}-CacheEndpoint`,
    });

    new cdk.CfnOutput(this, 'BackendRepositoryUri', {
      value: container.backendRepo.repositoryUri,
      description: 'Backend ECR repository URI',
      exportName: `${this.stackName}-BackendRepoUri`,
    });

    new cdk.CfnOutput(this, 'FrontendRepositoryUri', {
      value: container.frontendRepo.repositoryUri,
      description: 'Frontend ECR repository URI',
      exportName: `${this.stackName}-FrontendRepoUri`,
    });

    new cdk.CfnOutput(this, 'SESConfigurationSetName', {
      value: monitoring.sesConfigurationSet.configurationSetName!,
      description: 'SES Configuration Set Name',
      exportName: `${this.stackName}-SESConfigSet`,
    });

    new cdk.CfnOutput(this, 'PipelineName', {
      value: cicd.pipeline.pipelineName,
      description: 'CodePipeline name',
      exportName: `${this.stackName}-PipelineName`,
    });

    new cdk.CfnOutput(this, 'GitHubConnectionArn', {
      value: cicd.githubConnection.attrConnectionArn,
      description: 'GitHub CodeConnection ARN',
      exportName: `${this.stackName}-GitHubConnectionArn`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
  }
}
