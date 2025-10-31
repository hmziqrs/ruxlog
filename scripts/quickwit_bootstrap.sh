#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONFIG_DIR="${ROOT_DIR}/observability/quickwit/config"
INDEX_DIR="${CONFIG_DIR}/indexes"

QW_BIN_RAW="${QUICKWIT_BIN:-quickwit}"
read -r -a QW_BIN_CMD <<<"${QW_BIN_RAW}"
API_URI="${QUICKWIT_API_URL:-http://localhost:7280}"

if ! "${QW_BIN_CMD[@]}" --version >/dev/null 2>&1; then
  echo "quickwit CLI not found. Install from https://quickwit.io/docs/reference/cli." >&2
  echo "(Tried to run: ${QW_BIN_RAW} --version)" >&2
  exit 1
fi

if [[ ! -d "${INDEX_DIR}" ]]; then
  echo "No index definitions found under ${INDEX_DIR}." >&2
  exit 1
fi

for index_file in "${INDEX_DIR}"/*.yaml; do
  [[ -f "${index_file}" ]] || continue

  echo "Applying index config: ${index_file}" >&2
  if "${QW_BIN_CMD[@]}" index create \
    --metastore-uri "${API_URI}" \
    --index-config "${index_file}"; then
    echo "✓ Created $(basename "${index_file}")" >&2
    continue
  fi

  echo "Index may already exist, attempting update..." >&2
  "${QW_BIN_CMD[@]}" index update \
    --metastore-uri "${API_URI}" \
    --index-config "${index_file}"
  echo "✓ Updated $(basename "${index_file}")" >&2

done

echo "Quickwit indexes are ready." >&2
