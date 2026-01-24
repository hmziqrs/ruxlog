# Migration Plan: Replace Garage with RustFS

## Overview
Replace Garage S3-compatible storage with RustFS across all environments. RustFS is a high-performance S3-compatible object storage system built in Rust. The migration requires zero backend code changes due to S3 compatibility.

## Key Insights
- Backend uses `aws-sdk-s3` crate - fully S3-compatible, no code changes needed
- RustFS uses simpler configuration (env vars) vs Garage (TOML + CLI)
- RustFS runs as non-root user (UID 10001) requiring proper volume permissions
- Different port defaults: RustFS (9000/9001) vs Garage (3900/3901/3902/3903)
- Production uses Cloudflare R2, not Garage - migration only affects dev/test/stage

## Critical Files to Modify

### 1. Docker Infrastructure
**File:** `/Users/hmziq/os/ruxlog/docker-compose.yml`
- **Lines 90-116**: Replace entire `garage` service with `rustfs` service
- **Lines 126-129**: Replace `garage_meta` and `garage_data` volumes with `rustfs_data`

### 2. Environment Files (Remove GARAGE_* vars, add RUSTFS_* vars)
- `/Users/hmziq/os/ruxlog/.env.dev` (lines 42, 48, 50-59)
- `/Users/hmziq/os/ruxlog/.env.test` (similar structure)
- `/Users/hmziq/os/ruxlog/.env.stage` (similar structure)
- `/Users/hmziq/os/ruxlog/.env.remote` (similar structure)
- `/Users/hmziq/os/ruxlog/backend/.env.docker` (similar structure)
- `/Users/hmziq/os/ruxlog/.env.prod` (keep as-is, uses R2)

### 3. Bootstrap Script
**Create:** `/Users/hmziq/os/ruxlog/scripts/rustfs-bootstrap.sh`
**Delete:** `/Users/hmziq/os/ruxlog/scripts/garage-bootstrap.sh`

### 4. Justfile
**File:** `/Users/hmziq/os/ruxlog/Justfile`
- **Line 29**: Update `storage-init` to call `rustfs-bootstrap.sh`

### 5. Cleanup
**Delete:** `/Users/hmziq/os/ruxlog/backend/docker/garage/` (entire directory)

## Implementation Steps

### Phase 1: Docker Compose Update

Replace garage service (lines 90-116) with:

```yaml
rustfs:
  <<: *service-defaults
  profiles: ["storage"]
  image: rustfs/rustfs:1.0.0-alpha.81  # Pin version; latest as of 2026-01-22
  command: /data
  environment:
    RUSTFS_ADDRESS: ":9000"
    RUSTFS_EXTERNAL_ADDRESS: ":9000"
    RUSTFS_ACCESS_KEY: ${RUSTFS_ACCESS_KEY}
    RUSTFS_SECRET_KEY: ${RUSTFS_SECRET_KEY}
    RUSTFS_CONSOLE_ENABLE: "true"
  ports:
    - "${RUSTFS_API_PORT:-1105}:9000"
    - "${RUSTFS_CONSOLE_PORT:-1106}:9001"
  volumes:
    - rustfs_data:/data
  user: "10001:10001"
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:9000/health"]
    interval: 10s
    timeout: 5s
    retries: 5
  security_opt:
    - no-new-privileges:true
```

Update volumes section (lines 126-129):

```yaml
# REMOVE:
# garage_meta:
#   driver: local
# garage_data:
#   driver: local

# ADD:
rustfs_data:
  driver: local
```

### Phase 2: Environment Variable Updates

For each env file, make these changes:

**Remove (lines 50-59 in .env.dev):**
```bash
GARAGE_ZONE=dc1
GARAGE_CAPACITY=5G
GARAGE_RPC_SECRET=...
GARAGE_ADMIN_TOKEN=...
GARAGE_METRICS_TOKEN=...
GARAGE_ACCESS_KEY_NAME=ruxlog-local
GARAGE_RPC_PORT=1103
GARAGE_ADMIN_PORT=1104
GARAGE_S3_PORT=1105
GARAGE_WEB_PORT=1106
```

**Add:**
```bash
# RustFS Configuration
RUSTFS_ACCESS_KEY=${S3_ACCESS_KEY}
RUSTFS_SECRET_KEY=${S3_SECRET_KEY}
RUSTFS_API_PORT=1105  # Keep same port as old GARAGE_S3_PORT
RUSTFS_CONSOLE_PORT=1106  # Keep same port as old GARAGE_WEB_PORT
```

