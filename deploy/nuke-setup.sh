#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration variables - CHANGE THESE to match your setup
DOMAIN="your-domain.com"
API_SUBDOMAIN="api.$DOMAIN"
DB_USER="bloguser"
DB_NAME="blogdb"
APP_USER=$USER
APP_DIR="/home/$APP_USER/apps/blog"
REPO_URL="your-git-repo-url"

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

# Prompt for confirmation
read -p "This script will undo changes made by the installation script. Are you sure? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

# 1. Stop and disable services
log "Stopping and disabling services..."
systemctl stop blog-backend
systemctl disable blog-backend
systemctl stop fail2ban
systemctl disable fail2ban

# 2. Remove systemd service configuration
log "Removing systemd service configuration..."
rm -f /etc/systemd/system/blog-backend.service
systemctl daemon-reload

# 3. Remove PostgreSQL user and database
log "Removing PostgreSQL user and database..."
sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"
sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;"

# 4. Revert Redis configuration
log "Reverting Redis configuration..."
if [ -f /etc/redis/redis.conf.backup ]; then
    mv /etc/redis/redis.conf.backup /etc/redis/redis.conf
    rm -f /etc/redis/users.acl
    systemctl restart redis-server
fi

# 5. Remove application directory
log "Removing application directory..."
rm -rf $APP_DIR

# 6. Remove backup scripts and backups
log "Removing backup scripts and backups..."
rm -f /home/$APP_USER/backup-blog.sh
rm -rf /home/$APP_USER/backups

# 7. Remove maintenance script
log "Removing maintenance script..."
rm -f /home/$APP_USER/maintain.sh

# 8. Remove cron jobs
log "Removing cron jobs..."
CRONJOBS=$(crontab -l | grep -v "/home/$APP_USER/backup-blog.sh" | grep -v "/home/$APP_USER/maintain.sh")
echo "$CRONJOBS" | crontab -

# 9. Remove Nginx configuration
log "Removing Nginx configuration..."
rm -f /etc/nginx/sites-available/blog-backend
rm -f /etc/nginx/sites-enabled/blog-backend
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl restart nginx

# 10. Remove SSL certificates
# Commented out to preserve certificates
# log "Removing SSL certificates..."
# certbot delete --cert-name $API_SUBDOMAIN --non-interactive

# 11. Uninstall packages
log "Preparing to uninstall packages..."
PACKAGES="build-essential curl git pkg-config libssl-dev postgresql postgresql-contrib redis-server nginx fail2ban htop python3-certbot-nginx ufw"
echo "The following packages will be removed:"
echo $PACKAGES
read -p "Do you want to continue? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    apt remove -y $PACKAGES
    apt autoremove -y
else
    log "Package uninstallation aborted."
fi

# 12. Reset firewall rules
log "Resetting firewall rules..."
ufw disable
ufw --force reset

# 13. Remove Rust and Diesel CLI (if installed by the script)
log "Removing Rust and Diesel CLI..."
if [ -d "$HOME/.cargo" ]; then
    rm -rf $HOME/.cargo
    rm -rf $HOME/.rustup
fi

# 14. Clean up log files and temporary files
log "Cleaning up log files and temporary files..."
rm -f /var/log/nginx/access.log /var/log/nginx/error.log
rm -f /var/log/redis/redis-server.log
journalctl --vacuum-time=1d

# Final steps and summary
log "Undo process complete! Summary of actions:"
echo "-----------------------------------"
echo "Removed services: blog-backend, fail2ban"
echo "Deleted PostgreSQL user: $DB_USER and database: $DB_NAME"
echo "Reverted Redis configuration"
echo "Removed application directory: $APP_DIR"
echo "Deleted backup scripts and backups"
echo "Removed maintenance script"
echo "Cleared cron jobs for backups and maintenance"
echo "Removed Nginx configuration for $API_SUBDOMAIN"
echo "Preserved SSL certificates for $API_SUBDOMAIN"
echo "Uninstalled specified packages (if confirmed)"
echo "Reset firewall rules"
echo "Removed Rust and Diesel CLI (if installed by the script)"
echo "Cleaned up log files and temporary files"
echo "-----------------------------------"

warning "Please verify that all changes have been undone as expected."
