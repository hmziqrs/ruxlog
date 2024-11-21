#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration variables - CHANGE THESE
DB_USER="badmin"
DB_PASSWORD="root"
DB_NAME="blog"
REDIS_USERNAME="red"
REDIS_PASSWORD="red"

# Temporary files and backup locations
TEMP_DIR="/tmp/db_setup_backup"
REDIS_CONF_BACKUP="/etc/redis/redis.conf.backup"

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

# Cleanup function
cleanup() {
    if [ $? -ne 0 ]; then
        error "An error occurred during installation. Rolling back changes..."

        # Restore Redis configuration if backup exists
        if [ -f "$REDIS_CONF_BACKUP" ]; then
            log "Restoring Redis configuration..."
            mv "$REDIS_CONF_BACKUP" /etc/redis/redis.conf
            systemctl restart redis-server || true
        fi

        # Drop PostgreSQL database and user if they exist
        if sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
            log "Removing PostgreSQL database..."
            sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"
        fi

        if sudo -u postgres psql -t -c "\du" | cut -d \| -f 1 | grep -qw "$DB_USER"; then
            log "Removing PostgreSQL user..."
            sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;"
        fi

        # Remove temporary directory
        if [ -d "$TEMP_DIR" ]; then
            rm -rf "$TEMP_DIR"
        fi

        error "Rollback complete. Please check the logs and try again."
    else
        # Cleanup successful installation
        [ -f "$REDIS_CONF_BACKUP" ] && rm "$REDIS_CONF_BACKUP"
        [ -d "$TEMP_DIR" ] && rm -rf "$TEMP_DIR"
        log "Cleanup completed successfully"
    fi
}

# Register cleanup function to run on script exit
trap cleanup EXIT

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    error "Please run as root"
    exit 1
fi

# Create temporary directory for backups
mkdir -p "$TEMP_DIR"

# Verify configuration variables
if [[ -z "$DB_USER" ]] || [[ -z "$DB_PASSWORD" ]] || [[ -z "$DB_NAME" ]] || [[ -z "$REDIS_USERNAME" ]] || [[ -z "$REDIS_PASSWORD" ]]; then
    error "Configuration variables cannot be empty"
    exit 1
fi

# 1. Update System
log "Updating system..."
apt update && apt upgrade -y || {
    error "System update failed"
    exit 1
}

# 2. Install PostgreSQL and Redis
log "Installing PostgreSQL and Redis..."
apt install -y postgresql postgresql-contrib redis-server || {
    error "Failed to install PostgreSQL and Redis"
    exit 1
}

# 3. Verify PostgreSQL is running
if ! systemctl is-active --quiet postgresql; then
    error "PostgreSQL is not running"
    exit 1
fi

# 4. Configure PostgreSQL
log "Configuring PostgreSQL..."
# Check if database already exists
if sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    error "Database $DB_NAME already exists"
    exit 1
fi

# Check if user already exists
if sudo -u postgres psql -t -c "\du" | cut -d \| -f 1 | grep -qw "$DB_USER"; then
    error "User $DB_USER already exists"
    exit 1
fi

# Create user and database
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" || {
    error "Failed to create PostgreSQL user"
    exit 1
}

sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;" || {
    error "Failed to create PostgreSQL database"
    exit 1
}

# 5. Configure Redis
log "Configuring Redis..."
# Backup original Redis configuration
cp /etc/redis/redis.conf "$REDIS_CONF_BACKUP" || {
    error "Failed to backup Redis configuration"
    exit 1
}

# Create new Redis configuration
cat > /etc/redis/redis.conf << EOF || exit 1
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
cat > /etc/redis/users.acl << EOF || exit 1
user default off
user $REDIS_USERNAME on >$REDIS_PASSWORD allcommands allkeys
EOF

# Set proper permissions
chown redis:redis /etc/redis/users.acl || exit 1
chmod 640 /etc/redis/users.acl || exit 1

# Restart Redis service
log "Restarting Redis..."
systemctl restart redis-server || {
    error "Failed to restart Redis"
    exit 1
}

# Wait for Redis to start
sleep 2

# Verify Redis is running
if ! systemctl is-active --quiet redis-server; then
    error "Redis failed to start"
    exit 1
fi

# Test Redis connection
log "Testing Redis connection..."
if ! redis-cli -u "redis://$REDIS_USERNAME:$REDIS_PASSWORD@127.0.0.1:6379" ping | grep -q "PONG"; then
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
