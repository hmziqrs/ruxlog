#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_DIR="${ROOT_DIR}/observability/quickwit/config"
INDEX_DIR="${CONFIG_DIR}/indexes"

QW_BIN="${QUICKWIT_BIN:-quickwit}"
API_URI="${QUICKWIT_API_URL:-http://localhost:7280}"
QW_CONFIG="${QUICKWIT_CONFIG:-/quickwit/config/quickwit.yaml}"

if ! ${QW_BIN} --config "${QW_CONFIG}" --version >/dev/null 2>&1; then
  echo "quickwit CLI not found. Install from https://quickwit.io/docs/reference/cli." >&2
  echo "(Tried to run: ${QW_BIN} --config ${QW_CONFIG} --version)" >&2
  exit 1
fi

if [[ ! -d "${INDEX_DIR}" ]]; then
  echo "No index definitions found under ${INDEX_DIR}." >&2
  exit 1
fi

for index_file in "${INDEX_DIR}"/*.yaml; do
  [[ -f "${index_file}" ]] || continue

  echo "Applying index config: ${index_file}" >&2
  if ${QW_BIN} --config "${QW_CONFIG}" index create \
    --index-config "${index_file}"; then
    echo "✓ Created $(basename "${index_file}")" >&2
    continue
  fi

  echo "Index may already exist, attempting update..." >&2
  ${QW_BIN} --config "${QW_CONFIG}" index update \
    --index-config "${index_file}"
  echo "✓ Updated $(basename "${index_file}")" >&2

done

echo "Quickwit indexes are ready." >&2
