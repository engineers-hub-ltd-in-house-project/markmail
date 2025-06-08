#!/bin/bash

# Production environment with domain configuration
export CDK_CONTEXT_ENVIRONMENT="prod"
export PROD_DOMAIN="markmail.engineers-hub.ltd"
export NOTIFICATION_EMAIL="admin@engineers-hub.ltd"
export GITHUB_OWNER="engineers-hub-ltd-in-house-project"
export GITHUB_REPO="markmail"
export AWS_PROFILE="yusuke.sato"

echo "Production environment configured:"
echo "  Domain: $PROD_DOMAIN"
echo "  Environment: $CDK_CONTEXT_ENVIRONMENT"
echo "  AWS Profile: $AWS_PROFILE"
echo ""
echo "To deploy the Route 53 stack:"
echo "  npx cdk deploy MarkMail-prod-Route53Stack"
echo ""
echo "After deployment, configure the NS records in Squarespace:"
echo "  1. Get the nameservers from the stack output"
echo "  2. In Squarespace, add NS records for 'markmail' subdomain"
echo "  3. Point them to the Route 53 nameservers"