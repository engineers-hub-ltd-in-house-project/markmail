import { App, Stack } from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as elasticloadbalancingv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Template, Match } from 'aws-cdk-lib/assertions';
import { MonitoringConstruct } from '../../lib/constructs/monitoring';

describe('MonitoringConstruct', () => {
  let app: App;
  let stack: Stack;
  let service: ecs.FargateService;
  let loadBalancer: elasticloadbalancingv2.ApplicationLoadBalancer;

  beforeEach(() => {
    app = new App();
    stack = new Stack(app, 'TestStack');

    // Create mock VPC and ECS resources
    const vpc = new ec2.Vpc(stack, 'TestVPC', {
      maxAzs: 2,
    });

    const cluster = new ecs.Cluster(stack, 'TestCluster', {
      vpc,
    });

    const taskDefinition = new ecs.FargateTaskDefinition(stack, 'TestTaskDef', {
      cpu: 512,
      memoryLimitMiB: 1024,
    });

    taskDefinition.addContainer('TestContainer', {
      image: ecs.ContainerImage.fromRegistry('nginx:latest'),
      memoryLimitMiB: 1024,
    });

    service = new ecs.FargateService(stack, 'TestService', {
      cluster,
      taskDefinition,
    });

    loadBalancer = new elasticloadbalancingv2.ApplicationLoadBalancer(stack, 'TestALB', {
      vpc,
      internetFacing: true,
    });
  });

  describe('SNS Topics', () => {
    test('creates alert topic with email subscription', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'dev',
        notificationEmail: 'alerts@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::SNS::Topic', {
        TopicName: 'markmail-dev-alerts',
        DisplayName: 'MarkMail dev Alerts',
      });

      template.hasResourceProperties('AWS::SNS::Subscription', {
        Protocol: 'email',
        Endpoint: 'alerts@example.com',
        TopicArn: Match.anyValue(),
      });
    });

    test('creates bounce and complaint topics for SES', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'staging',
        notificationEmail: 'bounce@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::SNS::Topic', {
        TopicName: 'markmail-staging-bounces',
        DisplayName: 'MarkMail staging Bounce Notifications',
      });

      template.hasResourceProperties('AWS::SNS::Topic', {
        TopicName: 'markmail-staging-complaints',
        DisplayName: 'MarkMail staging Complaint Notifications',
      });

      // Check subscriptions
      const subscriptions = template.findResources('AWS::SNS::Subscription', {
        Properties: {
          Protocol: 'email',
          Endpoint: 'bounce@example.com',
        },
      });

      expect(Object.keys(subscriptions).length).toBeGreaterThanOrEqual(3); // Alert, bounce, and complaint topics
    });
  });

  describe('CloudWatch Alarms', () => {
    test('creates CPU utilization alarm', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'ops@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CloudWatch::Alarm', {
        MetricName: 'CPUUtilization',
        Namespace: 'AWS/ECS',
        Statistic: 'Average',
        Period: 300,
        EvaluationPeriods: 2,
        DatapointsToAlarm: 2,
        Threshold: 80,
        ComparisonOperator: 'GreaterThanOrEqualToThreshold',
        TreatMissingData: 'notBreaching',
        AlarmDescription: 'Alert when CPU utilization is too high',
      });
    });

    test('creates memory utilization alarm', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'ops@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CloudWatch::Alarm', {
        MetricName: 'MemoryUtilization',
        Namespace: 'AWS/ECS',
        Statistic: 'Average',
        Period: 300,
        EvaluationPeriods: 2,
        DatapointsToAlarm: 2,
        Threshold: 80,
        ComparisonOperator: 'GreaterThanOrEqualToThreshold',
        TreatMissingData: 'notBreaching',
        AlarmDescription: 'Alert when memory utilization is too high',
      });
    });

    test('alarms are configured to send to SNS topic', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'dev',
        notificationEmail: 'alerts@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CloudWatch::Alarm', {
        AlarmActions: [
          Match.objectLike({
            Ref: Match.stringLikeRegexp('.*AlertTopic.*'),
          }),
        ],
      });
    });
  });

  describe('SES Configuration', () => {
    test('creates SES configuration set', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'staging',
        notificationEmail: 'mail@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::SES::ConfigurationSet', {
        Name: 'markmail-staging',
      });
    });
  });

  describe('WAF Configuration', () => {
    test('creates WAF WebACL for production with domain', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'security@example.com',
        service,
        loadBalancer,
        domainName: 'markmail.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::WAFv2::WebACL', {
        Scope: 'REGIONAL',
        DefaultAction: { Allow: {} },
        Rules: Match.arrayWith([
          Match.objectLike({
            Name: 'RateLimitRule',
            Priority: 1,
            Statement: {
              RateBasedStatement: {
                Limit: 2000,
                AggregateKeyType: 'IP',
              },
            },
            Action: { Block: {} },
          }),
          Match.objectLike({
            Name: 'CommonRuleSet',
            Priority: 2,
            Statement: {
              ManagedRuleGroupStatement: {
                VendorName: 'AWS',
                Name: 'AWSManagedRulesCommonRuleSet',
              },
            },
          }),
        ]),
      });
    });

    test('associates WAF with ALB in production', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'security@example.com',
        service,
        loadBalancer,
        domainName: 'markmail.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      // WAF association properties are different in CDK output
      const resources = template.toJSON().Resources;
      const wafAssociation = Object.values(resources).find(
        (r: any) => r.Type === 'AWS::WAFv2::WebACLAssociation'
      ) as any;

      expect(wafAssociation).toBeDefined();
      expect(wafAssociation.Properties.ResourceArn).toBeDefined();
      expect(wafAssociation.Properties.WebACLArn).toBeDefined();
    });

    test('does not create WAF for non-production environments', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'dev',
        notificationEmail: 'dev@example.com',
        service,
        loadBalancer,
        domainName: 'dev.markmail.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      const resources = template.toJSON().Resources;
      const wafResources = Object.values(resources).filter(
        (r: any) => r.Type === 'AWS::WAFv2::WebACL'
      );

      expect(wafResources).toHaveLength(0);
    });

    test('does not create WAF for production without domain', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'prod@example.com',
        service,
        loadBalancer,
      });

      // Assert
      const template = Template.fromStack(stack);

      const resources = template.toJSON().Resources;
      const wafResources = Object.values(resources).filter(
        (r: any) => r.Type === 'AWS::WAFv2::WebACL'
      );

      expect(wafResources).toHaveLength(0);
    });
  });

  describe('Construct Properties', () => {
    test('exposes all required properties', () => {
      // Arrange & Act
      const monitoring = new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'dev',
        notificationEmail: 'test@example.com',
        service,
        loadBalancer,
      });

      // Assert
      expect(monitoring.alertTopic).toBeDefined();
      expect(monitoring.sesConfigurationSet).toBeDefined();
      expect(monitoring.bounceSnsTopic).toBeDefined();
      expect(monitoring.complaintSnsTopic).toBeDefined();
    });
  });

  describe('WAF Rules', () => {
    test('rate limit rule is configured correctly', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'security@example.com',
        service,
        loadBalancer,
        domainName: 'markmail.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::WAFv2::WebACL', {
        Rules: Match.arrayWith([
          Match.objectLike({
            Name: 'RateLimitRule',
            Statement: {
              RateBasedStatement: {
                Limit: 2000,
                AggregateKeyType: 'IP',
              },
            },
            VisibilityConfig: {
              SampledRequestsEnabled: true,
              CloudWatchMetricsEnabled: true,
              MetricName: 'RateLimitRule',
            },
          }),
        ]),
      });
    });

    test('managed rule set is configured correctly', () => {
      // Arrange & Act
      new MonitoringConstruct(stack, 'Monitoring', {
        environmentName: 'prod',
        notificationEmail: 'security@example.com',
        service,
        loadBalancer,
        domainName: 'markmail.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::WAFv2::WebACL', {
        Rules: Match.arrayWith([
          Match.objectLike({
            Name: 'CommonRuleSet',
            OverrideAction: { None: {} },
            Statement: {
              ManagedRuleGroupStatement: {
                VendorName: 'AWS',
                Name: 'AWSManagedRulesCommonRuleSet',
              },
            },
            VisibilityConfig: {
              SampledRequestsEnabled: true,
              CloudWatchMetricsEnabled: true,
              MetricName: 'CommonRuleSet',
            },
          }),
        ]),
      });
    });
  });
});
