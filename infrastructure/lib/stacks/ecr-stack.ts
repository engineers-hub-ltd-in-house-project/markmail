import * as cdk from 'aws-cdk-lib';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import type { Construct } from 'constructs';

export interface ECRStackProps extends cdk.StackProps {
  environmentName: string;
}

export class ECRStack extends cdk.Stack {
  public readonly backendRepo: ecr.Repository;
  public readonly frontendRepo: ecr.Repository;

  constructor(scope: Construct, id: string, props: ECRStackProps) {
    super(scope, id, props);

    const { environmentName } = props;

    // Backend ECR Repository
    this.backendRepo = new ecr.Repository(this, 'BackendRepository', {
      repositoryName: `markmail-${environmentName}-backend`,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
      lifecycleRules: [
        {
          maxImageCount: 10,
          description: 'Keep only 10 images',
        },
      ],
    });

    // Frontend ECR Repository
    this.frontendRepo = new ecr.Repository(this, 'FrontendRepository', {
      repositoryName: `markmail-${environmentName}-frontend`,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
      lifecycleRules: [
        {
          maxImageCount: 10,
          description: 'Keep only 10 images',
        },
      ],
    });

    // Export values for cross-stack references
    new cdk.CfnOutput(this, 'BackendRepositoryUri', {
      value: this.backendRepo.repositoryUri,
      description: 'Backend ECR repository URI',
      exportName: `${this.stackName}-BackendRepoUri`,
    });

    new cdk.CfnOutput(this, 'BackendRepositoryArn', {
      value: this.backendRepo.repositoryArn,
      description: 'Backend ECR repository ARN',
      exportName: `${this.stackName}-BackendRepoArn`,
    });

    new cdk.CfnOutput(this, 'FrontendRepositoryUri', {
      value: this.frontendRepo.repositoryUri,
      description: 'Frontend ECR repository URI',
      exportName: `${this.stackName}-FrontendRepoUri`,
    });

    new cdk.CfnOutput(this, 'FrontendRepositoryArn', {
      value: this.frontendRepo.repositoryArn,
      description: 'Frontend ECR repository ARN',
      exportName: `${this.stackName}-FrontendRepoArn`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'ECR');
  }
}
