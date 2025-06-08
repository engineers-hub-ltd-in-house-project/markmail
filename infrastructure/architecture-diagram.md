# MarkMail AWS Infrastructure Architecture

## Overview

This document contains a comprehensive Mermaid diagram showing the complete AWS infrastructure for the MarkMail application, including all resources, their relationships, and network architecture.

## Architecture Diagram

```mermaid
graph TB
    %% External Users and Services
    Internet((Internet))
    GitHub[GitHub Repository]
    Developer[Developer]

    %% AWS Account and Region
    subgraph AWS["AWS Account"]
        %% Network Stack
        subgraph VPC["VPC (markmail-{env}-vpc)"]
            %% Availability Zones
            subgraph AZ1["Availability Zone 1"]
                PublicSubnet1[Public Subnet 1<br/>CIDR: /24]
                PrivateSubnet1[Private Subnet 1<br/>CIDR: /24]
                IsolatedSubnet1[Isolated Subnet 1<br/>CIDR: /24]
            end

            subgraph AZ2["Availability Zone 2"]
                PublicSubnet2[Public Subnet 2<br/>CIDR: /24]
                PrivateSubnet2[Private Subnet 2<br/>CIDR: /24]
                IsolatedSubnet2[Isolated Subnet 2<br/>CIDR: /24]
            end

            %% Network Components
            IGW[Internet Gateway]
            NAT1[NAT Gateway 1]
            NAT2[NAT Gateway 2<br/>prod only]

            %% Security Groups
            ALBSecurityGroup[ALB Security Group<br/>Inbound: 80, 443<br/>Outbound: All]
            ECSSecurityGroup[ECS Security Group<br/>Inbound: 8080 from ALB<br/>Outbound: All]
            RDSSecurityGroup[RDS Security Group<br/>Inbound: 5432 from ECS<br/>Outbound: None]
            CacheSecurityGroup[Cache Security Group<br/>Inbound: 6379 from ECS<br/>Outbound: None]
        end

        %% ECR Stack
        subgraph ECR["Elastic Container Registry"]
            BackendECR[Backend Repository<br/>markmail-{env}-backend<br/>Lifecycle: 10 images]
            FrontendECR[Frontend Repository<br/>markmail-{env}-frontend<br/>Lifecycle: 10 images]
        end

        %% ALB Stack
        subgraph ALBLayer["Load Balancing Layer"]
            ALB[Application Load Balancer<br/>markmail-{env}-alb<br/>Internet Facing]
            HttpListener[HTTP Listener<br/>Port 80]
            HttpsListener[HTTPS Listener<br/>Port 443<br/>if domain configured]
            Certificate[ACM Certificate<br/>Domain + *.Domain]
            BackendTargetGroup[Backend Target Group<br/>Port 8080<br/>Health: /health]
            FrontendTargetGroup[Frontend Target Group<br/>Port 8080<br/>Health: /]
        end

        %% ECS Cluster Stack
        subgraph ECSCluster["ECS Cluster (markmail-{env})"]
            ClusterConfig[Cluster<br/>Container Insights: prod only]
            TaskExecutionRole[Task Execution Role<br/>ECR Pull<br/>Secrets Manager<br/>CloudWatch Logs]
            TaskRole[Task Role<br/>SES Permissions]
            LogGroup[CloudWatch Log Group<br/>/ecs/markmail-{env}<br/>Retention: 30 days]
        end

        %% ECS Service Stack
        subgraph ECSServices["ECS Services"]
            %% Backend Service
            subgraph BackendService["Backend Service"]
                BackendTaskDef[Backend Task Definition<br/>CPU: 512<br/>Memory: 1024]
                BackendContainer[Backend Container<br/>Port: 8080<br/>Health Check: curl /health]
                BackendFargate[Fargate Service<br/>markmail-{env}-backend<br/>Desired: 2<br/>Auto Scaling: 2-4]
            end

            %% Frontend Service
            subgraph FrontendService["Frontend Service"]
                FrontendTaskDef[Frontend Task Definition<br/>CPU: 256<br/>Memory: 512]
                FrontendContainer[Frontend Container<br/>Port: 8080<br/>Health Check: wget /]
                FrontendFargate[Fargate Service<br/>markmail-{env}-frontend<br/>Desired: 2<br/>Auto Scaling: 2-4]
            end
        end

        %% Database Stack
        subgraph DatabaseLayer["Database Layer"]
            DBSecret[Secrets Manager<br/>markmail-{env}-db-secret]
            RDS[RDS PostgreSQL 15<br/>markmail-{env}-db<br/>t3.micro/20-100GB<br/>Multi-AZ: prod only]
            ElastiCache[ElastiCache Redis<br/>markmail-{env}-cache<br/>t3.micro/r7g.large]
        end

        %% Monitoring Stack
        subgraph MonitoringLayer["Monitoring & Alerting"]
            Dashboard[CloudWatch Dashboard<br/>markmail-{env}]
            SNSTopic[SNS Topic<br/>MarkMail {env} Alarms]
            EmailSubscription[Email Subscription]

            %% Alarms
            BackendCPUAlarm[Backend CPU Alarm<br/>> 80%]
            BackendMemoryAlarm[Backend Memory Alarm<br/>> 80%]
            FrontendCPUAlarm[Frontend CPU Alarm<br/>> 80%]
            FrontendMemoryAlarm[Frontend Memory Alarm<br/>> 80%]
            ResponseTimeAlarm[ALB Response Time Alarm<br/>> 1s]

            %% SES
            SESConfigSet[SES Configuration Set<br/>markmail-{env}]
            SESDomain[SES Domain Identity<br/>if domain configured]
        end

        %% CI/CD Stack
        subgraph CICDPipeline["CI/CD Pipeline"]
            GitHubConnection[GitHub Connection<br/>markmail-{env}-github]

            subgraph Pipeline["CodePipeline (markmail-{env})"]
                SourceStage[Source Stage<br/>GitHub Source Action]

                subgraph BuildStage["Build Stage"]
                    BackendBuild[Backend Build<br/>CodeBuild Project<br/>Docker Build & Push]
                    FrontendBuild[Frontend Build<br/>CodeBuild Project<br/>Docker Build & Push]
                end

                subgraph DeployStage["Deploy Stage"]
                    BackendDeploy[Deploy Backend<br/>ECS Deploy Action]
                    FrontendDeploy[Deploy Frontend<br/>ECS Deploy Action]
                end
            end
        end

        %% Route 53 (if domain configured)
        Route53[Route 53<br/>A Record → ALB<br/>if domain configured]
    end

    %% Connections - Network Flow
    Internet --> IGW
    IGW --> PublicSubnet1
    IGW --> PublicSubnet2
    PublicSubnet1 --> NAT1
    PublicSubnet2 --> NAT2
    NAT1 --> PrivateSubnet1
    NAT2 --> PrivateSubnet2

    %% Connections - ALB
    Internet --> ALB
    ALB --> ALBSecurityGroup
    ALB --> HttpListener
    ALB --> HttpsListener
    HttpsListener --> Certificate
    HttpListener --> BackendTargetGroup
    HttpListener --> FrontendTargetGroup
    HttpsListener --> BackendTargetGroup
    HttpsListener --> FrontendTargetGroup

    %% Connections - ECS Services
    BackendTargetGroup --> BackendFargate
    FrontendTargetGroup --> FrontendFargate
    BackendFargate --> BackendContainer
    FrontendFargate --> FrontendContainer
    BackendContainer --> ECSSecurityGroup
    FrontendContainer --> ECSSecurityGroup

    %% Connections - Backend Dependencies
    BackendContainer --> RDS
    BackendContainer --> ElastiCache
    BackendContainer --> DBSecret
    BackendContainer --> SESConfigSet
    RDS --> RDSSecurityGroup
    ElastiCache --> CacheSecurityGroup
    RDS --> IsolatedSubnet1
    RDS --> IsolatedSubnet2
    ElastiCache --> IsolatedSubnet1
    ElastiCache --> IsolatedSubnet2

    %% Connections - Container Images
    BackendContainer --> BackendECR
    FrontendContainer --> FrontendECR

    %% Connections - Logs
    BackendContainer --> LogGroup
    FrontendContainer --> LogGroup

    %% Connections - Monitoring
    BackendFargate --> BackendCPUAlarm
    BackendFargate --> BackendMemoryAlarm
    FrontendFargate --> FrontendCPUAlarm
    FrontendFargate --> FrontendMemoryAlarm
    ALB --> ResponseTimeAlarm
    BackendCPUAlarm --> SNSTopic
    BackendMemoryAlarm --> SNSTopic
    FrontendCPUAlarm --> SNSTopic
    FrontendMemoryAlarm --> SNSTopic
    ResponseTimeAlarm --> SNSTopic
    SNSTopic --> EmailSubscription

    %% Connections - CI/CD
    GitHub --> GitHubConnection
    GitHubConnection --> SourceStage
    SourceStage --> BuildStage
    BackendBuild --> BackendECR
    FrontendBuild --> FrontendECR
    BuildStage --> DeployStage
    BackendDeploy --> BackendFargate
    FrontendDeploy --> FrontendFargate

    %% Connections - DNS
    Route53 --> ALB

    %% Connections - Developer Access
    Developer --> AWS

    %% Styling
    classDef network fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef compute fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef database fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px
    classDef storage fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef security fill:#fce4ec,stroke:#880e4f,stroke-width:2px
    classDef monitoring fill:#f1f8e9,stroke:#33691e,stroke-width:2px
    classDef cicd fill:#e3f2fd,stroke:#0d47a1,stroke-width:2px

    class VPC,PublicSubnet1,PublicSubnet2,PrivateSubnet1,PrivateSubnet2,IsolatedSubnet1,IsolatedSubnet2,IGW,NAT1,NAT2 network
    class ECSCluster,BackendService,FrontendService,BackendFargate,FrontendFargate,ALB compute
    class RDS,ElastiCache,DBSecret database
    class BackendECR,FrontendECR,LogGroup storage
    class ALBSecurityGroup,ECSSecurityGroup,RDSSecurityGroup,CacheSecurityGroup,TaskExecutionRole,TaskRole security
    class Dashboard,SNSTopic,BackendCPUAlarm,BackendMemoryAlarm,FrontendCPUAlarm,FrontendMemoryAlarm,ResponseTimeAlarm monitoring
    class Pipeline,GitHubConnection,BackendBuild,FrontendBuild cicd
```

