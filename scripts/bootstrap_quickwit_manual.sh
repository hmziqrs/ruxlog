#!/usr/bin/env bash
# Manually bootstrap Quickwit indexes via REST API
set -euo pipefail

API_URL="${QUICKWIT_API_URL:-http://localhost:7280}"
INDEX_DIR="./observability/quickwit/config/indexes"
BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BOLD}=== Quickwit Index Bootstrap (REST API) ===${NC}\n"

# Wait for Quickwit API to be ready
echo -e "${BOLD}Waiting for Quickwit API to be ready...${NC}"
for i in {1..30}; do
  if curl -sf "${API_URL}/api/v1/version" >/dev/null 2>&1; then
    VERSION=$(curl -s "${API_URL}/api/v1/version" | grep -o '"version":"[^"]*"' | cut -d'"' -f4)
    echo -e "${GREEN}✓${NC} Quickwit API is ready (${VERSION})"
    break
  fi
  if [ $i -eq 30 ]; then
    echo -e "${RED}✗${NC} Quickwit API not responding after 30 seconds"
    echo "   Check if container is running: docker ps | grep quickwit"
    echo "   Check logs: docker logs ruxlog-quickwit"
    exit 1
  fi
  echo -n "."
  sleep 1
done
echo ""

# Check if index directory exists
if [ ! -d "${INDEX_DIR}" ]; then
  echo -e "${RED}✗${NC} Index config directory not found: ${INDEX_DIR}"
  exit 1
fi

# Create indexes from config files using REST API
echo -e "\n${BOLD}Creating indexes from configuration files...${NC}\n"

for index_file in "${INDEX_DIR}"/*.yaml; do
  if [ -f "${index_file}" ]; then
    index_name=$(basename "${index_file}" .yaml)
    echo -e "${BOLD}Processing:${NC} ${index_name}"
    
    # Create index via REST API
    HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/qw_response.txt -X POST \
      "${API_URL}/api/v1/indexes" \
      --header "content-type: application/yaml" \
      --data-binary "@${index_file}" 2>&1)
    
    BODY=$(cat /tmp/qw_response.txt)
    
    if [ "${HTTP_CODE}" = "200" ]; then
      echo -e "${GREEN}✓${NC} Created index: ${index_name}"
    elif echo "${BODY}" | grep -qi "already exist"; then
      echo -e "${YELLOW}⚠${NC} Index already exists: ${index_name}, skipping..."
    else
      echo -e "${RED}✗${NC} Failed to create index: ${index_name} (HTTP ${HTTP_CODE})"
      echo "${BODY}"
    fi
    echo ""
  fi
done

# List all indexes
echo -e "${BOLD}=== Listing Created Indexes ===${NC}\n"
INDEXES_JSON=$(curl -s "${API_URL}/api/v1/indexes" 2>/dev/null || echo "[]")

if [ "${INDEXES_JSON}" != "[]" ] && [ -n "${INDEXES_JSON}" ]; then
  # Extract index IDs (handles spaces in JSON)
  echo "${INDEXES_JSON}" | grep -o '"index_id"[[:space:]]*:[[:space:]]*"[^"]*"' | cut -d'"' -f4 | sort | uniq | while read -r idx; do
    [ -n "${idx}" ] && echo "  - ${idx}"
  done
else
  echo -e "${YELLOW}⚠${NC} No indexes found"
fi

echo -e "\n${GREEN}✓${NC} ${BOLD}Bootstrap completed!${NC}"
echo -e "\nYou can now:"
echo "  - List indexes:  curl ${API_URL}/api/v1/indexes"
echo "  - Query logs:    curl '${API_URL}/api/v1/ruxlog-logs/search?query=*'"
echo "  - Web UI:        ${API_URL}/ui"
