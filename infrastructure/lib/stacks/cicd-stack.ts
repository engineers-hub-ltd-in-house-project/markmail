import * as cdk from 'aws-cdk-lib';
import * as codepipeline from 'aws-cdk-lib/aws-codepipeline';
import * as codepipeline_actions from 'aws-cdk-lib/aws-codepipeline-actions';
import * as codebuild from 'aws-cdk-lib/aws-codebuild';
import type * as ecr from 'aws-cdk-lib/aws-ecr';
import type * as ecs from 'aws-cdk-lib/aws-ecs';
import type { Construct } from 'constructs';

export interface CICDStackProps extends cdk.StackProps {
  environmentName: string;
  githubOwner: string;
  githubRepo: string;
  githubBranch: string;
  backendRepo: ecr.Repository;
  frontendRepo: ecr.Repository;
  backendService: ecs.FargateService;
  frontendService: ecs.FargateService;
  domainName?: string;
}

export class CICDStack extends cdk.Stack {
  public readonly pipeline: codepipeline.Pipeline;
  public readonly githubConnection: cdk.aws_codestarconnections.CfnConnection;

  constructor(scope: Construct, id: string, props: CICDStackProps) {
    super(scope, id, props);

    const {
      environmentName,
      githubOwner,
      githubRepo,
      githubBranch,
      backendRepo,
      frontendRepo,
      backendService,
      frontendService,
      domainName,
    } = props;

    // GitHub Connection
    this.githubConnection = new cdk.aws_codestarconnections.CfnConnection(
      this,
      'GitHubConnection',
      {
        connectionName: `markmail-${environmentName}-github`,
        providerType: 'GitHub',
      }
    );

    // Source Output
    const sourceOutput = new codepipeline.Artifact();

    // CodeBuild projects
    const backendBuildProject = new codebuild.PipelineProject(this, 'BackendBuild', {
      projectName: `markmail-${environmentName}-backend-build`,
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        computeType: codebuild.ComputeType.SMALL,
        privileged: true,
      },
      environmentVariables: {
        ECR_REPOSITORY_URI: {
          value: backendRepo.repositoryUri,
        },
        AWS_DEFAULT_REGION: {
          value: cdk.Stack.of(this).region,
        },
        AWS_ACCOUNT_ID: {
          value: cdk.Stack.of(this).account,
        },
      },
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          pre_build: {
            commands: [
              'echo Logging in to Amazon ECR...',
              'aws ecr get-login-password --region $AWS_DEFAULT_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_DEFAULT_REGION.amazonaws.com',
            ],
          },
          build: {
            commands: [
              'echo Build started on `date`',
              'cd backend',
              'docker build -t $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION .',
              'docker tag $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION $ECR_REPOSITORY_URI:latest',
            ],
          },
          post_build: {
            commands: [
              'echo Build completed on `date`',
              'echo Pushing the Docker images...',
              'docker push $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION',
              'docker push $ECR_REPOSITORY_URI:latest',
              'cd ..',
              'printf \'[{"name":"backend","imageUri":"%s"}]\' $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION > imagedefinitions.json',
            ],
          },
        },
        artifacts: {
          files: ['imagedefinitions.json'],
        },
      }),
    });

    const frontendBuildProject = new codebuild.PipelineProject(this, 'FrontendBuild', {
      projectName: `markmail-${environmentName}-frontend-build`,
      environment: {
        buildImage: codebuild.LinuxBuildImage.STANDARD_7_0,
        computeType: codebuild.ComputeType.SMALL,
        privileged: true,
      },
      environmentVariables: {
        ECR_REPOSITORY_URI: {
          value: frontendRepo.repositoryUri,
        },
        AWS_DEFAULT_REGION: {
          value: cdk.Stack.of(this).region,
        },
        AWS_ACCOUNT_ID: {
          value: cdk.Stack.of(this).account,
        },
        VITE_API_URL: {
          value: domainName ? `https://${domainName}/api` : '/api',
        },
      },
      buildSpec: codebuild.BuildSpec.fromObject({
        version: '0.2',
        phases: {
          pre_build: {
            commands: [
              'echo Logging in to Amazon ECR...',
              'aws ecr get-login-password --region $AWS_DEFAULT_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_DEFAULT_REGION.amazonaws.com',
            ],
          },
          build: {
            commands: [
              'echo Build started on `date`',
              'cd frontend',
              'docker build --build-arg VITE_API_URL=$VITE_API_URL -t $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION .',
              'docker tag $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION $ECR_REPOSITORY_URI:latest',
            ],
          },
          post_build: {
            commands: [
              'echo Build completed on `date`',
              'echo Pushing the Docker images...',
              'docker push $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION',
              'docker push $ECR_REPOSITORY_URI:latest',
              'cd ..',
              'printf \'[{"name":"frontend","imageUri":"%s"}]\' $ECR_REPOSITORY_URI:$CODEBUILD_RESOLVED_SOURCE_VERSION > imagedefinitions.json',
            ],
          },
        },
        artifacts: {
          files: ['imagedefinitions.json'],
        },
      }),
    });

    // Grant permissions to push to ECR
    backendRepo.grantPullPush(backendBuildProject);
    frontendRepo.grantPullPush(frontendBuildProject);

    // Pipeline
    this.pipeline = new codepipeline.Pipeline(this, 'Pipeline', {
      pipelineName: `markmail-${environmentName}`,
      restartExecutionOnUpdate: true,
    });

    // Source Stage
    this.pipeline.addStage({
      stageName: 'Source',
      actions: [
        new codepipeline_actions.CodeStarConnectionsSourceAction({
          actionName: 'GitHub_Source',
          owner: githubOwner,
          repo: githubRepo,
          branch: githubBranch,
          output: sourceOutput,
          connectionArn: this.githubConnection.attrConnectionArn,
        }),
      ],
    });

    // Build Stage
    const backendBuildOutput = new codepipeline.Artifact('BackendBuildOutput');
    const frontendBuildOutput = new codepipeline.Artifact('FrontendBuildOutput');

    this.pipeline.addStage({
      stageName: 'Build',
      actions: [
        new codepipeline_actions.CodeBuildAction({
          actionName: 'Backend_Build',
          project: backendBuildProject,
          input: sourceOutput,
          outputs: [backendBuildOutput],
          runOrder: 1,
        }),
        new codepipeline_actions.CodeBuildAction({
          actionName: 'Frontend_Build',
          project: frontendBuildProject,
          input: sourceOutput,
          outputs: [frontendBuildOutput],
          runOrder: 1,
        }),
      ],
    });

    // Deploy Stage
    this.pipeline.addStage({
      stageName: 'Deploy',
      actions: [
        new codepipeline_actions.EcsDeployAction({
          actionName: 'Deploy_Backend',
          service: backendService,
          input: backendBuildOutput,
          runOrder: 1,
        }),
        new codepipeline_actions.EcsDeployAction({
          actionName: 'Deploy_Frontend',
          service: frontendService,
          input: frontendBuildOutput,
          runOrder: 1,
        }),
      ],
    });

    // Export values
    new cdk.CfnOutput(this, 'PipelineName', {
      value: this.pipeline.pipelineName,
      exportName: `${this.stackName}-PipelineName`,
    });

    new cdk.CfnOutput(this, 'GitHubConnectionArn', {
      value: this.githubConnection.attrConnectionArn,
      exportName: `${this.stackName}-GitHubConnectionArn`,
    });

    // Stack tags
    cdk.Tags.of(this).add('Project', 'MarkMail');
    cdk.Tags.of(this).add('Environment', environmentName);
    cdk.Tags.of(this).add('ManagedBy', 'CDK');
    cdk.Tags.of(this).add('StackType', 'CICD');
  }
}
