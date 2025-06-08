import * as cdk from 'aws-cdk-lib';
import type { Construct } from 'constructs';
import { DatabaseConstruct } from '../constructs/database';

export interface DatabaseStackProps extends cdk.StackProps {
  environmentName: string;
  vpc: cdk.aws_ec2.Vpc;
  rdsSecurityGroup: cdk.aws_ec2.SecurityGroup;
  cacheSecurityGroup: cdk.aws_ec2.SecurityGroup;
  dbInstanceClass?: cdk.aws_ec2.InstanceClass;
  dbInstanceSize?: cdk.aws_ec2.InstanceSize;
}

export class DatabaseStack extends cdk.Stack {
  public readonly database: cdk.aws_rds.DatabaseInstance;
  public readonly dbSecret: cdk.aws_secretsmanager.Secret;
  public readonly cacheCluster: cdk.aws_elasticache.CfnCacheCluster;

  constructor(scope: Construct, id: string, props: DatabaseStackProps) {
    super(scope, id, props);

    const {
      environmentName,
      vpc,
      rdsSecurityGroup,
      cacheSecurityGroup,
      dbInstanceClass,
      dbInstanceSize,
    } = props;

    // Database Layer
    const database = new DatabaseConstruct(this, 'Database', {
      environmentName,
      vpc,
      rdsSecurityGroup,
      cacheSecurityGroup,
      dbInstanceClass,
      dbInstanceSize,
    });

    this.database = database.database;
    this.dbSecret = database.dbSecret;
    this.cacheCluster = database.cacheCluster;

    // Export values for cross-stack references
    new cdk.CfnOutput(this, 'DatabaseEndpoint', {
      value: this.database.dbInstanceEndpointAddress,
      exportName: `${this.stackName}-DatabaseEndpoint`,
    });

    new cdk.CfnOutput(this, 'DatabaseSecretArn', {
      value: this.dbSecret.secretArn,
      exportName: `${this.stackName}-DatabaseSecretArn`,
    });

    new cdk.CfnOutput(this, 'CacheEndpoint', {
      value: this.cacheCluster.attrRedisEndpointAddress,
      exportName: `${this.stackName}-CacheEndpoint`,
    });

    new cdk.CfnOutput(this, 'CachePort', {
      value: this.cacheCluster.attrRedisEndpointPort,
      exportName: `${this.stackName}-CachePort`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'Database');
  }
}
