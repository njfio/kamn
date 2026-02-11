#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HELPER="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_ERR"' EXIT

if [ ! -x "$HELPER" ]; then
  echo "expected local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

set +e
"$HELPER" >"$TMP_ERR" 2>&1
guard_without_opt_in_code=$?
set -e

if [ "$guard_without_opt_in_code" -eq 0 ]; then
  echo "expected local-heavy opt-in guard helper to fail when opt-in is missing" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic guard failure message when opt-in is missing" >&2
  exit 1
fi

KAMN_KOLME_LOCAL_HEAVY=1 "$HELPER" >/dev/null

echo "local-heavy opt-in guard helper tests passed."
