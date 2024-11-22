#!/bin/bash

# Exit on any error
set -e

source $HOME/.bashrc

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Configuration variables - CHANGE THESE
DOMAIN="hmziqrs.com"
API_SUBDOMAIN="blog-api.$DOMAIN"

DB_USER="badmin"
DB_PASSWORD="root"
DB_NAME="blog"

REDIS_USERNAME="red"
REDIS_PASSWORD="red"

SMTP_HOST="0.0.0.0"
SMTP_USERNAME="sam"
SMTP_PASSWORD="sam"
SMTP_PORT="587"
APP_USER=$USER
APP_DIR="/home/$APP_USER/apps/blog"
REPO_URL="https://github.com/hmziqrs/ruxlog-backend.git"
PROGRESS_FILE="/var/log/setup_progress.txt"

# Function to log messages
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $1${NC}"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    error "Please run as root"
    exit 1
fi


# Function to detect distribution
detect_distribution() {
    if [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/redhat-release ]; then
        echo "redhat"
    else
        echo "unknown"
    fi
}

# Function to check if a step is done
is_step_done() {
    grep -q "$1" "$PROGRESS_FILE"
}

# Ensure progress file exists
touch "$PROGRESS_FILE"

# 1. Update System
if ! is_step_done "system_updated"; then
    log "Updating system..."
    apt update && apt upgrade -y
    echo "system_updated" >> "$PROGRESS_FILE"
fi

# 2. Install Essential Packages
if ! is_step_done "packages_installed"; then
    log "Installing essential packages..."
    apt install -y build-essential curl git pkg-config libssl-dev postgresql libpq-dev postgresql-contrib redis-server nginx fail2ban htop ufw
    echo "packages_installed" >> "$PROGRESS_FILE"
fi

# 3. Configure PostgreSQL
if ! is_step_done "postgresql_configured"; then
    log "Configuring PostgreSQL..."

    # Check if PostgreSQL is installed
    if ! dpkg -l | grep -q postgresql; then
        error "PostgreSQL is not installed."
        exit 1
    fi

    DISTRO=$(detect_distribution)

    if [ "$DISTRO" = "debian" ]; then
        PG_VERSION=$(psql --version | grep -oP '^\d+')
        PG_DATA_DIR="/var/lib/postgresql/$PG_VERSION/main"
        if [ ! -d "$PG_DATA_DIR" ]; then
            sudo pg_createcluster $PG_VERSION main --start
        fi
    elif [ "$DISTRO" = "redhat" ]; then
        PG_DATA_DIR="/var/lib/pgsql/data"
        if [ ! -d "$PG_DATA_DIR" ]; then
            sudo postgresql-setup initdb
        fi
    else
        error "Unsupported distribution."
        exit 1
    fi

    # Ensure PostgreSQL service is enabled and started
    sudo systemctl enable postgresql
    sudo systemctl start postgresql

    # Create the database user if it doesn't exist
    if ! sudo -u postgres psql -t -c "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'"; then
        sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';"
    fi

    # Create the database if it doesn't exist
    if ! sudo -u postgres psql -t -c "SELECT 1 FROM pg_database WHERE datname='$DB_NAME'"; then
        sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"
    fi

    echo "postgresql_configured" >> "$PROGRESS_FILE"
fi

# 4. Configure Firewall
if ! is_step_done "firewall_configured"; then
    log "Configuring firewall..."
    ufw allow ssh
    ufw allow 'Nginx Full'
    ufw --force enable
    echo "firewall_configured" >> "$PROGRESS_FILE"
fi

# 5. Configure Redis
if ! is_step_done "redis_configured"; then
    log "Configuring Redis..."
    # Backup original Redis configuration
    if [ ! -f /etc/redis/redis.conf.backup ]; then
        cp /etc/redis/redis.conf /etc/redis/redis.conf.backup
    fi
    # Ensure Redis configuration is set
    if ! grep -q "aclfile /etc/redis/users.acl" /etc/redis/redis.conf; then
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
    fi
    # Ensure Redis ACL file is set
    if [ ! -f /etc/redis/users.acl ]; then
        cat > /etc/redis/users.acl << EOF
user default off
user $REDIS_USERNAME on >$REDIS_PASSWORD allcommands allkeys
EOF
        chown redis:redis /etc/redis/users.acl
        chmod 640 /etc/redis/users.acl
    fi
    systemctl restart redis-server
    # Test Redis connection
    if ! redis-cli -u "redis://$REDIS_USERNAME:$REDIS_PASSWORD@127.0.0.1:6379" ping | grep -q "PONG"; then
        error "Redis authentication test failed"
        exit 1
    fi
    echo "redis_configured" >> "$PROGRESS_FILE"
fi

# 6. Install Rust and tools
if ! is_step_done "rust_and_tools_installed"; then
    log "Installing Rust..."
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi
    log "Installing Diesel CLI..."
    cargo install diesel_cli --no-default-features --features postgres
    echo "rust_and_tools_installed" >> "$PROGRESS_FILE"
fi

# 7. Set up Application Directory
if ! is_step_done "app_directory_setup"; then
    log "Setting up application directory..."
    mkdir -p $APP_DIR
    cd $APP_DIR
    if [ ! -d ".git" ]; then
        git clone $REPO_URL .
    fi
    echo "app_directory_setup" >> "$PROGRESS_FILE"
fi

# 8. Create .env file
if ! is_step_done "env_file_created"; then
    log "Creating .env file..."
    cat > $APP_DIR/.env << EOF
