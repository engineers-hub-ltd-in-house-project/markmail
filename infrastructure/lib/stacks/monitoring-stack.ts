import * as cdk from 'aws-cdk-lib';
import * as cloudwatch from 'aws-cdk-lib/aws-cloudwatch';
import * as cloudwatch_actions from 'aws-cdk-lib/aws-cloudwatch-actions';
import * as sns from 'aws-cdk-lib/aws-sns';
import * as sns_subscriptions from 'aws-cdk-lib/aws-sns-subscriptions';
import * as ses from 'aws-cdk-lib/aws-ses';
import type * as ecs from 'aws-cdk-lib/aws-ecs';
import type * as elbv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import type { Construct } from 'constructs';

export interface MonitoringStackProps extends cdk.StackProps {
  environmentName: string;
  notificationEmail: string;
  backendService: ecs.FargateService;
  frontendService: ecs.FargateService;
  loadBalancer: elbv2.ApplicationLoadBalancer;
  domainName?: string;
}

export class MonitoringStack extends cdk.Stack {
  public readonly alarmTopic: sns.Topic;
  public readonly sesConfigurationSet: ses.ConfigurationSet;

  constructor(scope: Construct, id: string, props: MonitoringStackProps) {
    super(scope, id, props);

    const {
      environmentName,
      notificationEmail,
      backendService,
      frontendService,
      loadBalancer,
      domainName,
    } = props;

    // SNS Topic for Alarms
    this.alarmTopic = new sns.Topic(this, 'AlarmTopic', {
      displayName: `MarkMail ${environmentName} Alarms`,
    });

    this.alarmTopic.addSubscription(new sns_subscriptions.EmailSubscription(notificationEmail));

    // CloudWatch Dashboard
    const dashboard = new cloudwatch.Dashboard(this, 'Dashboard', {
      dashboardName: `markmail-${environmentName}`,
    });

    // Backend Service Metrics
    const backendCpuMetric = backendService.metricCpuUtilization();
    const backendMemoryMetric = backendService.metricMemoryUtilization();

    // Frontend Service Metrics
    const frontendCpuMetric = frontendService.metricCpuUtilization();
    const frontendMemoryMetric = frontendService.metricMemoryUtilization();

    // ALB Metrics
    const targetResponseTimeMetric = loadBalancer.metricTargetResponseTime();
    const activeConnectionCountMetric = loadBalancer.metricActiveConnectionCount();
    const requestCountMetric = loadBalancer.metricRequestCount();

    // Add widgets to dashboard
    dashboard.addWidgets(
      new cloudwatch.GraphWidget({
        title: 'Backend Service Metrics',
        left: [backendCpuMetric, backendMemoryMetric],
      }),
      new cloudwatch.GraphWidget({
        title: 'Frontend Service Metrics',
        left: [frontendCpuMetric, frontendMemoryMetric],
      }),
      new cloudwatch.GraphWidget({
        title: 'ALB Metrics',
        left: [targetResponseTimeMetric],
        right: [activeConnectionCountMetric, requestCountMetric],
      })
    );

    // Backend Alarms
    new cloudwatch.Alarm(this, 'BackendHighCpuAlarm', {
      metric: backendCpuMetric,
      threshold: 80,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'Backend Service CPU utilization is too high',
    }).addAlarmAction(new cloudwatch_actions.SnsAction(this.alarmTopic));

    new cloudwatch.Alarm(this, 'BackendHighMemoryAlarm', {
      metric: backendMemoryMetric,
      threshold: 80,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'Backend Service memory utilization is too high',
    }).addAlarmAction(new cloudwatch_actions.SnsAction(this.alarmTopic));

    // Frontend Alarms
    new cloudwatch.Alarm(this, 'FrontendHighCpuAlarm', {
      metric: frontendCpuMetric,
      threshold: 80,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'Frontend Service CPU utilization is too high',
    }).addAlarmAction(new cloudwatch_actions.SnsAction(this.alarmTopic));

    new cloudwatch.Alarm(this, 'FrontendHighMemoryAlarm', {
      metric: frontendMemoryMetric,
      threshold: 80,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'Frontend Service memory utilization is too high',
    }).addAlarmAction(new cloudwatch_actions.SnsAction(this.alarmTopic));

    new cloudwatch.Alarm(this, 'HighResponseTimeAlarm', {
      metric: targetResponseTimeMetric,
      threshold: 1,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'ALB response time is too high',
    }).addAlarmAction(new cloudwatch_actions.SnsAction(this.alarmTopic));

    // SES Configuration Set
    this.sesConfigurationSet = new ses.ConfigurationSet(this, 'SESConfigurationSet', {
      configurationSetName: `markmail-${environmentName}`,
      sendingEnabled: true,
      suppressionReasons: ses.SuppressionReasons.BOUNCES_AND_COMPLAINTS,
    });

    // Domain verification for SES (if domain is provided)
    if (domainName) {
      new ses.EmailIdentity(this, 'DomainIdentity', {
        identity: ses.Identity.domain(domainName),
      });
    }

    // Export values
    new cdk.CfnOutput(this, 'AlarmTopicArn', {
      value: this.alarmTopic.topicArn,
      exportName: `${this.stackName}-AlarmTopicArn`,
    });

    new cdk.CfnOutput(this, 'SESConfigurationSetName', {
      value: this.sesConfigurationSet.configurationSetName!,
      exportName: `${this.stackName}-SESConfigSet`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'Monitoring');
  }
}
