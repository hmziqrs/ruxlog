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
  echo "[garage-bootstrap] Unable to find env file: ${ENV_PATH}" >&2
  exit 1
fi
set -a
# shellcheck disable=SC1090
source "${ENV_PATH}"
set +a
cd "${PROJECT_ROOT}" >/dev/null 2>&1
COMPOSE_CMD=(docker compose --env-file "${ENV_PATH}")
"${COMPOSE_CMD[@]}" --profile storage up -d garage >/dev/null
GARAGE_BASE=("${COMPOSE_CMD[@]}" exec -T garage /garage --config /etc/garage/garage.toml --rpc-secret "${GARAGE_RPC_SECRET}" --admin-token "${GARAGE_ADMIN_TOKEN}")
READY=0
for _ in {1..15}; do
  if "${GARAGE_BASE[@]}" status >/dev/null 2>&1; then
    READY=1
    break
  fi
  sleep 2
done
if [[ "${READY}" -ne 1 ]]; then
  echo "[garage-bootstrap] garage service failed to become healthy" >&2
  exit 1
fi
NODE_LINE=$("${GARAGE_BASE[@]}" node id --quiet)
NODE_ID=$(echo "${NODE_LINE}" | grep -Eo '[0-9a-f]{16,}' | head -n 1 || true)
if [[ -z "${NODE_ID}" ]]; then
  echo "[garage-bootstrap] Failed to determine node id" >&2
  exit 1
fi
CURRENT_VERSION=$("${GARAGE_BASE[@]}" layout show | awk -F': ' '/Current cluster layout version/ {print $2}' | tr -d '\r\n' || true)
if [[ -z "${CURRENT_VERSION}" ]]; then
  CURRENT_VERSION="0"
fi
if [[ "${CURRENT_VERSION}" == "0" ]]; then
  echo "[garage-bootstrap] Assigning layout to node ${NODE_ID}" >&2
  "${GARAGE_BASE[@]}" layout assign -z "${GARAGE_ZONE:-dc1}" -c "${GARAGE_CAPACITY:-5G}" "${NODE_ID}" >/dev/null
  "${GARAGE_BASE[@]}" layout apply --version 1 >/dev/null
else
  echo "[garage-bootstrap] Layout already applied (version ${CURRENT_VERSION})" >&2
fi
if ! "${GARAGE_BASE[@]}" bucket info "${R2_BUCKET}" >/dev/null 2>&1; then
  echo "[garage-bootstrap] Creating bucket ${R2_BUCKET}" >&2
  "${GARAGE_BASE[@]}" bucket create "${R2_BUCKET}" >/dev/null
else
  echo "[garage-bootstrap] Bucket ${R2_BUCKET} already exists" >&2
fi
if ! "${GARAGE_BASE[@]}" key info "${GARAGE_ACCESS_KEY_NAME}" >/dev/null 2>&1; then
  echo "[garage-bootstrap] Importing key ${GARAGE_ACCESS_KEY_NAME}" >&2
  "${GARAGE_BASE[@]}" key import "${R2_ACCESS_KEY}" "${R2_SECRET_KEY}" --yes -n "${GARAGE_ACCESS_KEY_NAME}" >/dev/null
else
  echo "[garage-bootstrap] Key ${GARAGE_ACCESS_KEY_NAME} already exists" >&2
fi
set +e
"${GARAGE_BASE[@]}" bucket allow --read --write --owner "${R2_BUCKET}" --key "${GARAGE_ACCESS_KEY_NAME}" >/dev/null 2>&1 || true
set -e
echo "[garage-bootstrap] Garage bucket + key ready" >&2
