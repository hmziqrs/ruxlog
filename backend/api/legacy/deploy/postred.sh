#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration variables
DB_USER="badmin"
DB_PASSWORD="root"
DB_NAME="blog"
REDIS_USERNAME="red"
REDIS_PASSWORD="red"
POSTGRES_VERSION="16"  # Updated to latest stable version

# Logging functions
log() { echo -e "${GREEN}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"; }
error() { echo -e "${RED}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"; }
warning() { echo -e "${YELLOW}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"; }

# Function to check if PostgreSQL is installed and running
check_postgres_status() {
    if ! command -v psql >/dev/null 2>&1; then
        error "PostgreSQL is not installed"
        return 1
    fi

    if ! systemctl is-active --quiet postgresql; then
        error "PostgreSQL service is not running"
        return 1
    fi

    return 0
}

# Function to setup PostgreSQL
setup_postgres() {
    log "Setting up PostgreSQL..."

    # Install PostgreSQL repository script
    if ! command -v postgresql-common >/dev/null 2>&1; then
        log "Installing postgresql-common..."
        apt install -y postgresql-common || return 1

        log "Adding PostgreSQL repository..."
        /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y || return 1
    fi

    # Install PostgreSQL
    log "Installing PostgreSQL..."
    apt install -y postgresql postgresql-contrib || return 1

    # Ensure PostgreSQL is started
    systemctl start postgresql
    systemctl enable postgresql

    # Wait for PostgreSQL to be ready
    sleep 3

    # Configure PostgreSQL authentication
    log "Configuring PostgreSQL authentication..."
    PG_HBA_CONF="/etc/postgresql/$POSTGRES_VERSION/main/pg_hba.conf"

    # Backup original configuration
    cp "$PG_HBA_CONF" "${PG_HBA_CONF}.backup"

    # Append new rules for badmin
    echo "
# Custom rules for badmin
host    all             $DB_USER             127.0.0.1/32            scram-sha-256
host    all             $DB_USER             ::1/128                 scram-sha-256
" >> "$PG_HBA_CONF"

    # Restart PostgreSQL to apply changes
    systemctl restart postgresql
    sleep 3

    return 0
}

# Function to create database and user
setup_database() {
    log "Setting up database and user..."

    # Set password for postgres user first
    sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'postgres';" || {
        error "Failed to set postgres user password"
        return 1
    }

    # Create user if not exists
    if ! sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1; then
        sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" || {
            error "Failed to create database user"
            return 1
        }
    else
        warning "User $DB_USER already exists"
    fi

    # Create database if not exists
    if ! sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
        sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;" || {
            error "Failed to create database"
            return 1
        }
    else
        warning "Database $DB_NAME already exists"
    fi

    # Grant privileges
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;" || {
        error "Failed to grant privileges"
        return 1
    }

    return 0
}

# Function to setup Redis
setup_redis() {
    log "Setting up Redis..."

    # Install Redis if not present
    if ! command -v redis-server >/dev/null 2>&1; then
        apt install -y redis-server || return 1
    fi

    # Backup existing Redis configuration
    if [ -f /etc/redis/redis.conf ]; then
        cp /etc/redis/redis.conf /etc/redis/redis.conf.backup
    fi

    # Configure Redis
    cat > /etc/redis/redis.conf << EOF
bind 127.0.0.1
port 6379
daemonize yes
supervised systemd
pidfile /var/run/redis/redis-server.pid
loglevel notice
logfile /var/log/redis/redis-server.log
databases 16
maxmemory-policy noeviction
aclfile /etc/redis/users.acl
EOF

    # Configure Redis ACL
    cat > /etc/redis/users.acl << EOF
user default off
user ${REDIS_USERNAME} on >${REDIS_PASSWORD} allcommands allkeys
EOF

    # Set proper permissions
    chown redis:redis /etc/redis/users.acl
    chmod 640 /etc/redis/users.acl

    # Restart Redis
    systemctl restart redis-server
    systemctl enable redis-server

    # Wait for Redis to start
    sleep 2

    # Test Redis connection
    if ! redis-cli -u "redis://${REDIS_USERNAME}:${REDIS_PASSWORD}@127.0.0.1:6379" ping | grep -q "PONG"; then
        error "Redis connection test failed"
        return 1
    fi

    return 0
}

# Main installation function
main() {
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        error "Please run as root"
        exit 1
    fi

    # Update system
    log "Updating system..."
    apt update || {
        error "Failed to update system"
        exit 1
    }

    # Setup PostgreSQL
    if ! check_postgres_status; then
        setup_postgres || {
            error "Failed to setup PostgreSQL"
            exit 1
        }
    fi

    # Setup database and user
    setup_database || {
        error "Failed to setup database"
        exit 1
    }

    # Setup Redis
    setup_redis || {
        error "Failed to setup Redis"
        exit 1
    }

    # Print connection details
    log "Setup completed successfully!"
    echo "-----------------------------------"
    echo "PostgreSQL Details:"
    echo "Database Name: $DB_NAME"
    echo "Database User: $DB_USER"
    echo "Database Password: $DB_PASSWORD"
    echo "Connection URL: postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME"
    echo ""
    echo "Redis Details:"
    echo "Host: 127.0.0.1"
    echo "Port: 6379"
    echo "Username: $REDIS_USERNAME"
    echo "Password: $REDIS_PASSWORD"
    echo "Connection URL: redis://$REDIS_USERNAME:$REDIS_PASSWORD@127.0.0.1:6379"
    echo "-----------------------------------"
}

# Run main function
main
