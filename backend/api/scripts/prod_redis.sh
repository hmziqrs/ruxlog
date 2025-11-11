#!/bin/bash

# Path to your env and acl files
ENV_FILE=".env.prod"
ACL_FILE="docker/redis/prod.acl"

# Check if .env.docker exists
if [ ! -f "$ENV_FILE" ]; then
    echo "Error: $ENV_FILE not found"
    exit 1
fi

# Source the env file to get variables
source "$ENV_FILE"

# Create directory if it doesn't exist
mkdir -p "$(dirname "$ACL_FILE")"

# Generate the ACL file
cat > "$ACL_FILE" << EOF
user default off
user ${REDIS_USER} on >${REDIS_PASSWORD} ~* &* +@all
EOF

echo "Generated Redis ACL file at $ACL_FILE"
