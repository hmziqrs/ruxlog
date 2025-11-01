#!/usr/bin/env bash
# Validate Quickwit setup - checks files, API health, and indexes
set -euo pipefail

API_URL="${QUICKWIT_API_URL:-http://localhost:7280}"
CONTAINER_NAME="${CONTAINER_NAME:-ruxlog-quickwit}"
BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BOLD}=== Quickwit Setup Validation ===${NC}\n"

# Check 1: Docker container is running
echo -e "${BOLD}1. Checking if Quickwit container is running...${NC}"
if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo -e "${GREEN}✓${NC} Container '${CONTAINER_NAME}' is running"
else
  echo -e "${RED}✗${NC} Container '${CONTAINER_NAME}' is NOT running"
  echo "   Run: docker compose -f docker-compose.observability.yml up -d quickwit"
  exit 1
fi

# Check 2: Config file exists on host
echo -e "\n${BOLD}2. Checking main config file...${NC}"
if [ -f "./observability/quickwit/config/quickwit.yaml" ]; then
  echo -e "${GREEN}✓${NC} Config file exists: ./observability/quickwit/config/quickwit.yaml"
else
  echo -e "${RED}✗${NC} Config file NOT found: ./observability/quickwit/config/quickwit.yaml"
  exit 1
fi

# Check 3: Index config files exist on host
echo -e "\n${BOLD}3. Checking index configuration files...${NC}"
INDEX_FILES=$(ls -1 ./observability/quickwit/config/indexes/*.yaml 2>/dev/null | wc -l | tr -d ' ')
if [ "${INDEX_FILES}" -gt 0 ]; then
  echo -e "${GREEN}✓${NC} Found ${INDEX_FILES} index configuration file(s):"
  ls -1 ./observability/quickwit/config/indexes/*.yaml | while read -r file; do
    echo "   - $(basename "$file")"
  done
else
  echo -e "${RED}✗${NC} No index configuration files found in ./observability/quickwit/config/indexes/"
  exit 1
fi

# Check 4: Quickwit API version
echo -e "\n${BOLD}4. Checking Quickwit API version...${NC}"
if VERSION_JSON=$(curl -sf "${API_URL}/api/v1/version" 2>/dev/null); then
  VERSION=$(echo "${VERSION_JSON}" | grep -o '"version":"[^"]*"' | cut -d'"' -f4)
  BUILD_DATE=$(echo "${VERSION_JSON}" | grep -o '"build_date":"[^"]*"' | cut -d'"' -f4)
  echo -e "${GREEN}✓${NC} Quickwit API is responding: ${VERSION} (${BUILD_DATE})"
else
  echo -e "${RED}✗${NC} Quickwit API not responding at ${API_URL}"
  echo "   Check logs: docker logs ${CONTAINER_NAME}"
  exit 1
fi

# Check 5: API health endpoints
echo -e "\n${BOLD}5. Checking API health endpoints...${NC}"
if curl -sf "${API_URL}/health/livez" >/dev/null 2>&1; then
  echo -e "${GREEN}✓${NC} Liveness probe: ${API_URL}/health/livez"
else
  echo -e "${RED}✗${NC} Liveness probe failed"
fi

if curl -sf "${API_URL}/health/readyz" >/dev/null 2>&1; then
  echo -e "${GREEN}✓${NC} Readiness probe: ${API_URL}/health/readyz"
else
  echo -e "${YELLOW}⚠${NC} Readiness probe check failed (may still be initializing)"
fi

# Check 6: List existing indexes via API
echo -e "\n${BOLD}6. Listing existing indexes via API...${NC}"
if INDEXES_JSON=$(curl -sf "${API_URL}/api/v1/indexes" 2>/dev/null); then
  # Extract unique index IDs (handles spaces in JSON)
  INDEX_LIST=$(echo "${INDEXES_JSON}" | grep -o '"index_id"[[:space:]]*:[[:space:]]*"[^"]*"' | cut -d'"' -f4 | sort | uniq)
  if [ -n "${INDEX_LIST}" ]; then
    INDEX_COUNT=$(echo "${INDEX_LIST}" | wc -l | tr -d ' ')
    echo -e "${GREEN}✓${NC} Found ${INDEX_COUNT} index(es):"
    echo "${INDEX_LIST}" | while read -r idx; do
      [ -n "${idx}" ] && echo "   - ${idx}"
    done
  else
    echo -e "${YELLOW}⚠${NC} No indexes created yet"
    echo "   Run: ./scripts/bootstrap_quickwit_manual.sh"
  fi
else
  echo -e "${YELLOW}⚠${NC} Unable to list indexes (API may not be fully ready)"
fi

# Check 7: MinIO connectivity (from host)
echo -e "\n${BOLD}7. Checking MinIO connectivity...${NC}"
if curl -sf http://localhost:9000/minio/health/live >/dev/null 2>&1; then
  echo -e "${GREEN}✓${NC} MinIO is accessible from host (http://localhost:9000)"
else
  echo -e "${YELLOW}⚠${NC} MinIO health check failed"
  echo "   Ensure MinIO container is running: docker ps | grep minio"
fi

# Check 8: MinIO connectivity from Quickwit container
if docker exec "${CONTAINER_NAME}" sh -c 'command -v curl >/dev/null 2>&1' >/dev/null 2>&1; then
  if docker exec "${CONTAINER_NAME}" curl -sf http://minio:9000/minio/health/live >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} MinIO is accessible from Quickwit container"
  else
    echo -e "${YELLOW}⚠${NC} MinIO not accessible from container (network issue?)"
  fi
fi

# Check 9: Bootstrap script exists
echo -e "\n${BOLD}8. Checking bootstrap script availability...${NC}"
if [ -f "./scripts/bootstrap_quickwit_manual.sh" ]; then
  echo -e "${GREEN}✓${NC} Bootstrap script found: ./scripts/bootstrap_quickwit_manual.sh"
else
  echo -e "${RED}✗${NC} Bootstrap script NOT found"
fi

echo -e "\n${BOLD}=== Validation Complete ===${NC}"
echo -e "\n${BOLD}Quick Commands:${NC}"
echo "  - View logs:       docker logs ${CONTAINER_NAME}"
echo "  - List indexes:    curl ${API_URL}/api/v1/indexes"
echo "  - Bootstrap:       ./scripts/bootstrap_quickwit_manual.sh"
echo "  - Web UI:          ${API_URL}/ui"
echo "  - MinIO Console:   http://localhost:9001 (quickwit/quickwit-secret)"