**Update (line 42):**
```bash
S3_REGION=us-east-1  # Was: garage
```

**Environment-specific S3_ENDPOINT updates:**
- `.env.dev`, `.env.test`, `.env.remote`: `http://127.0.0.1:${RUSTFS_API_PORT}`
- `backend/.env.docker`, `.env.stage`: `http://rustfs:9000` (internal Docker)
- `.env.prod`: No change (uses Cloudflare R2)

### Phase 3: Bootstrap Script

Create `/Users/hmziq/os/ruxlog/scripts/rustfs-bootstrap.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REQUESTED_ENV="${1:-.env.dev}"

if [[ "${REQUESTED_ENV}" = /* ]]; then
  ENV_PATH="${REQUESTED_ENV}"
else
  ENV_PATH="${PROJECT_ROOT}/${REQUESTED_ENV}"
fi

if [[ ! -f "${ENV_PATH}" ]]; then
  echo "[rustfs-bootstrap] Unable to find env file: ${ENV_PATH}" >&2
  exit 1
fi

set -a
source "${ENV_PATH}"
set +a

cd "${PROJECT_ROOT}" >/dev/null 2>&1
COMPOSE_CMD=(docker compose --env-file "${ENV_PATH}")

bucket_name="${S3_BUCKET:-${AWS_S3_BUCKET:-}}"
access_key="${RUSTFS_ACCESS_KEY:-${S3_ACCESS_KEY:-}}"
secret_key="${RUSTFS_SECRET_KEY:-${S3_SECRET_KEY:-}}"

if [[ -z "${bucket_name}" ]]; then
  echo "[rustfs-bootstrap] S3_BUCKET is not set" >&2
  exit 1
fi

if [[ -z "${access_key}" || -z "${secret_key}" ]]; then
  echo "[rustfs-bootstrap] RUSTFS credentials not set" >&2
  exit 1
fi

echo "[rustfs-bootstrap] Ensuring RustFS is running (env: ${ENV_PATH})" >&2
"${COMPOSE_CMD[@]}" --profile storage up -d rustfs >/dev/null

# Wait for RustFS health check
ready=0
for i in {1..30}; do
  if curl -sf "http://localhost:${RUSTFS_API_PORT:-1105}/health" >/dev/null 2>&1; then
    ready=1
    echo "[rustfs-bootstrap] RustFS is healthy after ${i} attempts" >&2
    break
  fi
  sleep 2
done

if [[ "${ready}" -ne 1 ]]; then
  echo "[rustfs-bootstrap] RustFS service failed to become healthy" >&2
  exit 1
fi

# Create bucket using S3 REST API (no external dependencies)
echo "[rustfs-bootstrap] Setting up bucket ${bucket_name}" >&2

api_port="${RUSTFS_API_PORT:-1105}"
date_header=$(date -u +"%a, %d %b %Y %H:%M:%S GMT")

# Create bucket via PUT request
http_code=$(curl -sf -o /dev/null -w "%{http_code}" \
  -X PUT "http://localhost:${api_port}/${bucket_name}" \
  -H "Host: localhost:${api_port}" \
  -H "Date: ${date_header}" \
  -u "${access_key}:${secret_key}" \
  2>/dev/null) || true

if [[ "${http_code}" == "200" || "${http_code}" == "409" ]]; then
  echo "[rustfs-bootstrap] Bucket '${bucket_name}' ready (${http_code})" >&2
else
  echo "[rustfs-bootstrap] Warning: bucket creation returned ${http_code}, trying via docker exec..." >&2
  # Fallback: use docker exec into the running container
  "${COMPOSE_CMD[@]}" exec -T rustfs sh -c \
    "curl -sf -X PUT http://localhost:9000/${bucket_name} -u ${access_key}:${secret_key}" \
    2>/dev/null || echo "[rustfs-bootstrap] Bucket may already exist" >&2
fi

echo "[rustfs-bootstrap] RustFS ready!" >&2
echo "[rustfs-bootstrap] Console: http://localhost:${RUSTFS_CONSOLE_PORT:-1106}" >&2
echo "[rustfs-bootstrap] Credentials: ${access_key}" >&2
```

Make executable:
```bash
chmod +x /Users/hmziq/os/ruxlog/scripts/rustfs-bootstrap.sh
```

### Phase 4: Justfile Update

**File:** `/Users/hmziq/os/ruxlog/Justfile`

Update line 29:
```makefile
storage-init env='dev':
    scripts/rustfs-bootstrap.sh .env.{{env}}
```

