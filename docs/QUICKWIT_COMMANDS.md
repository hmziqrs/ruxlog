# Quickwit Setup Commands

## Quick Start

### 1. Start Quickwit (without bootstrap container)
```bash
docker compose -f docker-compose.observability.yml up -d quickwit minio
```

### 2. Validate Setup
```bash
./scripts/validate_quickwit.sh
```

### 3. Bootstrap Indexes Manually
```bash
./scripts/bootstrap_quickwit_manual.sh
```

---

## Detailed Commands

### Validation Checks

**Run full validation suite:**
```bash
./scripts/validate_quickwit.sh
```

This checks:
- ✓ Container is running
- ✓ Quickwit binary exists
- ✓ Config files are present
- ✓ Index configs are available
- ✓ API is healthy
- ✓ Data directory exists
- ✓ Indexes are created
- ✓ MinIO connectivity
- ✓ Bootstrap script exists

**Manual checks:**

```bash
# Check if container is running
docker ps | grep ruxlog-quickwit

# Check Quickwit version inside container
docker exec ruxlog-quickwit quickwit --version

# Check if config file exists
docker exec ruxlog-quickwit test -f /quickwit/config/quickwit.yaml && echo "Config exists" || echo "Config missing"

# List index config files
docker exec ruxlog-quickwit ls -la /quickwit/config/indexes/

# Check API health
docker exec ruxlog-quickwit curl -f http://localhost:7280/api/v1/health

# Check MinIO connectivity
docker exec ruxlog-quickwit curl -f http://minio:9000/minio/health/live

# View container logs
docker logs ruxlog-quickwit --tail 50
```

---

## Bootstrap Commands

### Automatic Bootstrap (Recommended)
```bash
./scripts/bootstrap_quickwit_manual.sh
```

### Manual Bootstrap via docker exec
```bash
# Execute bootstrap script inside container
docker exec ruxlog-quickwit sh -c '
  for index_file in /quickwit/config/indexes/*.yaml; do
    if [ -f "$index_file" ]; then
      echo "Creating index from $index_file"
      quickwit --config /quickwit/config/quickwit.yaml index create \
        --index-config "$index_file" 2>&1 | grep -v "already exists" || true
    fi
  done
'
```

### Create Individual Index
```bash
# Create logs index
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index create --index-config /quickwit/config/indexes/logs.yaml

# Create traces index  
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index create --index-config /quickwit/config/indexes/traces.yaml

# Create metrics index
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index create --index-config /quickwit/config/indexes/metrics.yaml
```

---

## Index Management Commands

### List All Indexes
```bash
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml index list
```

### Describe Specific Index
```bash
# Logs index
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index describe --index ruxlog-logs

# Traces index
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index describe --index ruxlog-traces

# Metrics index
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index describe --index ruxlog-metrics
```

### Delete Index (if needed)
```bash
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index delete --index ruxlog-logs --yes
```

---

## Query & Search Commands

### Search via CLI
```bash
# Search logs
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index search --index ruxlog-logs --query "*"

# Search with filter
docker exec ruxlog-quickwit quickwit --config /quickwit/config/quickwit.yaml \
  index search --index ruxlog-logs --query "level:error"
```

### Search via API
```bash
# Search logs API
curl 'http://localhost:7280/api/v1/ruxlog-logs/search?query=*'

# Search with filter
curl 'http://localhost:7280/api/v1/ruxlog-logs/search?query=level:error'

# Search traces
curl 'http://localhost:7280/api/v1/ruxlog-traces/search?query=*'
```

---

## Container Management

### Start Services
```bash
# Start everything
docker compose -f docker-compose.observability.yml up -d

# Start only Quickwit and MinIO
docker compose -f docker-compose.observability.yml up -d quickwit minio
```

### Stop Services
```bash
# Stop all
docker compose -f docker-compose.observability.yml down

# Stop but keep volumes
docker compose -f docker-compose.observability.yml stop

# Stop and remove volumes (CAUTION: deletes all data)
docker compose -f docker-compose.observability.yml down -v
```

### Restart Quickwit
```bash
docker restart ruxlog-quickwit
```

### View Logs
```bash
# Follow logs
docker logs -f ruxlog-quickwit

# Last 50 lines
docker logs ruxlog-quickwit --tail 50

# Logs since 10 minutes ago
docker logs ruxlog-quickwit --since 10m
```

### Interactive Shell
```bash
# Enter container shell
docker exec -it ruxlog-quickwit sh

# Run commands interactively
docker exec -it ruxlog-quickwit quickwit --help
```

---

## Troubleshooting

### Check if all binaries are available
```bash
# Quickwit binary
docker exec ruxlog-quickwit which quickwit

# Curl (for health checks)
docker exec ruxlog-quickwit which curl

# Test basic commands
docker exec ruxlog-quickwit sh -c 'ls -la /quickwit/config && echo "Config dir OK"'
```

### Verify index files are mounted correctly
```bash
docker exec ruxlog-quickwit sh -c 'cat /quickwit/config/indexes/logs.yaml | head -20'
```

### Check network connectivity
```bash
# From Quickwit to MinIO
docker exec ruxlog-quickwit ping -c 3 minio

# Check if MinIO port is accessible
docker exec ruxlog-quickwit nc -zv minio 9000
```

### Reset everything (clean slate)
```bash
# Stop all services
docker compose -f docker-compose.observability.yml down -v

# Remove any orphaned containers
docker compose -f docker-compose.observability.yml rm -f

# Start fresh
docker compose -f docker-compose.observability.yml up -d quickwit minio

# Wait for startup
sleep 10

# Validate
./scripts/validate_quickwit.sh

# Bootstrap
./scripts/bootstrap_quickwit_manual.sh
```

---

## Web Interfaces & Endpoints

- **Quickwit UI**: http://localhost:7280/ui
- **API Playground**: http://localhost:7280/ui/api-playground
- **Quickwit API**: http://localhost:7280/api/v1
- **Liveness Check**: http://localhost:7280/health/livez
- **Readiness Check**: http://localhost:7280/health/readyz
- **Version Info**: http://localhost:7280/api/v1/version
- **MinIO Console**: http://localhost:9001 (credentials: quickwit/quickwit-secret)

---

## Common Issues

### "Config file not found"
```bash
# Check if volume is mounted
docker inspect ruxlog-quickwit | grep -A 10 Mounts

# Verify files exist on host
ls -la observability/quickwit/config/
```

### "Index already exists"
This is normal on re-runs. The script handles this gracefully.

### "MinIO connection failed"
```bash
# Ensure MinIO is running
docker ps | grep minio

# Check if bucket exists (optional, Quickwit creates it automatically)
docker exec ruxlog-quickwit curl -I http://minio:9000/quickwit
```

### "API not responding"
```bash
# Check if service is running
docker exec ruxlog-quickwit ps aux | grep quickwit

# Check logs for errors
docker logs ruxlog-quickwit --tail 100
```

---

## Configuration Files Location

- Main config: `observability/quickwit/config/quickwit.yaml`
- Index configs: `observability/quickwit/config/indexes/*.yaml`
- Scripts: `scripts/validate_quickwit.sh`, `scripts/bootstrap_quickwit_manual.sh`
