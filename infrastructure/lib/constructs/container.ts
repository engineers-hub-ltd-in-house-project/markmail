import * as cdk from 'aws-cdk-lib';
import type * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as ecsPatterns from 'aws-cdk-lib/aws-ecs-patterns';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as logs from 'aws-cdk-lib/aws-logs';
import type * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import * as certificatemanager from 'aws-cdk-lib/aws-certificatemanager';
import * as route53 from 'aws-cdk-lib/aws-route53';
import type * as rds from 'aws-cdk-lib/aws-rds';
import type * as elasticache from 'aws-cdk-lib/aws-elasticache';
import { Construct } from 'constructs';

export interface ContainerConstructProps {
  environmentName: string;
  vpc: ec2.Vpc;
  ecsSecurityGroup: ec2.SecurityGroup;
  database: rds.DatabaseInstance;
  dbSecret: secretsmanager.Secret;
  cacheCluster: elasticache.CfnCacheCluster;
  domainName?: string;
  desiredCount?: number;
  cpu?: number;
  memoryLimitMiB?: number;
}

export class ContainerConstruct extends Construct {
  public readonly cluster: ecs.Cluster;
  public readonly alb: ecsPatterns.ApplicationLoadBalancedFargateService;
  public readonly backendRepo: ecr.Repository;
  public readonly frontendRepo: ecr.Repository;
  public readonly taskExecutionRole: iam.Role;
  public readonly taskRole: iam.Role;

  constructor(scope: Construct, id: string, props: ContainerConstructProps) {
    super(scope, id);

    const {
      environmentName,
      vpc,
      ecsSecurityGroup,
      database,
      dbSecret,
      cacheCluster,
      domainName,
      desiredCount = 2,
      cpu = 512,
      memoryLimitMiB = 1024,
    } = props;

    // ECR Repositories
    this.backendRepo = new ecr.Repository(this, 'BackendRepository', {
      repositoryName: `markmail-${environmentName}-backend`,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
      lifecycleRules: [
        {
          maxImageCount: 10,
          description: 'Keep only 10 images',
        },
      ],
    });

    this.frontendRepo = new ecr.Repository(this, 'FrontendRepository', {
      repositoryName: `markmail-${environmentName}-frontend`,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
      lifecycleRules: [
        {
          maxImageCount: 10,
          description: 'Keep only 10 images',
        },
      ],
    });

    // ECS Cluster
    this.cluster = new ecs.Cluster(this, 'Cluster', {
      clusterName: `markmail-${environmentName}`,
      vpc,
    });

    // Enable container insights for production
    if (environmentName === 'prod') {
      this.cluster.addDefaultCloudMapNamespace({
        name: `markmail-${environmentName}`,
      });
    }

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
        ],
        resources: ['*'],
      })
    );

    // Secrets Manager permissions for task execution role
    this.taskExecutionRole.addToPolicy(
      new iam.PolicyStatement({
        effect: iam.Effect.ALLOW,
        actions: ['secretsmanager:GetSecretValue'],
        resources: [dbSecret.secretArn],
      })
    );

    // CloudWatch Logs
    const logGroup = new logs.LogGroup(this, 'LogGroup', {
      logGroupName: `/ecs/markmail-${environmentName}`,
      retention: logs.RetentionDays.ONE_MONTH,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
    });

    // Certificate and Hosted Zone (if domain is provided)
    let certificate: certificatemanager.Certificate | undefined;
    let hostedZone: route53.IHostedZone | undefined;

    if (domainName) {
      hostedZone = route53.HostedZone.fromLookup(this, 'HostedZone', {
        domainName: domainName.split('.').slice(-2).join('.'), // Get base domain
      });

      certificate = new certificatemanager.Certificate(this, 'Certificate', {
        domainName: domainName,
        subjectAlternativeNames: [`*.${domainName}`],
        validation: certificatemanager.CertificateValidation.fromDns(hostedZone),
      });
    }

    // ALB and Fargate Service
    this.alb = new ecsPatterns.ApplicationLoadBalancedFargateService(this, 'FargateService', {
      cluster: this.cluster,
      taskImageOptions: {
        image: ecs.ContainerImage.fromEcrRepository(this.backendRepo, 'latest'),
        containerPort: 8080,
        environment: {
          ENVIRONMENT: environmentName,
          DATABASE_URL: `postgresql://${dbSecret.secretValueFromJson('username').unsafeUnwrap()}:${dbSecret.secretValueFromJson('password').unsafeUnwrap()}@${database.dbInstanceEndpointAddress}:5432/markmail`,
          REDIS_URL: `redis://${cacheCluster.attrRedisEndpointAddress}:${cacheCluster.attrRedisEndpointPort}`,
          AWS_REGION: cdk.Stack.of(this).region,
        },
        secrets: {
          JWT_SECRET: ecs.Secret.fromSecretsManager(dbSecret, 'password'),
        },
        taskRole: this.taskRole,
        executionRole: this.taskExecutionRole,
        logDriver: ecs.LogDrivers.awsLogs({
          streamPrefix: 'markmail',
          logGroup,
        }),
      },
      desiredCount,
      cpu,
      memoryLimitMiB,
      assignPublicIp: false,
      domainName: domainName,
      domainZone: hostedZone,
      certificate,
      redirectHTTP: domainName ? true : false,
      securityGroups: [ecsSecurityGroup],
    });
  }
}
