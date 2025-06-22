import * as cdk from 'aws-cdk-lib';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as elbv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import type * as ecr from 'aws-cdk-lib/aws-ecr';
import type * as iam from 'aws-cdk-lib/aws-iam';
import type * as logs from 'aws-cdk-lib/aws-logs';
import type * as ec2 from 'aws-cdk-lib/aws-ec2';
import type * as rds from 'aws-cdk-lib/aws-rds';
import type * as elasticache from 'aws-cdk-lib/aws-elasticache';
import type * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import type { Construct } from 'constructs';

export interface ECSServiceStackProps extends cdk.StackProps {
  environmentName: string;
  vpc: ec2.Vpc;
  ecsSecurityGroup: ec2.SecurityGroup;
  cluster: ecs.Cluster;
  taskExecutionRole: iam.Role;
  taskRole: iam.Role;
  logGroup: logs.LogGroup;
  backendRepo: ecr.Repository;
  frontendRepo: ecr.Repository;
  database: rds.DatabaseInstance;
  dbSecret: secretsmanager.Secret;
  aiSecret?: secretsmanager.Secret;
  stripeSecret?: secretsmanager.Secret;
  cacheCluster: elasticache.CfnCacheCluster;
  loadBalancer: elbv2.ApplicationLoadBalancer;
  httpsListener?: elbv2.ApplicationListener;
  httpListener: elbv2.ApplicationListener;
  desiredCount?: number;
  cpu?: number;
  memoryLimitMiB?: number;
}

export class ECSServiceStack extends cdk.Stack {
  public readonly backendService: ecs.FargateService;
  public readonly frontendService: ecs.FargateService;
  public readonly backendTaskDefinition: ecs.FargateTaskDefinition;
  public readonly frontendTaskDefinition: ecs.FargateTaskDefinition;
  public readonly backendTargetGroup: elbv2.ApplicationTargetGroup;
  public readonly frontendTargetGroup: elbv2.ApplicationTargetGroup;
  // For backward compatibility with CI/CD stack
  public readonly service: ecs.FargateService;

