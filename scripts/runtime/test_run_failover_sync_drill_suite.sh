#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUITE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_suite.sh"
TMP_REPORT="$(mktemp)"
TMP_GITHUB_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_GITHUB_OUTPUT"' EXIT

if [ ! -x "$SUITE" ]; then
  echo "expected failover/sync drill suite script to be executable" >&2
  exit 1
fi

preflight_output="$(
  bash "$SUITE" \
    --event-name pull_request \
    --skip-suite \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$preflight_output" | grep -q "failover/sync drill suite tests passed."; then
  echo "expected failover/sync suite preflight success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.failover-sync-drill-suite-report.v1":
    raise SystemExit("unexpected failover/sync suite report schema")
if payload.get("selected_lane") != "preflight":
    raise SystemExit("expected preflight lane for pull_request event")
if payload.get("status") != "pass":
    raise SystemExit("expected preflight suite status to pass")
PY

deep_output="$(
  bash "$SUITE" \
    --event-name schedule \
    --skip-suite \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$deep_output" | grep -q "failover/sync drill suite tests passed."; then
  echo "expected failover/sync suite deep success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("selected_lane") != "deep":
    raise SystemExit("expected deep lane for schedule event")
if payload.get("status") != "pass":
    raise SystemExit("expected deep suite status to pass")
lane_report = payload.get("lane_report", {})
if lane_report.get("lane") != "deep":
    raise SystemExit("expected deep lane report payload")
PY

ci_output="$(
  GITHUB_OUTPUT="$TMP_GITHUB_OUTPUT" \
    bash "$SUITE" \
      --event-name schedule \
      --skip-suite \
      --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$ci_output" | grep -q "failover/sync drill suite tests passed."; then
  echo "expected failover/sync suite success marker under GitHub output env" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("selected_lane") != "deep":
    raise SystemExit("expected deep lane under GitHub output env")
if payload.get("status") != "pass":
    raise SystemExit("expected deep suite status to pass under GitHub output env")
PY

echo "failover/sync suite script tests passed."
