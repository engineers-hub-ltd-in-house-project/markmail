import * as cdk from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { NetworkConstruct } from '../constructs/network';

export interface NetworkStackProps extends cdk.StackProps {
  environmentName: string;
}

export class NetworkStack extends cdk.Stack {
  public readonly vpc: cdk.aws_ec2.Vpc;
  public readonly ecsSecurityGroup: cdk.aws_ec2.SecurityGroup;
  public readonly rdsSecurityGroup: cdk.aws_ec2.SecurityGroup;
  public readonly cacheSecurityGroup: cdk.aws_ec2.SecurityGroup;
  public readonly albSecurityGroup: cdk.aws_ec2.SecurityGroup;

  constructor(scope: Construct, id: string, props: NetworkStackProps) {
    super(scope, id, props);

    const { environmentName } = props;

    // Network Layer
    const network = new NetworkConstruct(this, 'Network', {
      environmentName,
    });

    this.vpc = network.vpc;
    this.ecsSecurityGroup = network.ecsSecurityGroup;
    this.rdsSecurityGroup = network.rdsSecurityGroup;
    this.cacheSecurityGroup = network.cacheSecurityGroup;
    this.albSecurityGroup = network.albSecurityGroup;

    // Export values for cross-stack references
    new cdk.CfnOutput(this, 'VpcId', {
      value: this.vpc.vpcId,
      exportName: `${this.stackName}-VpcId`,
    });

    new cdk.CfnOutput(this, 'EcsSecurityGroupId', {
      value: this.ecsSecurityGroup.securityGroupId,
      exportName: `${this.stackName}-EcsSecurityGroupId`,
    });

    new cdk.CfnOutput(this, 'RdsSecurityGroupId', {
      value: this.rdsSecurityGroup.securityGroupId,
      exportName: `${this.stackName}-RdsSecurityGroupId`,
    });

    new cdk.CfnOutput(this, 'CacheSecurityGroupId', {
      value: this.cacheSecurityGroup.securityGroupId,
      exportName: `${this.stackName}-CacheSecurityGroupId`,
    });

    new cdk.CfnOutput(this, 'ALBSecurityGroupId', {
      value: this.albSecurityGroup.securityGroupId,
      exportName: `${this.stackName}-ALBSecurityGroupId`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'Network');
  }
}
