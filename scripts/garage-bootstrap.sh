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
GARAGE_CMD=("${COMPOSE_CMD[@]}" exec -T garage /garage --config /etc/garage/garage.toml --rpc-secret "${GARAGE_RPC_SECRET}" --admin-token "${GARAGE_ADMIN_TOKEN}")

bucket_name="${S3_BUCKET:-${AWS_S3_BUCKET:-${GARAGE_BUCKET:-}}}"
key_name="${GARAGE_ACCESS_KEY_NAME:-${bucket_name}}"
requested_key_id="${S3_ACCESS_KEY:-${AWS_ACCESS_KEY_ID:-}}"
requested_key_secret="${S3_SECRET_KEY:-${AWS_SECRET_ACCESS_KEY:-}}"

upsert_env_var() {
  local var_name="$1"
  local var_value="$2"
  local file_path="$3"

  if grep -q "^${var_name}=" "${file_path}"; then
    sed -i.bak "s|^${var_name}=.*|${var_name}=${var_value}|" "${file_path}"
  else
    echo "${var_name}=${var_value}" >>"${file_path}"
  fi
}

echo "[garage-bootstrap] Ensuring Garage is running via docker compose (env: ${ENV_PATH})" >&2
"${COMPOSE_CMD[@]}" --profile storage up -d garage >/dev/null

ready=0
for _ in {1..30}; do
  if "${GARAGE_CMD[@]}" status >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 2
done

if [[ "${ready}" -ne 1 ]]; then
  echo "[garage-bootstrap] Garage service failed to become healthy" >&2
  exit 1
fi

layout_status="$("${GARAGE_CMD[@]}" status 2>&1 || true)"
node_id="$("${GARAGE_CMD[@]}" node id 2>&1 | grep -Eo '^[0-9a-f]{16,}' | head -n 1 || true)"

if [[ -z "${node_id}" ]]; then
  echo "[garage-bootstrap] Failed to determine node id" >&2
  exit 1
fi

current_version="$("${GARAGE_CMD[@]}" layout show 2>/dev/null | awk -F': ' '/Current cluster layout version/ {print $2}' | tr -d '\r\n' || true)"
if echo "${layout_status}" | grep -qi "NO ROLE ASSIGNED" || [[ -z "${current_version}" || "${current_version}" == "0" ]]; then
  echo "[garage-bootstrap] Assigning layout to node ${node_id}" >&2
  "${GARAGE_CMD[@]}" layout assign "${node_id}" -z "${GARAGE_ZONE:-dc1}" -c "${GARAGE_CAPACITY:-5G}" -t "${GARAGE_NODE_TAG:-local}" >/dev/null
  "${GARAGE_CMD[@]}" layout apply --version 1 >/dev/null 2>&1 || "${GARAGE_CMD[@]}" layout apply >/dev/null
else
  echo "[garage-bootstrap] Layout already applied (version ${current_version})" >&2
fi

if [[ -z "${bucket_name}" ]]; then
  echo "[garage-bootstrap] S3/Garage bucket name is not set (S3_BUCKET/AWS_S3_BUCKET/GARAGE_BUCKET)" >&2
  exit 1
fi

bucket_info="$("${GARAGE_CMD[@]}" bucket info "${bucket_name}" 2>&1 || true)"
if echo "${bucket_info}" | grep -qiE "not found|does not exist|Unknown bucket"; then
  echo "[garage-bootstrap] Creating bucket ${bucket_name}" >&2
  "${GARAGE_CMD[@]}" bucket create "${bucket_name}" >/dev/null
else
  echo "[garage-bootstrap] Bucket ${bucket_name} already exists" >&2
fi

key_list="$("${GARAGE_CMD[@]}" key list 2>&1 || true)"
existing_key_id=""

if [[ -n "${requested_key_id}" ]]; then
  existing_key_id=$(echo "${key_list}" | awk -v key_id="${requested_key_id}" '$1==key_id {print $1; exit}')
fi

if [[ -z "${existing_key_id}" && -n "${key_name}" ]]; then
  existing_key_id=$(echo "${key_list}" | awk -v name="${key_name}" '$2==name {print $1; exit}')
fi

key_id="${existing_key_id}"
key_secret=""

if [[ -n "${existing_key_id}" ]]; then
  echo "[garage-bootstrap] Key ${key_name} already exists (access key ${existing_key_id})" >&2
else
  if [[ -n "${requested_key_id}" && -n "${requested_key_secret}" ]]; then
    echo "[garage-bootstrap] Importing key ${key_name}" >&2
    "${GARAGE_CMD[@]}" key import "${requested_key_id}" "${requested_key_secret}" --yes -n "${key_name}" >/dev/null
    key_id="${requested_key_id}"
    key_secret="${requested_key_secret}"
  else
    echo "[garage-bootstrap] Creating new key ${key_name}" >&2
    key_output="$("${GARAGE_CMD[@]}" key create "${key_name}" 2>&1 || true)"
    key_id=$(echo "${key_output}" | awk '/Key ID:/ {print $3; exit}')
    key_secret=$(echo "${key_output}" | awk '/Secret key:/ {print $3; exit}')

    if [[ -z "${key_id}" || -z "${key_secret}" ]]; then
      echo "[garage-bootstrap] Unable to parse key creation output" >&2
      echo "${key_output}" >&2
      exit 1
    fi
  fi
fi

if [[ -n "${key_secret}" ]]; then
  upsert_env_var "S3_ACCESS_KEY" "${key_id}" "${ENV_PATH}"
  upsert_env_var "S3_SECRET_KEY" "${key_secret}" "${ENV_PATH}"
  upsert_env_var "AWS_ACCESS_KEY_ID" "${key_id}" "${ENV_PATH}"
  upsert_env_var "AWS_SECRET_ACCESS_KEY" "${key_secret}" "${ENV_PATH}"
  rm -f "${ENV_PATH}.bak"
else
  echo "[garage-bootstrap] Key secret not available; ensure your env file contains the correct credentials" >&2
fi

if [[ -n "${key_id}" ]]; then
  allow_output="$("${GARAGE_CMD[@]}" bucket allow --read --write --owner "${bucket_name}" --key "${key_id}" 2>&1 || true)"
  if echo "${allow_output}" | grep -qi "already"; then
    echo "[garage-bootstrap] Permissions already configured for ${bucket_name}/${key_name}" >&2
  elif echo "${allow_output}" | grep -qi "error"; then
    echo "[garage-bootstrap] Unexpected response while setting permissions:" >&2
    echo "${allow_output}" >&2
  else
    echo "[garage-bootstrap] Permissions granted for ${bucket_name}/${key_name}" >&2
  fi
else
  echo "[garage-bootstrap] No key id available; skipping permission grant" >&2
fi

echo "[garage-bootstrap] Bucket information:" >&2
"${GARAGE_CMD[@]}" bucket info "${bucket_name}" | head -20

if [[ -n "${key_id}" ]]; then
  echo "[garage-bootstrap] Key information:" >&2
  "${GARAGE_CMD[@]}" key info "${key_id}" 2>&1 | head -20
fi

echo "[garage-bootstrap] Garage bucket + key ready" >&2
