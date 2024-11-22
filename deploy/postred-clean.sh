set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Stop services
systemctl stop postgresql
systemctl stop redis-server

# Remove PostgreSQL
apt remove --purge postgresql postgresql-contrib -y
apt autoremove -y
rm -rf /var/lib/postgresql/
rm -rf /var/log/postgresql/
rm -rf /etc/postgresql/

# Remove Redis
apt remove --purge redis-server -y
apt autoremove -y
rm -rf /var/lib/redis/
rm -rf /var/log/redis/
rm -rf /etc/redis/

echo "PostgreSQL and Redis have been removed from the system."
