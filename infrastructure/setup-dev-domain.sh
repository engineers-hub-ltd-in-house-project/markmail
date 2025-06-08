#!/bin/bash

# Development environment with domain configuration
export CDK_CONTEXT_ENVIRONMENT="dev"
export DEV_DOMAIN="dev.markmail.engineers-hub.ltd"
export NOTIFICATION_EMAIL="admin@engineers-hub.ltd"
export GITHUB_OWNER="engineers-hub-ltd-in-house-project"
export GITHUB_REPO="markmail"
export GITHUB_BRANCH="develop"
export AWS_PROFILE="yusuke.sato"

echo "Development environment configured:"
echo "  Domain: $DEV_DOMAIN"
echo "  Environment: $CDK_CONTEXT_ENVIRONMENT"
echo "  AWS Profile: $AWS_PROFILE"
echo ""
echo "To deploy the Route 53 stack:"
echo "  npx cdk deploy MarkMail-dev-Route53Stack"
echo ""
echo "After deployment, configure the NS records in Squarespace:"
echo "  1. Get the nameservers from the stack output"
echo "  2. In Squarespace, add NS records for 'dev.markmail' subdomain"
echo "  3. Point them to the Route 53 nameservers"