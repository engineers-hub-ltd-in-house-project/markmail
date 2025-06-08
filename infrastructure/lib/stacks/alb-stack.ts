import * as cdk from 'aws-cdk-lib';
import * as elbv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import * as certificatemanager from 'aws-cdk-lib/aws-certificatemanager';
import * as route53 from 'aws-cdk-lib/aws-route53';
import * as route53targets from 'aws-cdk-lib/aws-route53-targets';
import type * as ec2 from 'aws-cdk-lib/aws-ec2';
import type { Construct } from 'constructs';

export interface ALBStackProps extends cdk.StackProps {
  environmentName: string;
  vpc: ec2.Vpc;
  albSecurityGroup: ec2.SecurityGroup;
  domainName?: string;
}

export class ALBStack extends cdk.Stack {
  public readonly loadBalancer: elbv2.ApplicationLoadBalancer;
  public readonly httpsListener: elbv2.ApplicationListener;
  public readonly httpListener: elbv2.ApplicationListener;
  public readonly albSecurityGroup: ec2.SecurityGroup;

  constructor(scope: Construct, id: string, props: ALBStackProps) {
    super(scope, id, props);

    const { environmentName, vpc, albSecurityGroup, domainName } = props;

    // Use the security group from NetworkStack
    this.albSecurityGroup = albSecurityGroup;

    // Application Load Balancer
    this.loadBalancer = new elbv2.ApplicationLoadBalancer(this, 'LoadBalancer', {
      vpc,
      internetFacing: true,
      securityGroup: this.albSecurityGroup,
      loadBalancerName: `markmail-${environmentName}-alb`,
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

      // HTTPS Listener
      this.httpsListener = this.loadBalancer.addListener('HttpsListener', {
        port: 443,
        certificates: [certificate],
        defaultAction: elbv2.ListenerAction.fixedResponse(503, {
          contentType: 'text/plain',
          messageBody: 'Service Unavailable',
        }),
      });

      // HTTP to HTTPS redirect
      this.httpListener = this.loadBalancer.addListener('HttpListener', {
        port: 80,
        defaultAction: elbv2.ListenerAction.redirect({
          protocol: 'HTTPS',
          port: '443',
          permanent: true,
        }),
      });

      // DNS record
      new route53.ARecord(this, 'AliasRecord', {
        zone: hostedZone,
        recordName: domainName,
        target: route53.RecordTarget.fromAlias(
          new route53targets.LoadBalancerTarget(this.loadBalancer)
        ),
      });
    } else {
      // HTTP Listener only for non-domain environments
      this.httpListener = this.loadBalancer.addListener('HttpListener', {
        port: 80,
        defaultAction: elbv2.ListenerAction.fixedResponse(503, {
          contentType: 'text/plain',
          messageBody: 'Service Unavailable',
        }),
      });
    }

    // Export values for cross-stack references
    new cdk.CfnOutput(this, 'LoadBalancerArn', {
      value: this.loadBalancer.loadBalancerArn,
      exportName: `${this.stackName}-LoadBalancerArn`,
    });

    new cdk.CfnOutput(this, 'LoadBalancerDnsName', {
      value: this.loadBalancer.loadBalancerDnsName,
      exportName: `${this.stackName}-LoadBalancerDnsName`,
    });

    new cdk.CfnOutput(this, 'LoadBalancerSecurityGroupId', {
      value: this.albSecurityGroup.securityGroupId,
      exportName: `${this.stackName}-ALBSecurityGroupId`,
    });

    if (this.httpsListener) {
      new cdk.CfnOutput(this, 'HttpsListenerArn', {
        value: this.httpsListener.listenerArn,
        exportName: `${this.stackName}-HttpsListenerArn`,
      });
    }

    new cdk.CfnOutput(this, 'HttpListenerArn', {
      value: this.httpListener.listenerArn,
      exportName: `${this.stackName}-HttpListenerArn`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'ALB');
  }
}
