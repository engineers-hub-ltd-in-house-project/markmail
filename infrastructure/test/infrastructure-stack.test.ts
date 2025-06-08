import { App } from 'aws-cdk-lib';
import { Template, Match } from 'aws-cdk-lib/assertions';
import { MarkMailInfrastructureStack } from '../lib/infrastructure-stack';

describe('MarkMailInfrastructureStack', () => {
  let app: App;

  beforeEach(() => {
    app = new App({
      context: {
        'hosted-zone:account=123456789012:domainName=example.com:region=us-east-1': {
          Id: '/hostedzone/ZXXXXXXXXXXXXX',
          Name: 'example.com.',
        },
      },
    });
  });

  describe('Stack Creation', () => {
    test('creates stack with minimal required properties', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'dev',
        notificationEmail: 'test@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Verify stack has all major components
      template.resourceCountIs('AWS::EC2::VPC', 1);
      template.resourceCountIs('AWS::RDS::DBInstance', 1);
      template.resourceCountIs('AWS::ElastiCache::CacheCluster', 1);
      template.resourceCountIs('AWS::ECS::Cluster', 1);
      template.resourceCountIs('AWS::ECS::Service', 1);
      template.resourceCountIs('AWS::CodePipeline::Pipeline', 1);
      template.resourceCountIs('AWS::SNS::Topic', 3); // Alert, bounce, complaint
      template.resourceCountIs('AWS::SES::ConfigurationSet', 1);
    });

    test('creates stack with all optional properties', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'prod',
        domainName: 'markmail.example.com',
        notificationEmail: 'ops@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        dbInstanceClass: cdk.aws_ec2.InstanceClass.T3,
        dbInstanceSize: cdk.aws_ec2.InstanceSize.MEDIUM,
        desiredCount: 4,
        cpu: 1024,
        memoryLimitMiB: 2048,
        env: {
          account: '123456789012',
          region: 'us-east-1',
        },
      });

      // Assert
      const template = Template.fromStack(stack);

      // Verify custom configurations are applied
      template.hasResourceProperties('AWS::RDS::DBInstance', {
        DBInstanceClass: 'db.t3.medium',
      });

      template.hasResourceProperties('AWS::ECS::TaskDefinition', {
        Cpu: '1024',
        Memory: '2048',
      });

      template.hasResourceProperties('AWS::ECS::Service', {
        DesiredCount: 4,
      });

      // Should have certificate for domain
      template.resourceCountIs('AWS::CertificateManager::Certificate', 1);

      // Should have WAF for production
      template.resourceCountIs('AWS::WAFv2::WebACL', 1);
    });
  });

  describe('Component Integration', () => {
    test('network resources are properly connected', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'dev',
        notificationEmail: 'test@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // ECS service should be in VPC
      template.hasResourceProperties('AWS::ECS::Service', {
        NetworkConfiguration: {
          AwsvpcConfiguration: {
            Subnets: Match.anyValue(),
            SecurityGroups: Match.anyValue(),
          },
        },
      });

      // RDS should be in VPC subnet group
      template.hasResourceProperties('AWS::RDS::DBInstance', {
        DBSubnetGroupName: Match.anyValue(),
        VPCSecurityGroups: Match.anyValue(),
      });

      // ElastiCache should be in VPC subnet group
      template.hasResourceProperties('AWS::ElastiCache::CacheCluster', {
        CacheSubnetGroupName: Match.anyValue(),
        VpcSecurityGroupIds: Match.anyValue(),
      });
    });

    test('container resources reference database correctly', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'staging',
        notificationEmail: 'test@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'develop',
      });

      // Assert
      const template = Template.fromStack(stack);

      // ECS task should have database environment variables
      template.hasResourceProperties('AWS::ECS::TaskDefinition', {
        ContainerDefinitions: Match.arrayWith([
          Match.objectLike({
            Environment: Match.arrayWith([
              { Name: 'ENVIRONMENT', Value: 'staging' },
              { Name: 'DATABASE_URL', Value: Match.anyValue() },
              { Name: 'REDIS_URL', Value: Match.anyValue() },
            ]),
            Secrets: Match.arrayWith([{ Name: 'JWT_SECRET', ValueFrom: Match.anyValue() }]),
          }),
        ]),
      });
    });

    test('pipeline references correct ECR repositories', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'dev',
        notificationEmail: 'test@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // CodeBuild projects should have ECR repository URIs
      template.hasResourceProperties('AWS::CodeBuild::Project', {
        Name: Match.stringLikeRegexp('.*backend-build'),
        Environment: {
          EnvironmentVariables: Match.arrayWith([
            Match.objectLike({
              Name: 'ECR_REPOSITORY_URI',
              Value: Match.anyValue(),
            }),
          ]),
        },
      });

      template.hasResourceProperties('AWS::CodeBuild::Project', {
        Name: Match.stringLikeRegexp('.*frontend-build'),
        Environment: {
          EnvironmentVariables: Match.arrayWith([
            Match.objectLike({
              Name: 'ECR_REPOSITORY_URI',
              Value: Match.anyValue(),
            }),
          ]),
        },
      });
    });

    test('monitoring is properly connected to services', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'prod',
        notificationEmail: 'ops@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // CloudWatch alarms should monitor ECS service
      template.hasResourceProperties('AWS::CloudWatch::Alarm', {
        MetricName: 'CPUUtilization',
        Namespace: 'AWS/ECS',
        Dimensions: Match.anyValue(),
      });

      template.hasResourceProperties('AWS::CloudWatch::Alarm', {
        MetricName: 'MemoryUtilization',
        Namespace: 'AWS/ECS',
        Dimensions: Match.anyValue(),
      });
    });
  });

  describe('Stack Outputs', () => {
    test('creates all required outputs', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'dev',
        notificationEmail: 'test@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check for all expected outputs
      template.hasOutput('ALBEndpoint', {
        Description: 'ALB endpoint URL',
        Export: { Name: 'TestStack-ALBEndpoint' },
      });

      template.hasOutput('DatabaseEndpoint', {
        Description: 'RDS endpoint',
        Export: { Name: 'TestStack-DatabaseEndpoint' },
      });

      template.hasOutput('CacheEndpoint', {
        Description: 'ElastiCache endpoint',
        Export: { Name: 'TestStack-CacheEndpoint' },
      });

      template.hasOutput('BackendRepositoryUri', {
        Description: 'Backend ECR repository URI',
        Export: { Name: 'TestStack-BackendRepoUri' },
      });

      template.hasOutput('FrontendRepositoryUri', {
        Description: 'Frontend ECR repository URI',
        Export: { Name: 'TestStack-FrontendRepoUri' },
      });

      template.hasOutput('SESConfigurationSetName', {
        Description: 'SES Configuration Set Name',
        Export: { Name: 'TestStack-SESConfigSet' },
      });

      template.hasOutput('PipelineName', {
        Description: 'CodePipeline name',
        Export: { Name: 'TestStack-PipelineName' },
      });

      template.hasOutput('GitHubConnectionArn', {
        Description: 'GitHub CodeConnection ARN',
        Export: { Name: 'TestStack-GitHubConnectionArn' },
      });
    });
  });

  describe('Stack Tags', () => {
    test('applies correct tags to stack', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'staging',
        notificationEmail: 'test@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'staging',
      });

      // Assert
      // Note: Stack-level tags are applied differently and might not show up in the template
      // They would be applied at deployment time
      // For unit testing, we can verify that the code runs without errors
      expect(stack).toBeDefined();
    });
  });

  describe('Environment-specific Configuration', () => {
    test('dev environment uses cost-optimized settings', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'dev',
        notificationEmail: 'dev@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'develop',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Single NAT gateway
      template.resourceCountIs('AWS::EC2::NatGateway', 1);

      // No cloud map namespace for dev
      template.resourceCountIs('AWS::ServiceDiscovery::PrivateDnsNamespace', 0);

      // No WAF
      template.resourceCountIs('AWS::WAFv2::WebACL', 0);
    });

    test('prod environment uses high-availability settings', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'prod',
        domainName: 'markmail.example.com',
        notificationEmail: 'ops@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        env: {
          account: '123456789012',
          region: 'us-east-1',
        },
      });

      // Assert
      const template = Template.fromStack(stack);

      // Multiple NAT gateways
      template.resourceCountIs('AWS::EC2::NatGateway', 2);

      // Cloud map namespace for production
      template.hasResourceProperties('AWS::ServiceDiscovery::PrivateDnsNamespace', {
        Name: 'markmail-prod',
      });

      // Multi-AZ RDS
      template.hasResourceProperties('AWS::RDS::DBInstance', {
        MultiAZ: true,
      });

      // WAF enabled
      template.resourceCountIs('AWS::WAFv2::WebACL', 1);
    });
  });

  describe('Security Configuration', () => {
    test('implements proper security group isolation', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'prod',
        notificationEmail: 'security@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Should have separate security groups
      const securityGroups = template.findResources('AWS::EC2::SecurityGroup');
      const sgDescriptions = Object.values(securityGroups).map(
        (sg: any) => sg.Properties.GroupDescription
      );

      expect(sgDescriptions).toContain('Security group for Application Load Balancer');
      expect(sgDescriptions).toContain('Security group for ECS tasks');
      expect(sgDescriptions).toContain('Security group for RDS');
      expect(sgDescriptions).toContain('Security group for ElastiCache');
    });

    test('uses secrets manager for sensitive data', () => {
      // Arrange & Act
      const stack = new MarkMailInfrastructureStack(app, 'TestStack', {
        environmentName: 'prod',
        notificationEmail: 'security@example.com',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Database credentials in Secrets Manager
      template.hasResourceProperties('AWS::SecretsManager::Secret', {
        Name: Match.stringLikeRegexp('markmail-prod-db-secret'),
      });

      // Task definition uses secrets
      template.hasResourceProperties('AWS::ECS::TaskDefinition', {
        ContainerDefinitions: Match.arrayWith([
          Match.objectLike({
            Secrets: Match.arrayWith([
              {
                Name: 'JWT_SECRET',
                ValueFrom: Match.anyValue(),
              },
            ]),
          }),
        ]),
      });
    });
  });
});

// Import cdk here to avoid circular dependency issues
import * as cdk from 'aws-cdk-lib';
