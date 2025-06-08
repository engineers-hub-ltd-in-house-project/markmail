import { App, Stack } from 'aws-cdk-lib';
import { Template, Match } from 'aws-cdk-lib/assertions';
import { NetworkConstruct } from '../../lib/constructs/network';

describe('NetworkConstruct', () => {
  let app: App;
  let stack: Stack;

  beforeEach(() => {
    app = new App();
    stack = new Stack(app, 'TestStack');
  });

  describe('VPC Configuration', () => {
    test('creates VPC with correct configuration for dev environment', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::EC2::VPC', {
        EnableDnsHostnames: true,
        EnableDnsSupport: true,
        Tags: Match.arrayWith([
          Match.objectLike({
            Key: 'Name',
            Value: 'markmail-dev-vpc',
          }),
        ]),
      });

      // Check NAT Gateways count for dev (should be 1)
      template.resourceCountIs('AWS::EC2::NatGateway', 1);
    });

    test('creates VPC with 2 NAT gateways for prod environment', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'prod',
      });

      // Assert
      const template = Template.fromStack(stack);
      template.resourceCountIs('AWS::EC2::NatGateway', 2);
    });

    test('creates correct subnet configuration', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Should have 6 subnets total (3 types x 2 AZs)
      template.resourceCountIs('AWS::EC2::Subnet', 6);

      // Check subnet types
      template.hasResourceProperties('AWS::EC2::Subnet', {
        Tags: Match.arrayWith([
          Match.objectLike({
            Key: 'aws-cdk:subnet-type',
            Value: 'Public',
          }),
        ]),
      });

      template.hasResourceProperties('AWS::EC2::Subnet', {
        Tags: Match.arrayWith([
          Match.objectLike({
            Key: 'aws-cdk:subnet-type',
            Value: 'Private',
          }),
        ]),
      });

      template.hasResourceProperties('AWS::EC2::Subnet', {
        Tags: Match.arrayWith([
          Match.objectLike({
            Key: 'aws-cdk:subnet-type',
            Value: 'Isolated',
          }),
        ]),
      });
    });
  });

  describe('Security Groups', () => {
    test('creates ALB security group with correct ingress rules', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::EC2::SecurityGroup', {
        GroupDescription: 'Security group for Application Load Balancer',
        SecurityGroupIngress: Match.arrayWith([
          Match.objectLike({
            IpProtocol: 'tcp',
            FromPort: 80,
            ToPort: 80,
            CidrIp: '0.0.0.0/0',
          }),
          Match.objectLike({
            IpProtocol: 'tcp',
            FromPort: 443,
            ToPort: 443,
            CidrIp: '0.0.0.0/0',
          }),
        ]),
      });
    });

    test('creates ECS security group with ALB ingress only', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check ECS security group
      template.hasResourceProperties('AWS::EC2::SecurityGroup', {
        GroupDescription: 'Security group for ECS tasks',
      });

      // Check for ingress rule (may be defined as separate resource)
      template.hasResourceProperties('AWS::EC2::SecurityGroupIngress', {
        IpProtocol: 'tcp',
        FromPort: 8080,
        ToPort: 8080,
        GroupId: Match.anyValue(),
        SourceSecurityGroupId: Match.anyValue(),
      });
    });

    test('creates RDS security group with no outbound rules', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::EC2::SecurityGroup', {
        GroupDescription: 'Security group for RDS',
        SecurityGroupEgress: [
          {
            CidrIp: '255.255.255.255/32',
            Description: 'Disallow all traffic',
            FromPort: 252,
            IpProtocol: 'icmp',
            ToPort: 86,
          },
        ],
      });
    });

    test('creates Cache security group with Redis port access from ECS', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::EC2::SecurityGroup', {
        GroupDescription: 'Security group for ElastiCache',
      });

      // Check for ingress rule (may be defined as separate resource)
      template.hasResourceProperties('AWS::EC2::SecurityGroupIngress', {
        IpProtocol: 'tcp',
        FromPort: 6379,
        ToPort: 6379,
        GroupId: Match.anyValue(),
        SourceSecurityGroupId: Match.anyValue(),
      });
    });
  });

  describe('Construct Properties', () => {
    test('exposes all required properties', () => {
      // Arrange & Act
      const network = new NetworkConstruct(stack, 'Network', {
        environmentName: 'dev',
      });

      // Assert
      expect(network.vpc).toBeDefined();
      expect(network.albSecurityGroup).toBeDefined();
      expect(network.ecsSecurityGroup).toBeDefined();
      expect(network.rdsSecurityGroup).toBeDefined();
      expect(network.cacheSecurityGroup).toBeDefined();
    });
  });

  describe('Tagging', () => {
    test('VPC has correct name tag', () => {
      // Arrange & Act
      new NetworkConstruct(stack, 'Network', {
        environmentName: 'staging',
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::EC2::VPC', {
        Tags: Match.arrayWith([
          Match.objectLike({
            Key: 'Name',
            Value: 'markmail-staging-vpc',
          }),
        ]),
      });
    });
  });
});