## Key Architecture Components

### 1. Network Architecture

- **VPC**: Multi-AZ deployment with 3 subnet tiers (Public, Private, Isolated)
- **NAT Gateways**: 1 for dev/staging, 2 for production (HA)
- **Security Groups**: Strict ingress rules following least privilege principle

### 2. Container Infrastructure

- **ECS on Fargate**: Serverless container hosting
- **ECR**: Private container registries with lifecycle policies
- **Auto Scaling**: CPU and Memory based scaling (2-4 tasks)

### 3. Database Layer

- **RDS PostgreSQL**: Multi-AZ for production, encrypted storage
- **ElastiCache Redis**: Session/cache storage
- **Secrets Manager**: Secure credential management

### 4. Load Balancing & Routing

- **Application Load Balancer**: Internet-facing with path-based routing
- **Target Groups**: Separate groups for backend API (/api/_) and frontend (/_)
- **HTTPS**: Optional SSL/TLS termination with ACM certificates

### 5. CI/CD Pipeline

- **CodePipeline**: Automated deployment pipeline
- **CodeBuild**: Docker image building
- **GitHub Integration**: Source control integration via CodeStar Connections

### 6. Monitoring & Alerting

- **CloudWatch**: Dashboards, logs, and metrics
- **SNS**: Email notifications for alarms
- **Alarms**: CPU, Memory, and Response Time monitoring
- **SES**: Email service configuration for application emails

## Environment-Specific Configurations

### Development/Staging

- Single NAT Gateway
- No Container Insights
- 7-day backup retention
- t3.micro instances

### Production

- Dual NAT Gateways (HA)
- Container Insights enabled
- 30-day backup retention
- Multi-AZ RDS deployment
- Larger cache instances (r7g.large)

## Security Features

- Private subnets for compute resources
- Isolated subnets for databases
- Encrypted data at rest (RDS)
- Secrets Manager for credentials
- IAM roles with minimal permissions
- Security group chaining
