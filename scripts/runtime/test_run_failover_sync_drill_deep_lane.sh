#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_deep_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected failover/sync deep lane script to be executable" >&2
  exit 1
fi

set +e
unscheduled_output="$(
  bash "$DEEP_LANE" --skip-suite --output-json "$TMP_REPORT" 2>&1
)"
unscheduled_code=$?
set -e

if [ "$unscheduled_code" -eq 0 ]; then
  echo "expected deep lane cadence guard to reject unscheduled execution" >&2
  exit 1
fi

if ! printf '%s\n' "$unscheduled_output" | grep -q "scheduled-only"; then
  echo "expected deep lane cadence rejection marker" >&2
  exit 1
fi

scheduled_output="$(
  KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled \
    bash "$DEEP_LANE" --skip-suite --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$scheduled_output" | grep -q "failover/sync deep lane tests passed."; then
  echo "expected failover/sync deep lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-report.v1":
    raise SystemExit("unexpected failover/sync deep report schema")
if payload.get("lane") != "deep":
    raise SystemExit("expected deep lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected deep lane to pass under scheduled cadence")
PY

echo "failover/sync deep lane script tests passed."