Optional: Add storage management helpers after line 29:
```makefile
storage-console env='dev':
    @echo "RustFS Console: http://localhost:$(grep RUSTFS_CONSOLE_PORT .env.{{env}} | cut -d= -f2)"

storage-reset env='dev':
    docker compose --env-file .env.{{env}} stop rustfs
    docker volume rm -f $(grep PROJECT .env.{{env}} | cut -d= -f2)_rustfs_data || true
    just storage-init {{env}}
```

### Phase 5: Cleanup

Delete after successful migration:
1. `/Users/hmziq/os/ruxlog/backend/docker/garage/` (entire directory including `garage.toml`)
2. `/Users/hmziq/os/ruxlog/scripts/garage-bootstrap.sh`

Keep backups until verification complete.

## Execution Order

1. **Update docker-compose.yml** (Phase 1)

2. **Update all env files** (Phase 2) - dev, test, stage, remote, docker

3. **Create rustfs-bootstrap.sh** (Phase 3)

4. **Update Justfile** (Phase 4)

5. **Test dev environment**:
   ```bash
   just down dev
   just dev dev
   ```

6. **Verify functionality** (see Verification section)

7. **Delete old Garage files** (Phase 5)

## Verification Steps

### Service Health
- [ ] RustFS container starts: `docker ps | grep rustfs`
- [ ] Health check passes: `curl http://localhost:1105/health`
- [ ] Console accessible: Open `http://localhost:1106` (login with configured credentials)

### Bucket Operations
- [ ] Bucket created: Check via console or `curl http://localhost:1105/${S3_BUCKET}`
- [ ] Files accessible via S3 API

### Backend Integration
Run API and test media upload:
```bash
just api-dev dev
# In another terminal, test upload via API
curl -X POST http://localhost:1100/media/v1/create \
  -H "Authorization: Bearer <token>" \
  -F "file=@test-image.jpg"
```

- [ ] Upload succeeds
- [ ] File visible in RustFS console
- [ ] Public URL accessible: `curl http://localhost:1105/ruxlog-local/media/...`
- [ ] Delete operation works
- [ ] No S3 errors in API logs

### Environment Parity
Repeat verification for:
- [ ] Dev environment (`.env.dev`)
- [ ] Test environment (`.env.test`)
- [ ] Stage environment (`.env.stage`)
- [ ] Remote environment (`.env.remote`)
- [ ] Docker environment (`backend/.env.docker`)

### Performance Check
- [ ] Upload latency < 500ms for small files
- [ ] No connection timeouts
- [ ] Image optimization still works

## Rollback Plan

If migration fails, restore from git:

```bash
git checkout docker-compose.yml .env.* Justfile
git checkout scripts/garage-bootstrap.sh
just down dev
docker compose --env-file .env.dev --profile storage up -d garage
scripts/garage-bootstrap.sh .env.dev
```

## Backend Code Changes

**NONE REQUIRED** - The backend is already S3-compatible:
- `/Users/hmziq/os/ruxlog/backend/api/src/main.rs` (lines 91-141) - Uses generic S3 endpoint config
- `/Users/hmziq/os/ruxlog/backend/api/src/state.rs` - Storage-agnostic ObjectStorageConfig
- `/Users/hmziq/os/ruxlog/backend/api/src/modules/media_v1/controller.rs` - Standard S3 operations

Optional: Update comment in `state.rs` line 11:
```rust
// S3-compatible storage (Cloudflare R2, RustFS, AWS S3, etc.)
// Was: // S3-compatible storage (Cloudflare R2, Garage, AWS S3, etc.)
```

## Data Migration

**Not needed** - greenfield project with no existing data in Garage.

## Notes

- **Production** (`.env.prod`) uses Cloudflare R2 - no changes needed
- RustFS default credentials: `rustfsadmin/rustfsadmin` (override via RUSTFS_ACCESS_KEY/SECRET_KEY env vars)
- RustFS runs as non-root (UID 10001) - Docker named volumes handle permissions automatically
- Apache 2.0 license (more permissive than Garage's AGPL)
- RustFS supports clustering for high availability (future consideration)
- No MinIO dependency - bucket creation uses S3 REST API directly via curl

## Success Criteria

✅ All environments (dev/test/stage) start successfully with RustFS
✅ Media upload/delete operations work via backend API
✅ Files accessible via public URLs
✅ No Garage references remain in codebase
✅ Bootstrap script initializes buckets automatically
✅ Zero backend code changes required
