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

# Create bucket using AWS CLI with proper S3 authentication
echo "[rustfs-bootstrap] Setting up bucket ${bucket_name}" >&2

# Get the docker network name for the project
network_name="${PROJECT}_ruxlog"

# Create bucket using AWS CLI Docker image (supports AWS Signature V4)
docker run --rm --network "${network_name}" \
  -e AWS_ACCESS_KEY_ID="${access_key}" \
  -e AWS_SECRET_ACCESS_KEY="${secret_key}" \
  amazon/aws-cli \
  --endpoint-url http://rustfs:9000 \
  s3 mb "s3://${bucket_name}" 2>&1 | grep -v "BucketAlreadyOwnedByYou" || {
    echo "[rustfs-bootstrap] Bucket '${bucket_name}' ready (created or already exists)" >&2
  }

echo "[rustfs-bootstrap] RustFS ready!" >&2
echo "[rustfs-bootstrap] Console: http://localhost:${RUSTFS_CONSOLE_PORT:-1106}" >&2
echo "[rustfs-bootstrap] Credentials: ${access_key}" >&2
