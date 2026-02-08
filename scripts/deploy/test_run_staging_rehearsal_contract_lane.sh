#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAST_SCRIPT="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_contract_lane.sh"
DEEP_SCRIPT="$ROOT_DIR/scripts/deploy/run_staging_rehearsal_deep_lane.sh"

if [ ! -x "$FAST_SCRIPT" ]; then
  echo "expected staging rehearsal fast-lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_SCRIPT" ]; then
  echo "expected staging rehearsal deep-lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$FAST_SCRIPT" >"$TMP_OUT"
if ! grep -q "staging rehearsal contract lane tests passed." "$TMP_OUT"; then
  echo "expected staging rehearsal contract lane success marker" >&2
  exit 1
fi

if ! grep -Fq "run_staging_rehearsal_contract_lane.sh" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to execute fast-lane rehearsal checks first" >&2
  exit 1
fi

if ! grep -q "staging-rehearsal-report.json" "$DEEP_SCRIPT"; then
  echo "expected deep-lane script to emit staging rehearsal report artifact" >&2
  exit 1
fi

echo "staging rehearsal contract lane script tests passed."
