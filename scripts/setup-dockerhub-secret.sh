#!/bin/bash

# Docker Hub credentials setup script for AWS Secrets Manager
# IMPORTANT: Set environment variables before running this script
# export DOCKERHUB_USERNAME="your-username"
# export DOCKERHUB_TOKEN="your-access-token"

# Check if required environment variables are set
if [ -z "$DOCKERHUB_USERNAME" ] || [ -z "$DOCKERHUB_TOKEN" ]; then
    echo "Error: DOCKERHUB_USERNAME and DOCKERHUB_TOKEN environment variables must be set!"
    echo "Usage:"
    echo "  export DOCKERHUB_USERNAME='your-username'"
    echo "  export DOCKERHUB_TOKEN='your-access-token'"
    echo "  ./scripts/setup-dockerhub-secret.sh"
    exit 1
fi

# Set your AWS profile and region
AWS_PROFILE=${AWS_PROFILE:-"default"}
AWS_REGION=${AWS_REGION:-"ap-northeast-1"}

# Create secret for dev environment
echo "Creating Docker Hub secret for dev environment..."
aws secretsmanager create-secret \
    --name "markmail-dev-dockerhub" \
    --description "Docker Hub credentials for CodeBuild" \
    --secret-string "{\"username\":\"${DOCKERHUB_USERNAME}\",\"password\":\"${DOCKERHUB_TOKEN}\"}" \
    --region ${AWS_REGION} \
    --profile ${AWS_PROFILE} 2>/dev/null || \
aws secretsmanager update-secret \
    --secret-id "markmail-dev-dockerhub" \
    --secret-string "{\"username\":\"${DOCKERHUB_USERNAME}\",\"password\":\"${DOCKERHUB_TOKEN}\"}" \
    --region ${AWS_REGION} \
    --profile ${AWS_PROFILE}

# Create secret for prod environment (if needed)
echo "Creating Docker Hub secret for prod environment..."
aws secretsmanager create-secret \
    --name "markmail-prod-dockerhub" \
    --description "Docker Hub credentials for CodeBuild" \
    --secret-string "{\"username\":\"${DOCKERHUB_USERNAME}\",\"password\":\"${DOCKERHUB_TOKEN}\"}" \
    --region ${AWS_REGION} \
    --profile ${AWS_PROFILE} 2>/dev/null || \
aws secretsmanager update-secret \
    --secret-id "markmail-prod-dockerhub" \
    --secret-string "{\"username\":\"${DOCKERHUB_USERNAME}\",\"password\":\"${DOCKERHUB_TOKEN}\"}" \
    --region ${AWS_REGION} \
    --profile ${AWS_PROFILE}

echo "Docker Hub secrets have been created/updated successfully!"