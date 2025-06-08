import * as cdk from 'aws-cdk-lib';
import * as route53 from 'aws-cdk-lib/aws-route53';
import type { Construct } from 'constructs';

export interface Route53StackProps extends cdk.StackProps {
  environmentName: string;
  domainName: string;
}

export class Route53Stack extends cdk.Stack {
  public readonly hostedZone: route53.HostedZone;

  constructor(scope: Construct, id: string, props: Route53StackProps) {
    super(scope, id, props);

    const { environmentName, domainName } = props;

    // Create a new hosted zone for the subdomain
    this.hostedZone = new route53.HostedZone(this, 'SubdomainHostedZone', {
      zoneName: domainName,
      comment: `MarkMail Application Subdomain for ${environmentName} environment`,
    });

    // Output the nameservers for delegation
    new cdk.CfnOutput(this, 'NameServers', {
      value: cdk.Fn.join(',', this.hostedZone.hostedZoneNameServers!),
      description: 'Name servers to configure in Squarespace for subdomain delegation',
      exportName: `${this.stackName}-NameServers`,
    });

    // Output the hosted zone ID
    new cdk.CfnOutput(this, 'HostedZoneId', {
      value: this.hostedZone.hostedZoneId,
      exportName: `${this.stackName}-HostedZoneId`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'Route53');
  }
}