  constructor(scope: Construct, id: string, props: ECSServiceStackProps) {
    super(scope, id, props);

    const {
      environmentName,
      vpc,
      ecsSecurityGroup,
      cluster,
      taskExecutionRole,
      taskRole,
      logGroup,
      backendRepo,
      frontendRepo,
      database,
      dbSecret,
      aiSecret,
      stripeSecret,
      cacheCluster,
      httpsListener,
      httpListener,
      desiredCount = 2,
      cpu = 512,
      memoryLimitMiB = 1024,
    } = props;

    // Backend Task Definition
    this.backendTaskDefinition = new ecs.FargateTaskDefinition(this, 'BackendTaskDefinition', {
      cpu: cpu,
      memoryLimitMiB: memoryLimitMiB,
      executionRole: taskExecutionRole,
      taskRole: taskRole,
    });

    // Backend Container
    const backendContainer = this.backendTaskDefinition.addContainer('backend', {
      image: ecs.ContainerImage.fromEcrRepository(backendRepo, 'latest'),
      environment: {
        ENVIRONMENT: environmentName,
        DATABASE_URL: `postgresql://${dbSecret.secretValueFromJson('username').unsafeUnwrap()}:${dbSecret.secretValueFromJson('password').unsafeUnwrap()}@${database.dbInstanceEndpointAddress}:5432/markmail`,
        REDIS_URL: `redis://${cacheCluster.attrRedisEndpointAddress}:${cacheCluster.attrRedisEndpointPort}`,
        AWS_REGION: cdk.Stack.of(this).region,
        SERVER_HOST: '0.0.0.0',
        SERVER_PORT: '8080',
        PORT: '8080',
        RUST_LOG: 'info',
        // Email configuration
        EMAIL_PROVIDER: 'aws_ses',
        AWS_SES_FROM_EMAIL: 'no-reply@engineers-hub.ltd',
      },
      secrets: {
        JWT_SECRET: ecs.Secret.fromSecretsManager(dbSecret, 'password'),
        ...(aiSecret && {
          OPENAI_API_KEY: ecs.Secret.fromSecretsManager(aiSecret, 'OPENAI_API_KEY'),
          ANTHROPIC_API_KEY: ecs.Secret.fromSecretsManager(aiSecret, 'ANTHROPIC_API_KEY'),
          AI_PROVIDER: ecs.Secret.fromSecretsManager(aiSecret, 'AI_PROVIDER'),
          OPENAI_MODEL: ecs.Secret.fromSecretsManager(aiSecret, 'OPENAI_MODEL'),
          ANTHROPIC_MODEL: ecs.Secret.fromSecretsManager(aiSecret, 'ANTHROPIC_MODEL'),
        }),
        ...(stripeSecret && {
          STRIPE_SECRET_KEY: ecs.Secret.fromSecretsManager(stripeSecret, 'STRIPE_SECRET_KEY'),
          STRIPE_PUBLISHABLE_KEY: ecs.Secret.fromSecretsManager(
            stripeSecret,
            'STRIPE_PUBLISHABLE_KEY'
          ),
          STRIPE_WEBHOOK_SECRET: ecs.Secret.fromSecretsManager(
            stripeSecret,
            'STRIPE_WEBHOOK_SECRET'
          ),
        }),
      },
      logging: ecs.LogDrivers.awsLogs({
        streamPrefix: 'backend',
        logGroup,
      }),
      healthCheck: {
        command: ['CMD-SHELL', 'curl -f http://localhost:8080/health || exit 1'],
        interval: cdk.Duration.seconds(30),
        timeout: cdk.Duration.seconds(5),
        retries: 3,
      },
    });

    backendContainer.addPortMappings({
      containerPort: 8080,
      protocol: ecs.Protocol.TCP,
    });

    // Backend Target Group
    this.backendTargetGroup = new elbv2.ApplicationTargetGroup(this, 'BackendTargetGroup', {
      vpc,
      port: 8080,
      protocol: elbv2.ApplicationProtocol.HTTP,
      targetType: elbv2.TargetType.IP,
      healthCheck: {
        path: '/health',
        interval: cdk.Duration.seconds(30),
        timeout: cdk.Duration.seconds(5),
        healthyThresholdCount: 2,
        unhealthyThresholdCount: 3,
      },
    });

    // Backend Fargate Service
    this.backendService = new ecs.FargateService(this, 'BackendService', {
      cluster,
      taskDefinition: this.backendTaskDefinition,
      desiredCount,
      assignPublicIp: false,
      securityGroups: [ecsSecurityGroup],
      serviceName: `markmail-${environmentName}-backend`,
    });

    // Register backend targets
    this.backendService.attachToApplicationTargetGroup(this.backendTargetGroup);

    // Frontend Task Definition
    this.frontendTaskDefinition = new ecs.FargateTaskDefinition(this, 'FrontendTaskDefinition', {
      cpu: 256,
      memoryLimitMiB: 512,
      executionRole: taskExecutionRole,
      taskRole: taskRole,
    });

    // Frontend Container
    const frontendContainer = this.frontendTaskDefinition.addContainer('frontend', {
      image: ecs.ContainerImage.fromEcrRepository(frontendRepo, 'latest'),
      environment: {
        ENVIRONMENT: environmentName,
      },
      logging: ecs.LogDrivers.awsLogs({
        streamPrefix: 'frontend',
        logGroup,
      }),
      healthCheck: {
        command: [
          'CMD-SHELL',
          'wget --no-verbose --tries=1 --spider http://127.0.0.1:8080/ || exit 1',
        ],
        interval: cdk.Duration.seconds(30),
        timeout: cdk.Duration.seconds(5),
        retries: 3,
      },
    });

    frontendContainer.addPortMappings({
      containerPort: 8080,
      protocol: ecs.Protocol.TCP,
    });

    // Frontend Target Group
    this.frontendTargetGroup = new elbv2.ApplicationTargetGroup(this, 'FrontendTargetGroup', {
      vpc,
      port: 8080,
      protocol: elbv2.ApplicationProtocol.HTTP,
      targetType: elbv2.TargetType.IP,
      healthCheck: {
        path: '/',
        interval: cdk.Duration.seconds(30),
        timeout: cdk.Duration.seconds(5),
        healthyThresholdCount: 2,
        unhealthyThresholdCount: 3,
      },
    });

    // Frontend Fargate Service
    this.frontendService = new ecs.FargateService(this, 'FrontendService', {
      cluster,
      taskDefinition: this.frontendTaskDefinition,
      desiredCount,
      assignPublicIp: false,
      securityGroups: [ecsSecurityGroup],
      serviceName: `markmail-${environmentName}-frontend`,
    });

    // Register frontend targets
    this.frontendService.attachToApplicationTargetGroup(this.frontendTargetGroup);

    // Add listener rules
    const listener = httpsListener || httpListener;

    // Backend API rule (higher priority)
    new elbv2.ApplicationListenerRule(this, 'BackendListenerRule', {
      listener,
      priority: 10,
      targetGroups: [this.backendTargetGroup],
      conditions: [elbv2.ListenerCondition.pathPatterns(['/api/*', '/health'])],
    });

    // Frontend rule (default - lower priority)
    new elbv2.ApplicationListenerRule(this, 'FrontendListenerRule', {
      listener,
      priority: 100,
      targetGroups: [this.frontendTargetGroup],
      conditions: [elbv2.ListenerCondition.pathPatterns(['/*'])],
    });

    // Service Auto Scaling for Backend
    const backendScaling = this.backendService.autoScaleTaskCount({
      minCapacity: desiredCount,
      maxCapacity: desiredCount * 2,
    });

    backendScaling.scaleOnCpuUtilization('BackendCpuScaling', {
      targetUtilizationPercent: 70,
      scaleInCooldown: cdk.Duration.seconds(60),
      scaleOutCooldown: cdk.Duration.seconds(60),
    });

    backendScaling.scaleOnMemoryUtilization('BackendMemoryScaling', {
      targetUtilizationPercent: 80,
      scaleInCooldown: cdk.Duration.seconds(60),
      scaleOutCooldown: cdk.Duration.seconds(60),
    });

    // Service Auto Scaling for Frontend
    const frontendScaling = this.frontendService.autoScaleTaskCount({
      minCapacity: desiredCount,
      maxCapacity: desiredCount * 2,
    });

    frontendScaling.scaleOnCpuUtilization('FrontendCpuScaling', {
      targetUtilizationPercent: 70,
      scaleInCooldown: cdk.Duration.seconds(60),
      scaleOutCooldown: cdk.Duration.seconds(60),
    });

    // For backward compatibility with CI/CD stack
    this.service = this.backendService;

    // Export values for cross-stack references
    new cdk.CfnOutput(this, 'BackendServiceName', {
      value: this.backendService.serviceName,
      exportName: `${this.stackName}-BackendServiceName`,
    });

    new cdk.CfnOutput(this, 'BackendServiceArn', {
      value: this.backendService.serviceArn,
      exportName: `${this.stackName}-BackendServiceArn`,
    });

    new cdk.CfnOutput(this, 'FrontendServiceName', {
      value: this.frontendService.serviceName,
      exportName: `${this.stackName}-FrontendServiceName`,
    });

    new cdk.CfnOutput(this, 'FrontendServiceArn', {
      value: this.frontendService.serviceArn,
      exportName: `${this.stackName}-FrontendServiceArn`,
    });

    new cdk.CfnOutput(this, 'BackendTaskDefinitionArn', {
      value: this.backendTaskDefinition.taskDefinitionArn,
      exportName: `${this.stackName}-BackendTaskDefinitionArn`,
    });

    new cdk.CfnOutput(this, 'FrontendTaskDefinitionArn', {
      value: this.frontendTaskDefinition.taskDefinitionArn,
      exportName: `${this.stackName}-FrontendTaskDefinitionArn`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'ECSService');
  }
}
