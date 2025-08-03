import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import type { Construct } from 'constructs';

export interface BastionStackProps extends cdk.StackProps {
  environmentName: string;
}

export class BastionStack extends cdk.Stack {
  public readonly bastionHost: ec2.BastionHostLinux;

  constructor(scope: Construct, id: string, props: BastionStackProps) {
    super(scope, id, props);

    const { environmentName } = props;

    // Import existing resources using Fn.importValue
    const vpcId = cdk.Fn.importValue(`MarkMail-${environmentName}-NetworkStack-VpcId`);
    const publicSubnetId1 = cdk.Fn.importValue(
      `MarkMail-${environmentName}-NetworkStack:ExportsOutputRefNetworkVPCPublicSubnet1Subnet8E5CCBD1D7C6EC91`
    );
    const publicSubnetId2 = cdk.Fn.importValue(
      `MarkMail-${environmentName}-NetworkStack:ExportsOutputRefNetworkVPCPublicSubnet2SubnetF5D10D0A6F61E242`
    );
    const rdsSecurityGroupId = cdk.Fn.importValue(
      `MarkMail-${environmentName}-NetworkStack-RdsSecurityGroupId`
    );

    // Import existing VPC - use specific attributes
    const vpc = ec2.Vpc.fromVpcAttributes(this, 'Vpc', {
      vpcId: vpcId,
      availabilityZones: ['ap-northeast-1a', 'ap-northeast-1c'],
      publicSubnetIds: [publicSubnetId1, publicSubnetId2],
    });

    // Import RDS security group
    const rdsSecurityGroup = ec2.SecurityGroup.fromSecurityGroupId(
      this,
      'RdsSecurityGroup',
      rdsSecurityGroupId
    );

    // Create bastion host
    this.bastionHost = new ec2.BastionHostLinux(this, 'BastionHost', {
      vpc,
      instanceName: `markmail-${environmentName}-bastion`,
      machineImage: ec2.MachineImage.latestAmazonLinux2023(),
      instanceType: ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.MICRO),
      subnetSelection: {
        subnetType: ec2.SubnetType.PUBLIC,
      },
    });

    // Add PostgreSQL client
    this.bastionHost.instance.addUserData('yum update -y', 'yum install -y postgresql15');

    // Allow bastion to connect to RDS
    rdsSecurityGroup.addIngressRule(
      this.bastionHost.instance.connections.securityGroups[0],
      ec2.Port.tcp(5432),
      'Allow bastion host to connect to RDS'
    );

    // Output
    new cdk.CfnOutput(this, 'BastionInstanceId', {
      value: this.bastionHost.instanceId,
      description: 'Bastion host instance ID for SSM Session Manager',
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'Bastion');
    cdk.Tags.of(this).add('Temporary', 'true'); // 一時的なリソースであることを明示
  }
}
