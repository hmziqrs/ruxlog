#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration variables - CHANGE THESE
DOMAIN="your-domain.com"
API_SUBDOMAIN="api.$DOMAIN"
DB_USER="bloguser"
DB_PASSWORD="your_secure_db_password"
DB_NAME="blogdb"
REDIS_USERNAME="redisuser"
REDIS_PASSWORD="your_secure_redis_password"
SMTP_HOST="your_smtp_host"
SMTP_USERNAME="your_smtp_username"
SMTP_PASSWORD="your_smtp_password"
SMTP_PORT="587"
APP_USER=$USER
APP_DIR="/home/$APP_USER/apps/blog"
REPO_URL="your-git-repo-url"

# Generate random keys
COOKIE_KEY=$(openssl rand -hex 32)
CSRF_KEY=$(openssl rand -hex 32)

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

# 2. Install Essential Packages
log "Installing essential packages..."
apt install -y build-essential curl git pkg-config libssl-dev postgresql postgresql-contrib redis-server nginx fail2ban htop python3-certbot-nginx ufw

# 3. Configure Firewall
log "Configuring firewall..."
ufw allow ssh
ufw allow 'Nginx Full'
ufw --force enable

# 4. Configure PostgreSQL
log "Configuring PostgreSQL..."
sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';"
sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;"

# 5. Configure Redis
log "Configuring Redis..."
# Backup original Redis configuration
cp /etc/redis/redis.conf /etc/redis/redis.conf.backup

# Create new Redis configuration with ACL support
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
systemctl restart redis-server

# Test Redis connection
sleep 2
if ! redis-cli -u "redis://$REDIS_USERNAME:$REDIS_PASSWORD@127.0.0.1:6379" ping | grep -q "PONG"; then
    error "Redis authentication test failed"
    exit 1
fi

# 6. Install Rust and tools
log "Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

log "Installing Diesel CLI..."
cargo install diesel_cli --no-default-features --features postgres

# 7. Set up Application Directory
log "Setting up application directory..."
mkdir -p $APP_DIR
cd $APP_DIR
git clone $REPO_URL .

# 8. Create .env file
log "Creating .env file..."
cat > $APP_DIR/.env << EOF
HOST=127.0.0.1
PORT=8888

# Database
POSTGRE_DB_URL=postgres://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME

# Redis
REDIS_USERNAME=$REDIS_USERNAME
REDIS_PASSWORD=$REDIS_PASSWORD
REDIS_HOST=127.0.0.1
REDIS_PORT=6379

# Keys
COOKIE_KEY=$COOKIE_KEY
CSRF_KEY=$CSRF_KEY

# SMTP
SMTP_HOST=$SMTP_HOST
SMTP_USERNAME=$SMTP_USERNAME
SMTP_PASSWORD=$SMTP_PASSWORD
SMTP_PORT=$SMTP_PORT
EOF

# 9. Set up Systemd Service
log "Creating systemd service..."
cat > /etc/systemd/system/blog-backend.service << EOF
[Unit]
Description=Blog Backend Service
After=network.target postgresql.service redis-server.service

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$APP_DIR
Environment="RUST_LOG=info"
ExecStart=$APP_DIR/target/release/ruxlog
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 10. Configure Nginx
log "Configuring Nginx..."
cat > /etc/nginx/sites-available/blog-backend << EOF
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

# 11. SSL Certificate
log "Setting up SSL certificate..."
certbot --nginx -d $API_SUBDOMAIN --non-interactive --agree-tos --email admin@$DOMAIN

# 12. Set up backup script
log "Setting up backup script..."
mkdir -p /home/$APP_USER/backups

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

# Add to crontab
(crontab -l 2>/dev/null; echo "0 0 * * * /home/$APP_USER/backup-blog.sh") | crontab -

# 13. Set up maintenance script
log "Setting up maintenance script..."
cat > /home/$APP_USER/maintain.sh << EOF
#!/bin/bash
apt update
apt upgrade -y
apt autoremove -y
journalctl --vacuum-time=7d
EOF

chmod +x /home/$APP_USER/maintain.sh
chown $APP_USER:$APP_USER /home/$APP_USER/maintain.sh

# Add to crontab (run maintenance weekly)
(crontab -l 2>/dev/null; echo "0 0 * * 0 /home/$APP_USER/maintain.sh") | crontab -

# 14. Build and start the application
log "Building application..."
cd $APP_DIR
cargo build --release

log "Running database migrations..."
diesel migration run

log "Starting services..."
systemctl daemon-reload
systemctl enable blog-backend
systemctl start blog-backend

# 15. Configure fail2ban
log "Configuring fail2ban..."
systemctl enable fail2ban
systemctl start fail2ban

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

warning "Please save these credentials securely and then delete them from this script!"
warning "Remember to update the DNS records for $API_SUBDOMAIN"
