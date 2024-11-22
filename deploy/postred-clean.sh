#!/bin/bash
set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Kill any running PostgreSQL and Redis processes
pkill -9 postgres || true
pkill -9 redis || true

# Stop services
systemctl stop postgresql || true
systemctl stop redis-server || true

# Disable services
systemctl disable postgresql || true
systemctl disable redis-server || true

# Remove PostgreSQL
apt remove --purge postgresql* -y
apt autoremove -y
rm -rf /var/lib/postgresql/
rm -rf /var/log/postgresql/
rm -rf /etc/postgresql/
rm -rf /var/run/postgresql/
rm -rf /usr/lib/postgresql/
rm -rf /usr/share/postgresql/
rm -rf /usr/share/postgresql-common/
rm -f /var/run/postgresql/.s.PGSQL*

# Remove Redis
apt remove --purge redis* -y
apt autoremove -y
rm -rf /var/lib/redis/
rm -rf /var/log/redis/
rm -rf /etc/redis/
rm -rf /var/run/redis/
rm -rf /usr/lib/redis/
rm -rf /usr/share/redis/

# Clean package lists and cached packages
apt clean
apt autoclean

# Remove any remaining configuration files
dpkg --purge $(dpkg -l | grep postgres | awk '{print $2}')
dpkg --purge $(dpkg -l | grep redis | awk '{print $2}')

echo "PostgreSQL and Redis have been completely removed from the system."
