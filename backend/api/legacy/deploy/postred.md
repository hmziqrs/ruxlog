This updated script:

1. Configures Redis with ACL-based authentication (Redis 6.0+ feature)
2. Creates a specific user with a password
3. Sets up proper permissions for the ACL file
4. Tests the Redis connection with the new credentials
5. Provides a Redis connection URL with both username and password

To use this script:

1. Save it to a file (e.g., `setup-db.sh`)
2. Make it executable: `chmod +x setup-db.sh`
3. Edit the configuration variables at the top of the script
4. Run it with sudo: `sudo ./setup-db.sh`

Important notes:

- This script requires Redis 6.0 or higher for username support
- The Redis connection URL format is: `redis://username:password@host:port`
- The script creates a backup of the original Redis configuration file
- The script includes basic security settings for Redis
- Make sure to change the default username and password in the configuration variables before running the script

## Result:

Database Name: blog
Database User: root
Database Password: root
Database Connection URL: postgres://root:root@localhost:5432/blog

Redis Host: 127.0.0.1
Redis Port: 6379
Redis Username: red
Redis Password: red
Redis Connection URL: redis://red:red@127.0.0.1:6379

---

Commands:
psql -U root -d blog

redis-cli -u "redis://red:red@127.0.0.1:6379"

---

rm -rf ./postred-clean.sh && nano postred-clean.sh && chmod +x postred-clean.sh && sudo ./postred-clean.sh
rm -rf ./postred.sh && nano postred.sh && chmod +x postred.sh && sudo ./postred.sh
