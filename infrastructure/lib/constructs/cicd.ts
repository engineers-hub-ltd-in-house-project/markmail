import * as cdk from 'aws-cdk-lib';
import * as codepipeline from 'aws-cdk-lib/aws-codepipeline';
import * as codepipelineActions from 'aws-cdk-lib/aws-codepipeline-actions';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import * as codeconnections from 'aws-cdk-lib/aws-codeconnections';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as s3 from 'aws-cdk-lib/aws-s3';
import type * as ecr from 'aws-cdk-lib/aws-ecr';
// import * as ecs from 'aws-cdk-lib/aws-ecs';
import type * as ecsPatterns from 'aws-cdk-lib/aws-ecs-patterns';
import { Construct } from 'constructs';

export interface CICDConstructProps {
  environmentName: string;
  githubOwner: string;
  githubRepo: string;
  githubBranch: string;
  backendRepo: ecr.Repository;
  frontendRepo: ecr.Repository;
  fargateService: ecsPatterns.ApplicationLoadBalancedFargateService;
  domainName?: string;
}

export class CICDConstruct extends Construct {
  public readonly pipeline: codepipeline.Pipeline;
  public readonly githubConnection: codeconnections.CfnConnection;

  constructor(scope: Construct, id: string, props: CICDConstructProps) {
    super(scope, id);

    const {
      environmentName,
      githubOwner,
      githubRepo,
      githubBranch,
      backendRepo,
      frontendRepo,
      fargateService,
      domainName,
    } = props;

    // S3 Bucket for artifacts
    const artifactBucket = new s3.Bucket(this, 'ArtifactBucket', {
      bucketName: `markmail-${environmentName}-artifacts-${cdk.Stack.of(this).account}`,
      removalPolicy:
        environmentName === 'prod' ? cdk.RemovalPolicy.RETAIN : cdk.RemovalPolicy.DESTROY,
      autoDeleteObjects: environmentName !== 'prod',
      encryption: s3.BucketEncryption.S3_MANAGED,
    });

    // CodeBuild Project for Backend
    const backendBuildProject = new codebuild.PipelineProject(this, 'BackendBuildProject', {
      projectName: `markmail-${environmentName}-backend-build`,
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        computeType: codebuild.ComputeType.MEDIUM,
        privileged: true,
      },
      environmentVariables: {
        AWS_ACCOUNT_ID: { value: cdk.Stack.of(this).account },
        AWS_REGION: { value: cdk.Stack.of(this).region },
        ECR_REPOSITORY_URI: { value: backendRepo.repositoryUri },
        ENVIRONMENT: { value: environmentName },
      },
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          pre_build: {
            commands: [
              'echo Logging in to Amazon ECR...',
              'aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com',
              'COMMIT_HASH=$(echo $CODEBUILD_RESOLVED_SOURCE_VERSION | cut -c 1-7)',
              'IMAGE_TAG=${COMMIT_HASH:=latest}',
            ],
          },
          build: {
            commands: [
              'cd backend',
              'echo Build started on `date`',
              'echo Building the Docker image...',
              'docker build -t $ECR_REPOSITORY_URI:latest .',
              'docker tag $ECR_REPOSITORY_URI:latest $ECR_REPOSITORY_URI:$IMAGE_TAG',
            ],
          },
          post_build: {
            commands: [
              'echo Build completed on `date`',
              'echo Pushing the Docker image...',
              'docker push $ECR_REPOSITORY_URI:latest',
              'docker push $ECR_REPOSITORY_URI:$IMAGE_TAG',
              'echo Writing image definitions file...',
              'printf \'[{"name":"container","imageUri":"%s"}]\' $ECR_REPOSITORY_URI:$IMAGE_TAG > imagedefinitions.json',
            ],
          },
        },
        artifacts: {
          files: ['imagedefinitions.json'],
        },
      }),
    });

    // CodeBuild Project for Frontend
    const frontendBuildProject = new codebuild.PipelineProject(this, 'FrontendBuildProject', {
      projectName: `markmail-${environmentName}-frontend-build`,
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        computeType: codebuild.ComputeType.MEDIUM,
        privileged: true,
      },
      environmentVariables: {
        AWS_ACCOUNT_ID: { value: cdk.Stack.of(this).account },
        AWS_REGION: { value: cdk.Stack.of(this).region },
        ECR_REPOSITORY_URI: { value: frontendRepo.repositoryUri },
        ENVIRONMENT: { value: environmentName },
        VITE_API_URL: {
          value: `https://${domainName || fargateService.loadBalancer.loadBalancerDnsName}/api`,
        },
      },
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          pre_build: {
            commands: [
              'echo Logging in to Amazon ECR...',
              'aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com',
              'COMMIT_HASH=$(echo $CODEBUILD_RESOLVED_SOURCE_VERSION | cut -c 1-7)',
              'IMAGE_TAG=${COMMIT_HASH:=latest}',
            ],
          },
          build: {
            commands: [
              'cd frontend',
              'echo Build started on `date`',
              'echo Building the Docker image...',
              'docker build --build-arg VITE_API_URL=$VITE_API_URL -t $ECR_REPOSITORY_URI:latest .',
              'docker tag $ECR_REPOSITORY_URI:latest $ECR_REPOSITORY_URI:$IMAGE_TAG',
            ],
          },
          post_build: {
            commands: [
              'echo Build completed on `date`',
              'echo Pushing the Docker image...',
              'docker push $ECR_REPOSITORY_URI:latest',
              'docker push $ECR_REPOSITORY_URI:$IMAGE_TAG',
            ],
          },
        },
      }),
    });

    // Grant ECR permissions
    backendRepo.grantPullPush(backendBuildProject);
    frontendRepo.grantPullPush(frontendBuildProject);

    // CodeConnections for GitHub
    this.githubConnection = new codeconnections.CfnConnection(this, 'GitHubConnection', {
      connectionName: `markmail-${environmentName}-github`,
      providerType: 'GitHub',
    });

    // Pipeline
    this.pipeline = new codepipeline.Pipeline(this, 'Pipeline', {
      pipelineName: `markmail-${environmentName}`,
      artifactBucket,
    });

    // Grant pipeline role permission to use CodeConnections
    this.pipeline.role.addToPrincipalPolicy(
      new iam.PolicyStatement({
        effect: iam.Effect.ALLOW,
        actions: ['codeconnections:UseConnection'],
        resources: [this.githubConnection.attrConnectionArn],
      })
    );

    // Source stage
    const sourceOutput = new codepipeline.Artifact();
    const sourceAction = new codepipelineActions.CodeStarConnectionsSourceAction({
      actionName: 'GitHub_Source',
      owner: githubOwner,
      repo: githubRepo,
      branch: githubBranch,
      connectionArn: this.githubConnection.attrConnectionArn,
      output: sourceOutput,
      codeBuildCloneOutput: true,
    });

    this.pipeline.addStage({
      stageName: 'Source',
      actions: [sourceAction],
    });

    // Build stage
    const backendBuildOutput = new codepipeline.Artifact();
    const frontendBuildOutput = new codepipeline.Artifact();

    this.pipeline.addStage({
      stageName: 'Build',
      actions: [
        new codepipelineActions.CodeBuildAction({
          actionName: 'Backend_Build',
          project: backendBuildProject,
          input: sourceOutput,
          outputs: [backendBuildOutput],
          runOrder: 1,
        }),
        new codepipelineActions.CodeBuildAction({
          actionName: 'Frontend_Build',
          project: frontendBuildProject,
          input: sourceOutput,
          outputs: [frontendBuildOutput],
          runOrder: 1,
        }),
      ],
    });

    // Deploy stage
    this.pipeline.addStage({
      stageName: 'Deploy',
      actions: [
        new codepipelineActions.EcsDeployAction({
          actionName: 'Deploy_to_ECS',
          service: fargateService.service,
          input: backendBuildOutput,
        }),
      ],
    });
  }
}
