#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REQUESTED_ENV="${1:-.env.test}"
if [[ "${REQUESTED_ENV}" = /* ]]; then
  ENV_PATH="${REQUESTED_ENV}"
else
  ENV_PATH="${PROJECT_ROOT}/${REQUESTED_ENV}"
fi
if [[ ! -f "${ENV_PATH}" ]]; then
  echo "[test-db-setup] Unable to find env file: ${ENV_PATH}" >&2
  exit 1
fi
set -a
# shellcheck disable=SC1090
source "${ENV_PATH}"
set +a
cd "${PROJECT_ROOT}" >/dev/null 2>&1
COMPOSE_CMD=(docker compose --env-file "${ENV_PATH}")
"${COMPOSE_CMD[@]}" --profile services --profile storage down --volumes --remove-orphans >/dev/null 2>&1 || true
"${COMPOSE_CMD[@]}" --profile services --profile storage up -d >/dev/null
echo "[test-db-setup] Waiting for Postgres to accept connections..."
READY=0
for _ in {1..30}; do
  if "${COMPOSE_CMD[@]}" exec -T postgres pg_isready -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" >/dev/null 2>&1; then
    READY=1
    break
  fi
  sleep 2
done
if [[ "${READY}" -ne 1 ]]; then
  echo "[test-db-setup] Postgres did not become ready" >&2
  exit 1
fi
echo "[test-db-setup] Resetting database ${POSTGRES_DB}" >&2
"${COMPOSE_CMD[@]}" exec -T postgres env PGPASSWORD="${POSTGRES_PASSWORD}" psql -U "${POSTGRES_USER}" -d postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='${POSTGRES_DB}' AND pid <> pg_backend_pid();" >/dev/null || true
"${COMPOSE_CMD[@]}" exec -T postgres env PGPASSWORD="${POSTGRES_PASSWORD}" psql -U "${POSTGRES_USER}" -d postgres -c "DROP DATABASE IF EXISTS \"${POSTGRES_DB}\";" >/dev/null
"${COMPOSE_CMD[@]}" exec -T postgres env PGPASSWORD="${POSTGRES_PASSWORD}" psql -U "${POSTGRES_USER}" -d postgres -c "CREATE DATABASE \"${POSTGRES_DB}\" OWNER \"${POSTGRES_USER}\";" >/dev/null

echo "[test-db-setup] Running migrations" >&2
pushd "${PROJECT_ROOT}/backend/api" >/dev/null 2>&1
cargo run --manifest-path migration/Cargo.toml --bin migrate >/dev/null
popd >/dev/null 2>&1

"${SCRIPT_DIR}/garage-bootstrap.sh" "${REQUESTED_ENV}" >/dev/null

echo "[test-db-setup] Test database + Garage bucket ready" >&2
