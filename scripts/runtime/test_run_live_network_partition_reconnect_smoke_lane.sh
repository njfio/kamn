#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_LANE="$ROOT_DIR/scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$SMOKE_LANE" ]; then
  echo "expected partition/reconnect smoke lane script to be executable" >&2
  exit 1
fi

lane_output="$(bash "$SMOKE_LANE" --event-name pull_request --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "live-network partition/reconnect smoke lane tests passed."; then
  echo "expected partition/reconnect smoke lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-network-partition-reconnect-matrix-report.v1":
    raise SystemExit("unexpected partition/reconnect smoke report schema")
if payload.get("lane") != "smoke":
    raise SystemExit("expected smoke lane report")
if payload.get("status") != "pass":
    raise SystemExit("expected smoke lane status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected smoke lane final_decision=GO")
required = payload.get("required_scenarios", [])
if required != [
    "fixture_contract",
    "primary_loss_reconnect_catchup",
    "three_process_failover",
]:
    raise SystemExit("unexpected smoke required scenario set")
PY

set +e
failed_output="$(
  bash "$SMOKE_LANE" \
    --event-name pull_request \
    --output-json "$TMP_REPORT" \
    --fail-scenarios three_process_failover 2>&1
)"
failed_code=$?
set -e

if [ "$failed_code" -eq 0 ]; then
  echo "expected partition/reconnect smoke lane to fail with injected scenario failure" >&2
  exit 1
fi

if ! printf '%s\n' "$failed_output" | grep -q "scenario_three_process_failover_failed"; then
  echo "expected injected scenario failure reason code in smoke lane output" >&2
  exit 1
fi

# Regression: #982
set +e
budget_output="$(
  bash "$SMOKE_LANE" \
    --event-name pull_request \
    --output-json "$TMP_REPORT" \
    --simulate-delay-seconds 1 \
    --max-seconds 0 2>&1
)"
budget_code=$?
set -e

if [ "$budget_code" -eq 0 ]; then
  echo "expected partition/reconnect smoke lane budget guard to fail over-budget runs" >&2
  exit 1
fi

if ! printf '%s\n' "$budget_output" | grep -q "runtime_budget_exceeded"; then
  echo "expected runtime budget regression reason code for partition/reconnect smoke lane" >&2
  exit 1
fi

echo "partition/reconnect smoke lane script tests passed."
