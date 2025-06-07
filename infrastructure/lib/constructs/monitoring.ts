// import * as cdk from 'aws-cdk-lib';
import * as cloudwatch from 'aws-cdk-lib/aws-cloudwatch';
import * as cloudwatchActions from 'aws-cdk-lib/aws-cloudwatch-actions';
import * as sns from 'aws-cdk-lib/aws-sns';
import * as snsSubscriptions from 'aws-cdk-lib/aws-sns-subscriptions';
import * as ses from 'aws-cdk-lib/aws-ses';
import * as wafv2 from 'aws-cdk-lib/aws-wafv2';
import type * as ecs from 'aws-cdk-lib/aws-ecs';
import type * as elasticloadbalancingv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';

export interface MonitoringConstructProps {
  environmentName: string;
  notificationEmail: string;
  service: ecs.FargateService;
  loadBalancer: elasticloadbalancingv2.ApplicationLoadBalancer;
  domainName?: string;
}

export class MonitoringConstruct extends Construct {
  public readonly alertTopic: sns.Topic;
  public readonly sesConfigurationSet: ses.ConfigurationSet;
  public readonly bounceSnsTopic: sns.Topic;
  public readonly complaintSnsTopic: sns.Topic;

  constructor(scope: Construct, id: string, props: MonitoringConstructProps) {
    super(scope, id);

    const { environmentName, notificationEmail, service, loadBalancer, domainName } = props;

    // SNS Topic for alerts
    this.alertTopic = new sns.Topic(this, 'AlertTopic', {
      topicName: `markmail-${environmentName}-alerts`,
      displayName: `MarkMail ${environmentName} Alerts`,
    });

    this.alertTopic.addSubscription(new snsSubscriptions.EmailSubscription(notificationEmail));

    // CloudWatch Alarms
    new cloudwatch.Alarm(this, 'HighCPUAlarm', {
      metric: service.metricCpuUtilization(),
      threshold: 80,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'Alert when CPU utilization is too high',
    }).addAlarmAction(new cloudwatchActions.SnsAction(this.alertTopic));

    new cloudwatch.Alarm(this, 'HighMemoryAlarm', {
      metric: service.metricMemoryUtilization(),
      threshold: 80,
      evaluationPeriods: 2,
      datapointsToAlarm: 2,
      treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
      alarmDescription: 'Alert when memory utilization is too high',
    }).addAlarmAction(new cloudwatchActions.SnsAction(this.alertTopic));

    // SES Configuration Set
    this.sesConfigurationSet = new ses.ConfigurationSet(this, 'SESConfigurationSet', {
      configurationSetName: `markmail-${environmentName}`,
      sendingEnabled: true,
    });

    // Bounce/Complaint Topics
    this.bounceSnsTopic = new sns.Topic(this, 'BounceNotificationTopic', {
      topicName: `markmail-${environmentName}-bounces`,
      displayName: `MarkMail ${environmentName} Bounce Notifications`,
    });

    this.complaintSnsTopic = new sns.Topic(this, 'ComplaintNotificationTopic', {
      topicName: `markmail-${environmentName}-complaints`,
      displayName: `MarkMail ${environmentName} Complaint Notifications`,
    });

    this.bounceSnsTopic.addSubscription(new snsSubscriptions.EmailSubscription(notificationEmail));
    this.complaintSnsTopic.addSubscription(
      new snsSubscriptions.EmailSubscription(notificationEmail)
    );

    // WAF (Production only)
    if (environmentName === 'prod' && domainName) {
      const webAcl = new wafv2.CfnWebACL(this, 'WebACL', {
        scope: 'REGIONAL',
        defaultAction: { allow: {} },
        rules: [
          {
            name: 'RateLimitRule',
            priority: 1,
            statement: {
              rateBasedStatement: {
                limit: 2000,
                aggregateKeyType: 'IP',
              },
            },
            action: { block: {} },
            visibilityConfig: {
              sampledRequestsEnabled: true,
              cloudWatchMetricsEnabled: true,
              metricName: 'RateLimitRule',
            },
          },
          {
            name: 'CommonRuleSet',
            priority: 2,
            overrideAction: { none: {} },
            statement: {
              managedRuleGroupStatement: {
                vendorName: 'AWS',
                name: 'AWSManagedRulesCommonRuleSet',
              },
            },
            visibilityConfig: {
              sampledRequestsEnabled: true,
              cloudWatchMetricsEnabled: true,
              metricName: 'CommonRuleSet',
            },
          },
        ],
        visibilityConfig: {
          sampledRequestsEnabled: true,
          cloudWatchMetricsEnabled: true,
          metricName: 'WebACL',
        },
      });

      // Associate WAF with ALB
      new wafv2.CfnWebACLAssociation(this, 'WebACLAssociation', {
        resourceArn: loadBalancer.loadBalancerArn,
        webAclArn: webAcl.attrArn,
      });
    }
  }
}
