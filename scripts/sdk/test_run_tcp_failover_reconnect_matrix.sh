#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_tcp_failover_reconnect_matrix.sh"
PY_RUNNER="$ROOT_DIR/scripts/sdk/run_tcp_failover_reconnect_matrix.py"
FIXTURE="$ROOT_DIR/fixtures/sdk_failover_reconnect/reconnect_drift_signatures.txt"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected tcp failover reconnect matrix runner to be executable" >&2
  exit 1
fi

report="$TMP_DIR/failover-reconnect-report.json"
run_output="$(
  bash "$RUNNER" \
    --fixture "$FIXTURE" \
    --lane fast \
    --output-json "$report"
)"

if ! printf '%s\n' "$run_output" | grep -q "^status=pass; lane=fast;"; then
  echo "expected pass status from tcp failover reconnect matrix fast lane" >&2
  exit 1
fi

python3 - "$report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.sdk.tcp-failover-reconnect.matrix.v1"
assert report["status"] == "pass"
assert report["lane"] == "fast"
assert report["fixture_case_count"] == 3
assert report["scenario_count"] == 5
assert report["failed_count"] == 0
scenario_names = [entry["name"] for entry in report["results"]]
assert "primary_loss_reconnect_catchup" in scenario_names
assert "three_process_failover" in scenario_names
assert "reconnect_drift_regression" in scenario_names
PY

bounded_output="$(
  bash "$RUNNER" \
    --fixture "$FIXTURE" \
    --lane fast \
    --max-cases 2
)"
if ! printf '%s\n' "$bounded_output" | grep -q "^status=pass; lane=fast; scenarios=2; failed=0;"; then
  echo "expected bounded tcp failover reconnect matrix subset to pass" >&2
  exit 1
fi

if ! grep -Fq "performance_tcp_failover_reconnect_matrix_deep_lane" "$PY_RUNNER"; then
  echo "expected tcp failover reconnect matrix runner to define deep-lane scenario" >&2
  exit 1
fi

echo "tcp failover reconnect matrix script tests passed."