HOST=127.0.0.1
PORT=8888

# Database
POSTGRE_DB_URL=postgres://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME
DATABASE_URL=postgres://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME
RUST_LOG=diesel_logger=debug

# Redis
REDIS_USERNAME=$REDIS_USERNAME
REDIS_PASSWORD=$REDIS_PASSWORD
REDIS_HOST=127.0.0.1
REDIS_PORT=6379

# SMTP
SMTP_HOST=$SMTP_HOST
SMTP_USERNAME=$SMTP_USERNAME
SMTP_PASSWORD=$SMTP_PASSWORD
SMTP_PORT=$SMTP_PORT
EOF
    echo "env_file_created" >> "$PROGRESS_FILE"
fi

# 9. Set up Systemd Service
if ! is_step_done "systemd_service_setup"; then
    log "Creating systemd service..."
    if [ ! -f /etc/systemd/system/blog-backend.service ]; then
        cat > /etc/systemd/system/blog-backend.service << EOF
[Unit]
Description=Blog Backend Service
After=network.target postgresql.service redis-server.service

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$APP_DIR
Environment="RUST_LOG=info"
ExecStart=$APP_DIR/target/release/blog-backend
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
        systemctl daemon-reload
    fi
    echo "systemd_service_setup" >> "$PROGRESS_FILE"
fi

# 10. Configure Nginx
if ! is_step_done "nginx_configured"; then
    log "Configuring Nginx..."
    cat > /etc/nginx/sites-available/blog-backend << EOF
server {
    listen 80;
    server_name $API_SUBDOMAIN;
    return 301 https://\$host\$request_uri;
}

server {
    listen 80;
    server_name $API_SUBDOMAIN;

    location / {
        proxy_pass http://127.0.0.1:8888;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_cache_bypass \$http_upgrade;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    }
}
EOF
    ln -sf /etc/nginx/sites-available/blog-backend /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default
    nginx -t
    systemctl restart nginx
    echo "nginx_configured" >> "$PROGRESS_FILE"
fi

# 11. Set up backup script
if ! is_step_done "backup_script_setup"; then
    log "Setting up backup script..."
    mkdir -p /home/$APP_USER/backups
    if [ ! -f /home/$APP_USER/backup-blog.sh ]; then
        cat > /home/$APP_USER/backup-blog.sh << EOF
#!/bin/bash
DATE=\$(date +%Y%m%d)
BACKUP_DIR=/home/$APP_USER/backups

# Database backup
pg_dump -U $DB_USER $DB_NAME > \$BACKUP_DIR/blog_\$DATE.sql

# Application backup
tar -czf \$BACKUP_DIR/blog_\$DATE.tar.gz $APP_DIR

# Keep only last 7 days of backups
find \$BACKUP_DIR -name "blog_*.sql" -mtime +7 -delete
find \$BACKUP_DIR -name "blog_*.tar.gz" -mtime +7 -delete
EOF
        chmod +x /home/$APP_USER/backup-blog.sh
        chown $APP_USER:$APP_USER /home/$APP_USER/backup-blog.sh
    fi
    # Add to crontab
    (crontab -l 2>/dev/null; echo "0 0 * * * /home/$APP_USER/backup-blog.sh") | crontab -
    echo "backup_script_setup" >> "$PROGRESS_FILE"
fi

# 12. Set up maintenance script
if ! is_step_done "maintenance_script_setup"; then
    log "Setting up maintenance script..."
    if [ ! -f /home/$APP_USER/maintain.sh ]; then
        cat > /home/$APP_USER/maintain.sh << EOF
#!/bin/bash
apt update
apt upgrade -y
apt autoremove -y
journalctl --vacuum-time=7d
EOF
        chmod +x /home/$APP_USER/maintain.sh
        chown $APP_USER:$APP_USER /home/$APP_USER/maintain.sh
    fi
    # Add to crontab (run maintenance weekly)
    (crontab -l 2>/dev/null; echo "0 0 * * 0 /home/$APP_USER/maintain.sh") | crontab -
    echo "maintenance_script_setup" >> "$PROGRESS_FILE"
fi

# 13. Build and start the application
if ! is_step_done "app_built_and_started"; then
    log "Building application..."
    cd $APP_DIR
    cargo build --release
    log "Running database migrations..."
    diesel migration run --database-url postgres://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME
    log "Starting services..."
    systemctl enable blog-backend
    systemctl start blog-backend
    echo "app_built_and_started" >> "$PROGRESS_FILE"
fi

# 14. Configure fail2ban
if ! is_step_done "fail2ban_configured"; then
    log "Configuring fail2ban..."
    systemctl enable fail2ban
    systemctl start fail2ban
    echo "fail2ban_configured" >> "$PROGRESS_FILE"
fi

# Final steps and cleanup
log "Setting correct permissions..."
chown -R $APP_USER:$APP_USER $APP_DIR
chmod -R 755 $APP_DIR

# Print summary
log "Installation complete! Summary of details:"
echo "-----------------------------------"
echo "Application URL: https://$API_SUBDOMAIN"
echo "Database Name: $DB_NAME"
echo "Database User: $DB_USER"
echo "Redis is running on localhost:6379"
echo "Backup script: /home/$APP_USER/backup-blog.sh"
echo "Maintenance script: /home/$APP_USER/maintain.sh"
echo "Logs can be viewed with: journalctl -u blog-backend -f"
echo "-----------------------------------"
