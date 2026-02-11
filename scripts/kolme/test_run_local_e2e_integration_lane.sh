#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_e2e_integration_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_ERR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$RUNNER" ]; then
  echo "expected Kolme local e2e integration lane runner to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_e2e_integration_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local e2e integration lane runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run e2e lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run lane mode marker"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only marker for e2e lane"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.local-e2e-integration-summary.v1":
    raise SystemExit("unexpected local e2e integration summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run e2e mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status for dry-run e2e summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in e2e summary")
checkpoints = report.get("checkpoints")
if not isinstance(checkpoints, list) or len(checkpoints) < 5:
    raise SystemExit("expected deterministic checkpoint entries in e2e summary")
checkpoint_ids = [
    entry.get("id")
    for entry in checkpoints
    if isinstance(entry, dict)
]
if "runtime_commit_adapter" not in checkpoint_ids:
    raise SystemExit("expected runtime_commit_adapter checkpoint id")
if "sdk_live_transport_parity" not in checkpoint_ids:
    raise SystemExit("expected sdk_live_transport_parity checkpoint id")
if "fork_rust_test_matrix" not in checkpoint_ids:
    raise SystemExit("expected fork_rust_test_matrix checkpoint id")
if "fork_live_api_conformance" not in checkpoint_ids:
    raise SystemExit("expected fork_live_api_conformance checkpoint id")
PY

set +e
bash "$RUNNER" --mode run --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

# Regression: #1418
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected e2e lane run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic opt-in failure message for e2e lane run mode" >&2
  exit 1
fi

echo "Kolme local e2e integration lane script tests passed."
