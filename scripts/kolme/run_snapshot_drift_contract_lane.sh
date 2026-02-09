#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_CHECKER="$ROOT_DIR/scripts/kolme/test_check_snapshot_drift.sh"
DOC_FILE="$ROOT_DIR/docs/research/kolme-upstream-compatibility.md"

if [ ! -x "$TEST_CHECKER" ]; then
  echo "expected Kolme snapshot drift checker test script to be executable" >&2
  exit 1
fi

if [ ! -f "$DOC_FILE" ]; then
  echo "expected Kolme upstream compatibility research doc to exist" >&2
  exit 1
fi

start_epoch="$(date +%s)"

bash "$TEST_CHECKER"

if ! grep -q "check_snapshot_drift.py" "$DOC_FILE"; then
  echo "expected Kolme compatibility doc to reference drift checker command" >&2
  exit 1
fi

if ! grep -q "run_snapshot_drift_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme compatibility doc to reference contract lane command" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 45 ]; then
  echo "Kolme snapshot drift contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "Kolme snapshot drift contract lane tests passed."
