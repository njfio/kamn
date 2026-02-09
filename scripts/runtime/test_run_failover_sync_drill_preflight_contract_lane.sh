#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PREFLIGHT_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$PREFLIGHT_LANE" ]; then
  echo "expected failover/sync preflight contract lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$PREFLIGHT_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "failover/sync preflight contract lane tests passed."; then
  echo "expected failover/sync preflight contract lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1":
    raise SystemExit("unexpected failover/sync preflight report schema")
if payload.get("lane") != "preflight":
    raise SystemExit("expected preflight lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected preflight lane to pass")
PY

set +e
over_budget_output="$(
  bash "$PREFLIGHT_LANE" \
    --skip-suite \
    --simulate-delay-seconds 1 \
    --max-seconds 0 \
    --output-json "$TMP_REPORT" 2>&1
)"
over_budget_code=$?
set -e

if [ "$over_budget_code" -eq 0 ]; then
  echo "expected failover/sync preflight budget guard to fail over-budget run" >&2
  exit 1
fi

# Regression: #788
if ! printf '%s\n' "$over_budget_output" | grep -q "exceeded runtime budget"; then
  echo "expected failover/sync preflight budget overrun signal" >&2
  exit 1
fi

echo "failover/sync preflight contract lane script tests passed."
