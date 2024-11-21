#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration variables - CHANGE THESE
DB_USER="root"
DB_PASSWORD="root"
DB_NAME="blog"
REDIS_USERNAME="red"
REDIS_PASSWORD="red"

# Function to log messages
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"
}

warning() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    error "Please run as root"
    exit 1
fi

# 1. Update System
log "Updating system..."
apt update && apt upgrade -y

# 2. Install PostgreSQL and Redis
log "Installing PostgreSQL and Redis..."
apt install -y postgresql postgresql-contrib redis-server

# 3. Configure PostgreSQL
log "Configuring PostgreSQL..."
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';"
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

# 4. Configure Redis
log "Configuring Redis..."
# Backup original Redis configuration
cp /etc/redis/redis.conf /etc/redis/redis.conf.backup

# Create new Redis configuration
cat > /etc/redis/redis.conf << EOF
bind 127.0.0.1
port 6379
daemonize yes
supervised systemd
pidfile /var/run/redis/redis-server.pid
loglevel notice
logfile /var/log/redis/redis-server.log
databases 16
always-show-logo no
set-proc-title yes
proc-title-template "{title} {listen-addr} {server-mode}"
stop-writes-on-bgsave-error yes
rdbcompression yes
rdbchecksum yes
dbfilename dump.rdb
dir /var/lib/redis
maxmemory-policy noeviction
aclfile /etc/redis/users.acl
EOF

# Create ACL file for Redis users
cat > /etc/redis/users.acl << EOF
user default off
user $REDIS_USERNAME on >$REDIS_PASSWORD allcommands allkeys
EOF

# Set proper permissions
chown redis:redis /etc/redis/users.acl
chmod 640 /etc/redis/users.acl

# Restart Redis service
log "Restarting Redis..."
systemctl restart redis-server

# Verify Redis is running
if systemctl is-active --quiet redis-server; then
    log "Redis is running successfully"
else
    error "Redis failed to start"
    exit 1
fi

# Test Redis connection
log "Testing Redis connection..."
if redis-cli -u "redis://$REDIS_USERNAME:$REDIS_PASSWORD@127.0.0.1:6379" ping | grep -q "PONG"; then
    log "Redis authentication test successful"
else
    error "Redis authentication test failed"
    exit 1
fi

# Print summary
log "Installation complete! Summary of details:"
echo "-----------------------------------"
echo "Database Name: $DB_NAME"
echo "Database User: $DB_USER"
echo "Database Password: $DB_PASSWORD"
echo "Database Connection URL: postgres://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME"
echo ""
echo "Redis Host: 127.0.0.1"
echo "Redis Port: 6379"
echo "Redis Username: $REDIS_USERNAME"
echo "Redis Password: $REDIS_PASSWORD"
echo "Redis Connection URL: redis://$REDIS_USERNAME:$REDIS_PASSWORD@127.0.0.1:6379"
echo "-----------------------------------"

warning "Please save these credentials securely and then delete them from this script!"
warning "Redis is configured with username/password authentication"
