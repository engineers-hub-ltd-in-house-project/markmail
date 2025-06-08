#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Environment
ENVIRONMENT=${1:-dev}

echo -e "${YELLOW}Starting sequential deployment for environment: ${ENVIRONMENT}${NC}"

# Function to check if a stack exists
stack_exists() {
    aws cloudformation describe-stacks --stack-name $1 >/dev/null 2>&1
}

# Function to deploy a stack
deploy_stack() {
    local STACK_NAME=$1
    local STEP=$2
    local TOTAL=$3
    local DESCRIPTION=$4
    
    echo -e "\n${YELLOW}[$STEP/$TOTAL] Deploying ${STACK_NAME}${NC}"
    echo -e "${GREEN}Description: ${DESCRIPTION}${NC}"
    
    if npx cdk deploy ${STACK_NAME} --require-approval never; then
        echo -e "${GREEN}✓ ${STACK_NAME} deployed successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to deploy ${STACK_NAME}${NC}"
        return 1
    fi
}

# Function to build and push Docker images
build_and_push_images() {
    echo -e "\n${YELLOW}Building and pushing Docker images...${NC}"
    
    # Get ECR repository URIs from CloudFormation outputs
    BACKEND_REPO_URI=$(aws cloudformation describe-stacks \
        --stack-name MarkMail-${ENVIRONMENT}-ECRStack \
        --query "Stacks[0].Outputs[?OutputKey=='BackendRepositoryUri'].OutputValue" \
        --output text)
    
    FRONTEND_REPO_URI=$(aws cloudformation describe-stacks \
        --stack-name MarkMail-${ENVIRONMENT}-ECRStack \
        --query "Stacks[0].Outputs[?OutputKey=='FrontendRepositoryUri'].OutputValue" \
        --output text)
    
    if [ -z "$BACKEND_REPO_URI" ] || [ -z "$FRONTEND_REPO_URI" ]; then
        echo -e "${RED}Failed to get ECR repository URIs${NC}"
        return 1
    fi
    
    # Login to ECR
    echo -e "${GREEN}Logging in to ECR...${NC}"
    aws ecr get-login-password --region ${AWS_DEFAULT_REGION:-ap-northeast-1} | \
        docker login --username AWS --password-stdin \
        $(echo $BACKEND_REPO_URI | cut -d'/' -f1)
    
    # Build and push backend
    echo -e "${GREEN}Building backend image...${NC}"
    cd ../backend
    docker build -t ${BACKEND_REPO_URI}:latest .
    docker push ${BACKEND_REPO_URI}:latest
    
    # Build and push frontend
    echo -e "${GREEN}Building frontend image...${NC}"
    cd ../frontend
    
    # Set API URL based on environment
    if [ "$ENVIRONMENT" = "prod" ] && [ ! -z "$PROD_DOMAIN" ]; then
        API_URL="https://${PROD_DOMAIN}/api"
    elif [ "$ENVIRONMENT" = "staging" ] && [ ! -z "$STAGING_DOMAIN" ]; then
        API_URL="https://${STAGING_DOMAIN}/api"
    else
        API_URL="/api"
    fi
    
    docker build --build-arg VITE_API_URL=${API_URL} -t ${FRONTEND_REPO_URI}:latest .
    docker push ${FRONTEND_REPO_URI}:latest
    
    cd ../infrastructure
    
    echo -e "${GREEN}✓ Docker images built and pushed successfully${NC}"
    return 0
}

# Main deployment sequence
echo -e "${YELLOW}Synthesizing all stacks...${NC}"
npx cdk synth

# Deploy stacks in order
deploy_stack "MarkMail-${ENVIRONMENT}-NetworkStack" 1 8 "VPC and Security Groups" || exit 1
deploy_stack "MarkMail-${ENVIRONMENT}-DatabaseStack" 2 8 "RDS and ElastiCache" || exit 1
deploy_stack "MarkMail-${ENVIRONMENT}-ECRStack" 3 8 "ECR Repositories" || exit 1

# Build and push Docker images after ECR is ready
build_and_push_images || exit 1

deploy_stack "MarkMail-${ENVIRONMENT}-ECSClusterStack" 4 8 "ECS Cluster and IAM Roles" || exit 1
deploy_stack "MarkMail-${ENVIRONMENT}-ALBStack" 5 8 "Application Load Balancer" || exit 1
deploy_stack "MarkMail-${ENVIRONMENT}-ECSServiceStack" 6 8 "ECS Service and Tasks" || exit 1
deploy_stack "MarkMail-${ENVIRONMENT}-CICDStack" 7 8 "CI/CD Pipeline" || exit 1
deploy_stack "MarkMail-${ENVIRONMENT}-MonitoringStack" 8 8 "CloudWatch and SES" || exit 1

echo -e "\n${GREEN}✓ All stacks deployed successfully!${NC}"
echo -e "${YELLOW}ALB Endpoint:${NC}"
aws cloudformation describe-stacks \
    --stack-name MarkMail-${ENVIRONMENT}-ALBStack \
    --query "Stacks[0].Outputs[?OutputKey=='LoadBalancerDnsName'].OutputValue" \
    --output text

# After deployment, activate GitHub connection if needed
if stack_exists "MarkMail-${ENVIRONMENT}-CICDStack"; then
    CONNECTION_ARN=$(aws cloudformation describe-stacks \
        --stack-name MarkMail-${ENVIRONMENT}-CICDStack \
        --query "Stacks[0].Outputs[?OutputKey=='GitHubConnectionArn'].OutputValue" \
        --output text)
    
    echo -e "\n${YELLOW}GitHub Connection ARN: ${CONNECTION_ARN}${NC}"
    echo -e "${YELLOW}Please activate the GitHub connection in the AWS Console if not already done.${NC}"
fi