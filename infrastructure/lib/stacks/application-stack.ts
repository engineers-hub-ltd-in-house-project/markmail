import * as cdk from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { ContainerConstruct } from '../constructs/container';
import { CICDConstruct } from '../constructs/cicd';
import { MonitoringConstruct } from '../constructs/monitoring';

export interface ApplicationStackProps extends cdk.StackProps {
  environmentName: string;
  vpc: cdk.aws_ec2.Vpc;
  ecsSecurityGroup: cdk.aws_ec2.SecurityGroup;
  database: cdk.aws_rds.DatabaseInstance;
  dbSecret: cdk.aws_secretsmanager.Secret;
  cacheCluster: cdk.aws_elasticache.CfnCacheCluster;
  domainName?: string;
  notificationEmail: string;
  githubOwner: string;
  githubRepo: string;
  githubBranch: string;
  desiredCount?: number;
  cpu?: number;
  memoryLimitMiB?: number;
}

export class ApplicationStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: ApplicationStackProps) {
    super(scope, id, props);

    const {
      environmentName,
      vpc,
      ecsSecurityGroup,
      database,
      dbSecret,
      cacheCluster,
      domainName,
      notificationEmail,
      githubOwner,
      githubRepo,
      githubBranch,
      desiredCount,
      cpu,
      memoryLimitMiB,
    } = props;

    // Container Layer
    const container = new ContainerConstruct(this, 'Container', {
      environmentName,
      vpc,
      ecsSecurityGroup,
      database,
      dbSecret,
      cacheCluster,
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
    cdk.Tags.of(this).add('StackType', 'Application');
  }
}
