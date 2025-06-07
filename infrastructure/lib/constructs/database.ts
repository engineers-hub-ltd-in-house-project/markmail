import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import * as elasticache from 'aws-cdk-lib/aws-elasticache';
import * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import { Construct } from 'constructs';

export interface DatabaseConstructProps {
  environmentName: string;
  vpc: ec2.Vpc;
  rdsSecurityGroup: ec2.SecurityGroup;
  cacheSecurityGroup: ec2.SecurityGroup;
  dbInstanceClass?: ec2.InstanceClass;
  dbInstanceSize?: ec2.InstanceSize;
}

export class DatabaseConstruct extends Construct {
  public readonly database: rds.DatabaseInstance;
  public readonly dbSecret: secretsmanager.Secret;
  public readonly cacheCluster: elasticache.CfnCacheCluster;

  constructor(scope: Construct, id: string, props: DatabaseConstructProps) {
    super(scope, id);

    const {
      environmentName,
      vpc,
      rdsSecurityGroup,
      cacheSecurityGroup,
      dbInstanceClass = ec2.InstanceClass.T3,
      dbInstanceSize = ec2.InstanceSize.MICRO,
    } = props;

    // RDS Secret
    this.dbSecret = new secretsmanager.Secret(this, 'DBSecret', {
      secretName: `markmail-${environmentName}-db-secret`,
      generateSecretString: {
        secretStringTemplate: JSON.stringify({ username: 'markmail' }),
        generateStringKey: 'password',
        excludeCharacters: ' %+~`#$&*()|[]{}:;<>?!\'/@"\\',
      },
    });

    // RDS Subnet Group
    const dbSubnetGroup = new rds.SubnetGroup(this, 'DBSubnetGroup', {
      vpc,
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
      },
      description: 'Subnet group for RDS',
    });

    // RDS Instance
    this.database = new rds.DatabaseInstance(this, 'Database', {
      instanceIdentifier: `markmail-${environmentName}-db`,
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_15,
      }),
      instanceType: ec2.InstanceType.of(dbInstanceClass, dbInstanceSize),
      vpc,
      vpcSubnets: {
        subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
      },
      subnetGroup: dbSubnetGroup,
      securityGroups: [rdsSecurityGroup],
      allocatedStorage: 20,
      maxAllocatedStorage: 100,
      storageEncrypted: true,
      multiAz: environmentName === 'prod',
      credentials: rds.Credentials.fromSecret(this.dbSecret),
      databaseName: 'markmail',
      backupRetention: cdk.Duration.days(environmentName === 'prod' ? 30 : 7),
      deleteAutomatedBackups: environmentName !== 'prod',
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.SNAPSHOT : cdk.RemovalPolicy.DESTROY,
    });

    // ElastiCache Subnet Group
    const cacheSubnetGroup = new elasticache.CfnSubnetGroup(this, 'CacheSubnetGroup', {
      description: 'Subnet group for ElastiCache',
      subnetIds: vpc.selectSubnets({
        subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
      }).subnetIds,
    });

    // ElastiCache Cluster
    this.cacheCluster = new elasticache.CfnCacheCluster(this, 'CacheCluster', {
      cacheNodeType: environmentName === 'prod' ? 'cache.r7g.large' : 'cache.t3.micro',
      engine: 'redis',
      numCacheNodes: 1,
      vpcSecurityGroupIds: [cacheSecurityGroup.securityGroupId],
      cacheSubnetGroupName: cacheSubnetGroup.ref,
      clusterName: `markmail-${environmentName}-cache`,
    });
    this.cacheCluster.addDependency(cacheSubnetGroup);
  }
}
