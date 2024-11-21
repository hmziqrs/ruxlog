#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Configuration variables - MAKE SURE THESE MATCH YOUR ORIGINAL DEPLOYMENT
DOMAIN="your-domain.com"
API_SUBDOMAIN="api.$DOMAIN"
DB_USER="bloguser"
DB_NAME="blogdb"
APP_USER=$USER
APP_DIR="/home/$APP_USER/apps/blog"

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

# 1. Stop and remove services
log "Stopping and removing services..."
systemctl stop blog-backend
systemctl disable blog-backend
rm -f /etc/systemd/system/blog-backend.service
systemctl daemon-reload

# 2. Remove Nginx configuration
log "Removing Nginx configuration..."
rm -f /etc/nginx/sites-enabled/blog-backend
rm -f /etc/nginx/sites-available/blog-backend
systemctl restart nginx

# 3. Remove SSL certificates
log "Removing SSL certificates..."
certbot delete --cert-name $API_SUBDOMAIN

# 4. Remove PostgreSQL database and user
log "Removing PostgreSQL database and user..."
sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"
sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;"

# 5. Reset Redis configuration
log "Resetting Redis configuration..."
if [ -f /etc/redis/redis.conf.backup ]; then
    mv /etc/redis/redis.conf.backup /etc/redis/redis.conf
else
    error "Redis backup configuration not found"
fi
rm -f /etc/redis/users.acl
systemctl restart redis-server

# 6. Remove application directory
log "Removing application directory..."
rm -rf $APP_DIR

# 7. Remove backup and maintenance scripts
log "Removing backup and maintenance scripts..."
rm -f /home/$APP_USER/backup-blog.sh
rm -f /home/$APP_USER/maintain.sh
rm -rf /home/$APP_USER/backups

# 8. Remove cron jobs
log "Removing cron jobs..."
crontab -l | grep -v "backup-blog.sh" | grep -v "maintain.sh" | crontab -

# 9. Optional: Remove installed packages
# Note: This might remove packages that were installed for other purposes
read -p "Do you want to remove installed packages? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    log "Removing installed packages..."
    apt remove -y build-essential curl git pkg-config libssl-dev postgresql postgresql-contrib redis-server nginx fail2ban htop python3-certbot-nginx
    apt autoremove -y
fi

log "Uninstallation complete!"
log "Note: Some system changes may remain. Manual verification is recommended."
log "The following were not removed:"
echo "- UFW rules (if you want to remove them, use 'ufw delete allow')"
echo "- Rust installation (if you want to remove it, remove ~/.cargo and ~/.rustup)"
echo "- Diesel CLI installation"
