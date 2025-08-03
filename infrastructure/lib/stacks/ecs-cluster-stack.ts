import * as cdk from 'aws-cdk-lib';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as logs from 'aws-cdk-lib/aws-logs';
import type * as ec2 from 'aws-cdk-lib/aws-ec2';
import type * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import type { Construct } from 'constructs';

export interface ECSClusterStackProps extends cdk.StackProps {
  environmentName: string;
  vpc: ec2.Vpc;
  dbSecret: secretsmanager.Secret;
  aiSecret?: secretsmanager.Secret;
  stripeSecret?: secretsmanager.Secret;
}

export class ECSClusterStack extends cdk.Stack {
  public readonly cluster: ecs.Cluster;
  public readonly taskExecutionRole: iam.Role;
  public readonly taskRole: iam.Role;
  public readonly logGroup: logs.LogGroup;

  constructor(scope: Construct, id: string, props: ECSClusterStackProps) {
    super(scope, id, props);

    const { environmentName, vpc, dbSecret, aiSecret, stripeSecret } = props;

    // ECS Cluster
    this.cluster = new ecs.Cluster(this, 'Cluster', {
      clusterName: `markmail-${environmentName}`,
      vpc,
      containerInsights: environmentName === 'prod',
    });

    // CloudWatch Logs
    this.logGroup = new logs.LogGroup(this, 'LogGroup', {
      logGroupName: `/ecs/markmail-${environmentName}`,
      retention: logs.RetentionDays.ONE_MONTH,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
    });

    // Task Execution Role
    this.taskExecutionRole = new iam.Role(this, 'TaskExecutionRole', {
      assumedBy: new iam.ServicePrincipal('ecs-tasks.amazonaws.com'),
      managedPolicies: [
        iam.ManagedPolicy.fromAwsManagedPolicyName('service-role/AmazonECSTaskExecutionRolePolicy'),
      ],
    });

    // Task Role
    this.taskRole = new iam.Role(this, 'TaskRole', {
      assumedBy: new iam.ServicePrincipal('ecs-tasks.amazonaws.com'),
    });

    // ECR permissions for task execution role
    this.taskExecutionRole.addToPolicy(
      new iam.PolicyStatement({
        effect: iam.Effect.ALLOW,
        actions: [
          'ecr:GetAuthorizationToken',
          'ecr:BatchCheckLayerAvailability',
          'ecr:GetDownloadUrlForLayer',
          'ecr:BatchGetImage',
        ],
        resources: ['*'],
      })
    );

    // Secrets Manager permissions for task execution role
    const secretArns = [dbSecret.secretArn];
    if (aiSecret) {
      secretArns.push(aiSecret.secretArn);
    }
    if (stripeSecret) {
      secretArns.push(stripeSecret.secretArn);
    }

    this.taskExecutionRole.addToPolicy(
      new iam.PolicyStatement({
        effect: iam.Effect.ALLOW,
        actions: ['secretsmanager:GetSecretValue'],
        resources: secretArns,
      })
    );

    // SES permissions for task role
    this.taskRole.addToPolicy(
      new iam.PolicyStatement({
        effect: iam.Effect.ALLOW,
        actions: [
          'ses:SendEmail',
          'ses:SendRawEmail',
          'ses:SendTemplatedEmail',
          'ses:SendBulkTemplatedEmail',
          'ses:GetSendQuota',
          'ses:GetSendStatistics',
          // SES v2 API permissions
          'ses:SendEmail',
          'sesv2:SendEmail',
          'sesv2:PutEmailIdentityDkimAttributes',
          'sesv2:GetEmailIdentity',
          'sesv2:ListEmailIdentities',
          'sesv2:CreateEmailIdentity',
          'sesv2:GetAccount',
          'sesv2:GetSuppressedDestination',
          'sesv2:ListSuppressedDestinations',
        ],
        resources: ['*'],
      })
    );

    // Export values for cross-stack references
    new cdk.CfnOutput(this, 'ClusterName', {
      value: this.cluster.clusterName,
      exportName: `${this.stackName}-ClusterName`,
    });

    new cdk.CfnOutput(this, 'ClusterArn', {
      value: this.cluster.clusterArn,
      exportName: `${this.stackName}-ClusterArn`,
    });

    new cdk.CfnOutput(this, 'TaskExecutionRoleArn', {
      value: this.taskExecutionRole.roleArn,
      exportName: `${this.stackName}-TaskExecutionRoleArn`,
    });

    new cdk.CfnOutput(this, 'TaskRoleArn', {
      value: this.taskRole.roleArn,
      exportName: `${this.stackName}-TaskRoleArn`,
    });

    new cdk.CfnOutput(this, 'LogGroupName', {
      value: this.logGroup.logGroupName,
      exportName: `${this.stackName}-LogGroupName`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'ECSCluster');
  }
}
