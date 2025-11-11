#!/usr/bin/env bash
# Bootstrap Quickwit indexes via REST API (can be used in CI/CD or manually)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_DIR="${ROOT_DIR}/observability/quickwit/config"
INDEX_DIR="${CONFIG_DIR}/indexes"

API_URL="${QUICKWIT_API_URL:-http://localhost:7280}"

echo "Checking Quickwit API availability at ${API_URL}..." >&2

# Wait for API to be ready
for i in {1..30}; do
  if curl -sf "${API_URL}/api/v1/version" >/dev/null 2>&1; then
    break
  fi
  if [ $i -eq 30 ]; then
    echo "Error: Quickwit API not responding after 30 seconds" >&2
    exit 1
  fi
  sleep 1
done

echo "Quickwit API is ready" >&2

if [[ ! -d "${INDEX_DIR}" ]]; then
  echo "No index definitions found under ${INDEX_DIR}." >&2
  exit 1
fi

for index_file in "${INDEX_DIR}"/*.yaml; do
  [[ -f "${index_file}" ]] || continue

  index_name=$(basename "${index_file}" .yaml)
  echo "Applying index config: ${index_file}" >&2

  # Create index via REST API
  HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/qw_response.txt -X POST \
    "${API_URL}/api/v1/indexes" \
    --header "content-type: application/yaml" \
    --data-binary "@${index_file}" 2>&1)
  
  BODY=$(cat /tmp/qw_response.txt)
  
  if [ "${HTTP_CODE}" = "200" ]; then
    echo "✓ Created ${index_name}" >&2
    continue
  fi

  # Check if index already exists
  if echo "${BODY}" | grep -qi "already exist"; then
    echo "Index ${index_name} already exists, skipping..." >&2
    continue
  fi

  # Unexpected error
  echo "✗ Failed to create ${index_name} (HTTP ${HTTP_CODE})" >&2
  echo "${BODY}" >&2
  exit 1
done

echo "Quickwit indexes are ready." >&2
