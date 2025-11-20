#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REQUESTED_ENV="${1:-${ENV_FILE:-.env.dev}}"
shift_args=0
if [[ $# -gt 0 ]]; then
  shift_args=1
fi
if [[ "${REQUESTED_ENV}" = /* ]]; then
  ENV_PATH="${REQUESTED_ENV}"
else
  ENV_PATH="${PROJECT_ROOT}/${REQUESTED_ENV}"
fi
if [[ ! -f "${ENV_PATH}" ]]; then
  echo "[compose-down] Unable to find env file: ${ENV_PATH}" >&2
  exit 1
fi
if [[ "${shift_args}" -eq 1 ]]; then
  shift
fi
cd "${PROJECT_ROOT}" >/dev/null 2>&1
docker compose --env-file "${ENV_PATH}" \
  --profile services \
  --profile storage \
  --profile full \
  down --remove-orphans "$@"
