#!/bin/bash

# Don't exit on errors (removing the set -e)
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
systemctl stop blog-backend || warning "Failed to stop blog-backend service"
systemctl disable blog-backend || warning "Failed to disable blog-backend service"
systemctl stop fail2ban || warning "Failed to stop fail2ban"
systemctl disable fail2ban || warning "Failed to disable fail2ban"

# 2. Remove systemd service configuration
log "Removing systemd service configuration..."
rm -f /etc/systemd/system/blog-backend.service || warning "Failed to remove service file"
systemctl daemon-reload || warning "Failed to reload systemd daemon"

# 3. Remove PostgreSQL completely
log "Removing PostgreSQL completely..."
# First stop the services
systemctl stop postgresql || warning "Failed to stop PostgreSQL service"
systemctl disable postgresql || warning "Failed to disable PostgreSQL service"

# Drop database and user before removing PostgreSQL
sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;" || warning "Failed to drop database"
sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;" || warning "Failed to drop user"

# Completely remove PostgreSQL packages and configurations
apt purge -y postgresql* || warning "Failed to purge PostgreSQL packages"
apt autoremove -y || warning "Failed to autoremove packages"
apt autoclean || warning "Failed to autoclean packages"

# Remove PostgreSQL directories and configurations
rm -rf /etc/postgresql/ || warning "Failed to remove PostgreSQL config directory"
rm -rf /var/lib/postgresql/ || warning "Failed to remove PostgreSQL data directory"
rm -rf /var/log/postgresql/ || warning "Failed to remove PostgreSQL log directory"

# Update the package list
apt update || warning "Failed to update package list"

# 4. Revert Redis configuration
log "Reverting Redis configuration..."
if [ -f /etc/redis/redis.conf.backup ]; then
    mv /etc/redis/redis.conf.backup /etc/redis/redis.conf || warning "Failed to restore Redis config"
    rm -f /etc/redis/users.acl || warning "Failed to remove Redis ACL file"
    systemctl restart redis-server || warning "Failed to restart Redis"
fi

# 5. Remove application directory
log "Removing application directory..."
rm -rf $APP_DIR || warning "Failed to remove application directory"

# 6. Remove backup scripts and backups
log "Removing backup scripts and backups..."
rm -f /home/$APP_USER/backup-blog.sh || warning "Failed to remove backup script"
rm -rf /home/$APP_USER/backups || warning "Failed to remove backups directory"

# 7. Remove maintenance script
log "Removing maintenance script..."
rm -f /home/$APP_USER/maintain.sh || warning "Failed to remove maintenance script"

# 8. Remove cron jobs
log "Removing cron jobs..."
(crontab -l | grep -v "/home/$APP_USER/backup-blog.sh" | grep -v "/home/$APP_USER/maintain.sh" | crontab -) || warning "Failed to update crontab"

# 9. Remove Nginx configuration
log "Removing Nginx configuration..."
rm -f /etc/nginx/sites-available/blog-backend || warning "Failed to remove Nginx config"
rm -f /etc/nginx/sites-enabled/blog-backend || warning "Failed to remove Nginx symlink"
rm -f /etc/nginx/sites-enabled/default || warning "Failed to remove default Nginx config"
nginx -t || warning "Nginx configuration test failed"
systemctl restart nginx || warning "Failed to restart Nginx"

# 11. Uninstall packages
# 11. Uninstall packages
log "Preparing to uninstall packages..."
PACKAGES="postgresql-contrib redis-server nginx fail2ban"
echo "The following packages will be removed:"
echo $PACKAGES
read -p "Do you want to continue? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Use purge instead of remove to completely remove packages and their configurations
    apt purge -y $PACKAGES || warning "Failed to purge packages"
    apt autoremove -y || warning "Failed to autoremove packages"
    apt autoclean || warning "Failed to autoclean packages"

    # Remove remaining configuration directories
    rm -rf /etc/nginx/ || warning "Failed to remove Nginx config directory"
    rm -rf /etc/redis/ || warning "Failed to remove Redis config directory"
    rm -rf /etc/fail2ban/ || warning "Failed to remove Fail2ban config directory"
else
    log "Package uninstallation aborted."
fi

# 12. Reset firewall rules
log "Resetting firewall rules..."
ufw disable || warning "Failed to disable UFW"
ufw --force reset || warning "Failed to reset UFW rules"

# 13. Remove Rust and Diesel CLI
log "Removing Rust and Diesel CLI..."
if [ -d "$HOME/.cargo" ]; then
    rm -rf $HOME/.cargo || warning "Failed to remove Cargo"
    rm -rf $HOME/.rustup || warning "Failed to remove Rustup"
fi

# 14. Clean up log files and temporary files
log "Cleaning up log files and temporary files..."
rm -f /var/log/nginx/access.log /var/log/nginx/error.log || warning "Failed to remove Nginx logs"
rm -f /var/log/redis/redis-server.log || warning "Failed to remove Redis logs"
rm -f /var/log/setup_progress.txt || warning "Failed to remove setup progress file" # Added this line
rm -rf /var/log/setup_progress.txt/var/log/setup_progress.txt || warning "Failed to remove setup progress file" # Added this line
journalctl --vacuum-time=1d || warning "Failed to clean journalctl logs"

# Final steps and summary
log "Undo process complete! Summary of actions:"
echo "-----------------------------------"
echo "Attempted to remove services: blog-backend, fail2ban"
echo "Attempted to delete PostgreSQL user: $DB_USER and database: $DB_NAME"
echo "Attempted to revert Redis configuration"
echo "Attempted to remove application directory: $APP_DIR"
echo "Attempted to delete backup scripts and backups"
echo "Attempted to remove maintenance script"
echo "Attempted to clear cron jobs for backups and maintenance"
echo "Attempted to remove Nginx configuration for $API_SUBDOMAIN"
echo "Preserved SSL certificates for $API_SUBDOMAIN"
echo "Attempted to uninstall specified packages (if confirmed)"
echo "Attempted to reset firewall rules"
echo "Attempted to remove Rust and Diesel CLI"
echo "Attempted to clean up log files and temporary files"
echo "-----------------------------------"

warning "Please verify that all changes have been undone as expected."
warning "Some operations might have failed - check the warnings above."
