import { App, Stack } from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Template, Match } from 'aws-cdk-lib/assertions';
import { DatabaseConstruct } from '../../lib/constructs/database';

describe('DatabaseConstruct', () => {
  let app: App;
  let stack: Stack;
  let vpc: ec2.Vpc;
  let rdsSecurityGroup: ec2.SecurityGroup;
  let cacheSecurityGroup: ec2.SecurityGroup;

  beforeEach(() => {
    app = new App();
    stack = new Stack(app, 'TestStack');

    // Create mock VPC and security groups
    vpc = new ec2.Vpc(stack, 'TestVPC', {
      maxAzs: 2,
      subnetConfiguration: [
        {
          name: 'Public',
          subnetType: ec2.SubnetType.PUBLIC,
          cidrMask: 24,
        },
        {
          name: 'Private',
          subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
          cidrMask: 24,
        },
        {
          name: 'Isolated',
          subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
          cidrMask: 24,
        },
      ],
    });

    rdsSecurityGroup = new ec2.SecurityGroup(stack, 'RDSSecurityGroup', {
      vpc,
      description: 'Test RDS security group',
    });

    cacheSecurityGroup = new ec2.SecurityGroup(stack, 'CacheSecurityGroup', {
      vpc,
      description: 'Test cache security group',
    });
  });

  describe('RDS Configuration', () => {
    test('creates RDS instance with correct properties for dev environment', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::RDS::DBInstance', {
        DBInstanceIdentifier: 'markmail-dev-db',
        Engine: 'postgres',
        DBInstanceClass: 'db.t3.micro',
        AllocatedStorage: '20',
        MaxAllocatedStorage: 100,
        StorageEncrypted: true,
        MultiAZ: false,
        BackupRetentionPeriod: 7,
        DeleteAutomatedBackups: true,
        DeletionProtection: Match.absent(),
      });
    });

    test('creates RDS instance with production settings', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'prod',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
        dbInstanceClass: ec2.InstanceClass.T3,
        dbInstanceSize: ec2.InstanceSize.SMALL,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::RDS::DBInstance', {
        DBInstanceIdentifier: 'markmail-prod-db',
        DBInstanceClass: 'db.t3.small',
        MultiAZ: true,
        BackupRetentionPeriod: 30,
        DeleteAutomatedBackups: false,
      });
    });

    test('uses PostgreSQL 15', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::RDS::DBInstance', {
        Engine: 'postgres',
        EngineVersion: '15',
      });
    });
  });

  describe('Secrets Manager', () => {
    test('creates secret with correct configuration', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'staging',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::SecretsManager::Secret', {
        Name: 'markmail-staging-db-secret',
        GenerateSecretString: {
          SecretStringTemplate: JSON.stringify({ username: 'markmail' }),
          GenerateStringKey: 'password',
          ExcludeCharacters: ' %+~`#$&*()|[]{}:;<>?!\'/@"\\',
        },
      });
    });

    test('RDS uses credentials from Secrets Manager', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::RDS::DBInstance', {
        MasterUsername: {
          'Fn::Join': Match.anyValue(),
        },
        MasterUserPassword: {
          'Fn::Join': Match.anyValue(),
        },
      });
    });
  });

  describe('ElastiCache Configuration', () => {
    test('creates ElastiCache cluster for dev environment', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::ElastiCache::CacheCluster', {
        ClusterName: 'markmail-dev-cache',
        Engine: 'redis',
        CacheNodeType: 'cache.t3.micro',
        NumCacheNodes: 1,
      });
    });

    test('creates larger ElastiCache cluster for production', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'prod',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::ElastiCache::CacheCluster', {
        ClusterName: 'markmail-prod-cache',
        CacheNodeType: 'cache.r7g.large',
      });
    });

    test('creates subnet group for ElastiCache', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::ElastiCache::SubnetGroup', {
        Description: 'Subnet group for ElastiCache',
        SubnetIds: Match.anyValue(),
      });
    });
  });

  describe('Networking', () => {
    test('RDS is placed in isolated subnets', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::RDS::DBSubnetGroup', {
        DBSubnetGroupDescription: 'Subnet group for RDS',
        SubnetIds: Match.anyValue(),
      });
    });

    test('uses provided security groups', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Verify security groups are used
      const resources = template.toJSON().Resources;

      // Find RDS instance
      const rdsInstance = Object.values(resources).find(
        (r: any) => r.Type === 'AWS::RDS::DBInstance'
      ) as any;

      expect(rdsInstance).toBeDefined();
      expect(rdsInstance.Properties.VPCSecurityGroups).toBeDefined();
      expect(rdsInstance.Properties.VPCSecurityGroups.length).toBeGreaterThan(0);

      // Find Cache cluster
      const cacheCluster = Object.values(resources).find(
        (r: any) => r.Type === 'AWS::ElastiCache::CacheCluster'
      ) as any;

      expect(cacheCluster).toBeDefined();
      expect(cacheCluster.Properties.VpcSecurityGroupIds).toBeDefined();
      expect(cacheCluster.Properties.VpcSecurityGroupIds.length).toBeGreaterThan(0);
    });
  });

  describe('Construct Properties', () => {
    test('exposes all required properties', () => {
      // Arrange & Act
      const database = new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      expect(database.database).toBeDefined();
      expect(database.dbSecret).toBeDefined();
      expect(database.cacheCluster).toBeDefined();
    });
  });

  describe('Removal Policies', () => {
    test('dev environment uses DESTROY removal policy', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'dev',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check RDS deletion policy
      const resources = template.toJSON().Resources;
      const dbInstance = Object.values(resources).find(
        (r: any) => r.Type === 'AWS::RDS::DBInstance'
      );

      expect(dbInstance).toBeDefined();
      expect((dbInstance as any).DeletionPolicy).toBe('Delete');
    });

    test('prod environment uses SNAPSHOT removal policy', () => {
      // Arrange & Act
      new DatabaseConstruct(stack, 'Database', {
        environmentName: 'prod',
        vpc,
        rdsSecurityGroup,
        cacheSecurityGroup,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check RDS deletion policy
      const resources = template.toJSON().Resources;
      const dbInstance = Object.values(resources).find(
        (r: any) => r.Type === 'AWS::RDS::DBInstance'
      );

      expect(dbInstance).toBeDefined();
      expect((dbInstance as any).DeletionPolicy).toBe('Snapshot');
    });
  });
});
