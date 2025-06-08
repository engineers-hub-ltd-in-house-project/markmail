#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Environment
ENVIRONMENT=${1:-dev}
STACK_NUMBER=${2}

# Stack names in deployment order
STACKS=(
    "MarkMail-${ENVIRONMENT}-NetworkStack"
    "MarkMail-${ENVIRONMENT}-DatabaseStack"
    "MarkMail-${ENVIRONMENT}-ECRStack"
    "MarkMail-${ENVIRONMENT}-ECSClusterStack"
    "MarkMail-${ENVIRONMENT}-ALBStack"
    "MarkMail-${ENVIRONMENT}-ECSServiceStack"
    "MarkMail-${ENVIRONMENT}-CICDStack"
    "MarkMail-${ENVIRONMENT}-MonitoringStack"
)

# Function to check if a stack exists
stack_exists() {
    aws cloudformation describe-stacks --stack-name $1 >/dev/null 2>&1
}

# Display usage
if [ -z "$STACK_NUMBER" ]; then
    echo -e "${YELLOW}Usage: $0 <environment> <stack-number>${NC}"
    echo -e "${YELLOW}Available stacks:${NC}"
    for i in "${!STACKS[@]}"; do
        STACK_NAME="${STACKS[$i]}"
        if stack_exists "$STACK_NAME"; then
            echo -e "${GREEN}  $((i+1)). ${STACK_NAME} (deployed)${NC}"
        else
            echo -e "  $((i+1)). ${STACK_NAME}"
        fi
    done
    exit 1
fi

# Validate stack number
if [ "$STACK_NUMBER" -lt 1 ] || [ "$STACK_NUMBER" -gt 8 ]; then
    echo -e "${RED}Invalid stack number. Must be between 1 and 8.${NC}"
    exit 1
fi

# Get stack name
STACK_NAME="${STACKS[$((STACK_NUMBER-1))]}"

# Check if stack exists
if ! stack_exists "$STACK_NAME"; then
    echo -e "${YELLOW}Stack ${STACK_NAME} does not exist.${NC}"
    exit 0
fi

# Confirm deletion
echo -e "${YELLOW}You are about to destroy: ${STACK_NAME}${NC}"
echo -e "${RED}This action cannot be undone!${NC}"
read -p "Are you sure? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo -e "${YELLOW}Deletion cancelled.${NC}"
    exit 0
fi

# Delete the stack
echo -e "${YELLOW}Destroying ${STACK_NAME}...${NC}"
if npx cdk destroy ${STACK_NAME} --force; then
    echo -e "${GREEN}✓ ${STACK_NAME} destroyed successfully${NC}"
else
    echo -e "${RED}✗ Failed to destroy ${STACK_NAME}${NC}"
    exit 1
fi