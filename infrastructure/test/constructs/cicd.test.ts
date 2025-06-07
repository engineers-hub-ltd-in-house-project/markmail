import { App, Stack } from 'aws-cdk-lib';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as ecsPatterns from 'aws-cdk-lib/aws-ecs-patterns';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Template, Match } from 'aws-cdk-lib/assertions';
import { CICDConstruct } from '../../lib/constructs/cicd';

describe('CICDConstruct', () => {
  let app: App;
  let stack: Stack;
  let backendRepo: ecr.Repository;
  let frontendRepo: ecr.Repository;
  let fargateService: ecsPatterns.ApplicationLoadBalancedFargateService;

  beforeEach(() => {
    app = new App();
    stack = new Stack(app, 'TestStack');

    // Create mock ECR repositories
    backendRepo = new ecr.Repository(stack, 'BackendRepo', {
      repositoryName: 'test-backend',
    });

    frontendRepo = new ecr.Repository(stack, 'FrontendRepo', {
      repositoryName: 'test-frontend',
    });

    // Create mock VPC and ECS cluster for the Fargate service
    const vpc = new ec2.Vpc(stack, 'TestVPC', {
      maxAzs: 2,
    });

    const cluster = new ecs.Cluster(stack, 'TestCluster', {
      vpc,
    });

    // Create mock Fargate service
    fargateService = new ecsPatterns.ApplicationLoadBalancedFargateService(stack, 'TestService', {
      cluster,
      taskImageOptions: {
        image: ecs.ContainerImage.fromRegistry('nginx:latest'),
      },
    });
  });

  describe('S3 Artifact Bucket', () => {
    test('creates artifact bucket with correct properties for dev', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check S3 bucket properties
      const resources = template.toJSON().Resources;
      const bucket = Object.values(resources).find((r: any) => r.Type === 'AWS::S3::Bucket') as any;

      expect(bucket).toBeDefined();
      expect(bucket.Properties.BucketName).toBeDefined();

      // Verify bucket name contains expected prefix
      const bucketNameJoin = bucket.Properties.BucketName['Fn::Join'];
      expect(bucketNameJoin).toBeDefined();
      expect(bucketNameJoin[1]).toContainEqual('markmail-dev-artifacts-');

      // Check encryption
      expect(bucket.Properties.BucketEncryption).toBeDefined();
      expect(
        bucket.Properties.BucketEncryption.ServerSideEncryptionConfiguration[0]
          .ServerSideEncryptionByDefault.SSEAlgorithm
      ).toBe('AES256');
    });

    test('sets retention policy for production', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'prod',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);
      const resources = template.toJSON().Resources;

      const bucket = Object.values(resources).find((r: any) => r.Type === 'AWS::S3::Bucket');

      expect(bucket).toBeDefined();
      expect((bucket as any).UpdateReplacePolicy).toBe('Retain');
      expect((bucket as any).DeletionPolicy).toBe('Retain');
    });
  });

  describe('CodeBuild Projects', () => {
    test('creates backend build project with correct configuration', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'staging',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'develop',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CodeBuild::Project', {
        Name: 'markmail-staging-backend-build',
        Environment: {
          ComputeType: 'BUILD_GENERAL1_MEDIUM',
          Image: 'aws/codebuild/standard:7.0',
          Type: 'LINUX_CONTAINER',
          PrivilegedMode: true,
          EnvironmentVariables: Match.arrayWith([
            Match.objectLike({ Name: 'AWS_ACCOUNT_ID', Value: Match.anyValue() }),
            Match.objectLike({ Name: 'AWS_REGION', Value: Match.anyValue() }),
            Match.objectLike({ Name: 'ECR_REPOSITORY_URI', Value: Match.anyValue() }),
            Match.objectLike({ Name: 'ENVIRONMENT', Value: 'staging' }),
          ]),
        },
      });
    });

    test('creates frontend build project with API URL', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'prod',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
        domainName: 'markmail.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check that frontend build project exists with VITE_API_URL
      const resources = template.toJSON().Resources;
      const frontendBuildProject = Object.values(resources).find(
        (r: any) =>
          r.Type === 'AWS::CodeBuild::Project' &&
          r.Properties?.Name === 'markmail-prod-frontend-build'
      ) as any;

      expect(frontendBuildProject).toBeDefined();
      const envVars = frontendBuildProject.Properties.Environment.EnvironmentVariables;
      const viteApiUrl = envVars.find((v: any) => v.Name === 'VITE_API_URL');
      expect(viteApiUrl).toBeDefined();
      expect(viteApiUrl.Value).toBe('https://markmail.example.com/api');
    });

    test('grants ECR permissions to build projects', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Check that ECR permissions are granted
      const resources = template.toJSON().Resources;
      const policies = Object.values(resources).filter((r: any) => r.Type === 'AWS::IAM::Policy');

      // Check if any policy contains ECR permissions
      const hasEcrPermissions = policies.some((policy: any) =>
        policy.Properties?.PolicyDocument?.Statement?.some(
          (statement: any) =>
            statement.Action === 'ecr:GetAuthorizationToken' ||
            (Array.isArray(statement.Action) &&
              statement.Action.some((action: string) => action.startsWith('ecr:')))
        )
      );

      expect(hasEcrPermissions).toBe(true);
    });
  });

  describe('CodeConnections', () => {
    test('creates GitHub connection with correct name', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'staging',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'develop',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CodeConnections::Connection', {
        ConnectionName: 'markmail-staging-github',
        ProviderType: 'GitHub',
      });
    });
  });

  describe('CodePipeline', () => {
    test('creates pipeline with correct name and stages', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CodePipeline::Pipeline', {
        Name: 'markmail-dev',
        Stages: [
          {
            Name: 'Source',
            Actions: [
              {
                Name: 'GitHub_Source',
                ActionTypeId: {
                  Category: 'Source',
                  Owner: 'AWS',
                  Provider: 'CodeStarSourceConnection',
                  Version: '1',
                },
                Configuration: {
                  ConnectionArn: Match.anyValue(),
                  FullRepositoryId: 'testowner/testrepo',
                  BranchName: 'main',
                  OutputArtifactFormat: 'CODEBUILD_CLONE_REF',
                },
              },
            ],
          },
          {
            Name: 'Build',
            Actions: Match.arrayWith([
              Match.objectLike({
                Name: 'Backend_Build',
                ActionTypeId: {
                  Category: 'Build',
                  Owner: 'AWS',
                  Provider: 'CodeBuild',
                },
              }),
              Match.objectLike({
                Name: 'Frontend_Build',
                ActionTypeId: {
                  Category: 'Build',
                  Owner: 'AWS',
                  Provider: 'CodeBuild',
                },
              }),
            ]),
          },
          {
            Name: 'Deploy',
            Actions: [
              {
                Name: 'Deploy_to_ECS',
                ActionTypeId: {
                  Category: 'Deploy',
                  Owner: 'AWS',
                  Provider: 'ECS',
                  Version: '1',
                },
              },
            ],
          },
        ],
      });
    });

    test('grants CodeConnections permissions to pipeline role', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Find the pipeline role policy
      const resources = template.toJSON().Resources;
      const pipelineRolePolicy = Object.values(resources).find(
        (r: any) =>
          r.Type === 'AWS::IAM::Policy' &&
          r.Properties?.PolicyDocument?.Statement?.some(
            (s: any) => s.Action === 'codeconnections:UseConnection'
          )
      );

      expect(pipelineRolePolicy).toBeDefined();
    });

    test('uses artifact bucket for pipeline', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CodePipeline::Pipeline', {
        ArtifactStore: {
          Type: 'S3',
          Location: Match.anyValue(),
        },
      });
    });
  });

  describe('Build Specs', () => {
    test('backend build spec includes Docker commands', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      // Find backend build project
      const resources = template.toJSON().Resources;
      const backendProject = Object.values(resources).find(
        (r: any) =>
          r.Type === 'AWS::CodeBuild::Project' &&
          r.Properties?.Name === 'markmail-dev-backend-build'
      );

      expect(backendProject).toBeDefined();
      const buildSpec = JSON.parse((backendProject as any).Properties.Source.BuildSpec);

      expect(buildSpec.phases.build.commands).toContain('cd backend');
      expect(buildSpec.phases.build.commands).toContainEqual(
        expect.stringContaining('docker build')
      );
      expect(buildSpec.phases.post_build.commands).toContainEqual(
        expect.stringContaining('docker push')
      );
      expect(buildSpec.artifacts.files).toContain('imagedefinitions.json');
    });

    test('frontend build spec includes VITE_API_URL build arg', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
        domainName: 'app.example.com',
      });

      // Assert
      const template = Template.fromStack(stack);

      // Find frontend build project
      const resources = template.toJSON().Resources;
      const frontendProject = Object.values(resources).find(
        (r: any) =>
          r.Type === 'AWS::CodeBuild::Project' &&
          r.Properties?.Name === 'markmail-dev-frontend-build'
      );

      expect(frontendProject).toBeDefined();
      const buildSpec = JSON.parse((frontendProject as any).Properties.Source.BuildSpec);

      expect(buildSpec.phases.build.commands).toContain('cd frontend');
      expect(buildSpec.phases.build.commands).toContainEqual(
        expect.stringContaining('docker build --build-arg VITE_API_URL=$VITE_API_URL')
      );
    });
  });

  describe('ECS Deploy Action', () => {
    test('configures ECS deploy action with correct service', () => {
      // Arrange & Act
      new CICDConstruct(stack, 'CICD', {
        environmentName: 'prod',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      const template = Template.fromStack(stack);

      template.hasResourceProperties('AWS::CodePipeline::Pipeline', {
        Stages: Match.arrayWith([
          Match.objectLike({
            Name: 'Deploy',
            Actions: [
              {
                Name: 'Deploy_to_ECS',
                Configuration: {
                  ClusterName: Match.anyValue(),
                  ServiceName: Match.anyValue(),
                },
              },
            ],
          }),
        ]),
      });
    });
  });

  describe('Construct Properties', () => {
    test('exposes all required properties', () => {
      // Arrange & Act
      const cicd = new CICDConstruct(stack, 'CICD', {
        environmentName: 'dev',
        githubOwner: 'testowner',
        githubRepo: 'testrepo',
        githubBranch: 'main',
        backendRepo,
        frontendRepo,
        fargateService,
      });

      // Assert
      expect(cicd.pipeline).toBeDefined();
      expect(cicd.githubConnection).toBeDefined();
    });
  });
});
