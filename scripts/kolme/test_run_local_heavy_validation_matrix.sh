#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_local_heavy_validation_matrix.sh"
BOOTSTRAP_RUNNER="$ROOT_DIR/scripts/kolme/run_local_bootstrap_health_checks.sh"
SUMMARY_HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

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

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected Kolme local heavy validation matrix runner to be executable" >&2
  exit 1
fi

if [ ! -x "$BOOTSTRAP_RUNNER" ]; then
  echo "expected Kolme local bootstrap health-check runner to be executable" >&2
  exit 1
fi

# Regression: #1579
if [ ! -x "$SUMMARY_HELPER" ]; then
  echo "expected shared local-lane summary helper to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/generate_local_lane_summary.py" "$MATRIX_RUNNER"; then
  echo "expected local heavy matrix runner to use shared local-lane summary helper" >&2
  exit 1
fi

# Regression: #1585
if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/assert_local_heavy_opt_in.sh" "$MATRIX_RUNNER"; then
  echo "expected local heavy matrix runner to use shared local-heavy opt-in guard helper" >&2
  exit 1
fi

if ! grep -q -- "--mode \$MODE --runtime-profile real-node" "$MATRIX_RUNNER"; then
  echo "expected real-node integration command to track selected matrix mode" >&2
  exit 1
fi

if ! grep -q "REAL_NODE_POLICY_REQUIRED_REASON_CODE" "$MATRIX_RUNNER"; then
  echo "expected matrix runner to declare dynamic real-node policy reason-code marker" >&2
  exit 1
fi

dry_run_output="$(
  bash "$MATRIX_RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run matrix execution to pass"
assert_eq "$(extract_value "$dry_run_output" "matrix_mode")" "dry-run" "expected dry-run matrix mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run matrix reason marker"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only enforcement marker"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.local-heavy-validation-summary.v1":
    raise SystemExit("unexpected local heavy matrix report schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in local heavy matrix summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in local heavy matrix summary")
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code marker in local heavy matrix summary")
commands = report.get("commands")
if not isinstance(commands, list) or len(commands) < 8:
    raise SystemExit("expected local heavy matrix summary to contain command entries")
if not any("run_local_bootstrap_health_checks.sh" in cmd for cmd in commands):
    raise SystemExit("expected bootstrap health-check command marker in local heavy matrix summary")
if not any("run_version_compatibility_replay_deep_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected deep replay command marker in local heavy matrix summary")
if not any("run_local_kolme_fork_rust_test_matrix_contract_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected local fork rust matrix contract-lane command marker in local heavy matrix summary")
if not any("run_local_kolme_live_api_conformance_contract_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected local live API conformance contract-lane command marker in local heavy matrix summary")
if not any("run_local_runtime_commit_live_finality_evidence_contract_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected local runtime commit finality contract-lane command marker in local heavy matrix summary")
if not any(
    "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" in cmd
    and "--max-seconds 120" in cmd
    and "--finality-max-seconds 15" in cmd
    and "--require-native-payload-evidence" in cmd
    for cmd in commands
):
    raise SystemExit("expected runtime commit native parity budget and strict marker flags in local heavy matrix summary")
if not any(
    "run_local_native_api_parity_live_proof_contract_lane.sh" in cmd and "--max-seconds 180" in cmd
    for cmd in commands
):
    raise SystemExit("expected native API parity budget marker in local heavy matrix summary")
if not any(
    "run_local_kamn_live_runtime_integration_lane.sh" in cmd
    and "--runtime-profile real-node" in cmd
    and "--max-seconds 210" in cmd
    and "--runtime-commit-max-seconds 30" in cmd
    and "--runtime-commit-finality-max-seconds 15" in cmd
    for cmd in commands
):
    raise SystemExit("expected real-node runtime integration budget markers in local heavy matrix summary")
if not any(
    "check_local_kamn_live_runtime_real_node_profile_policy.py" in cmd
    and "--require-non-synthetic-run-evidence" in cmd
    for cmd in commands
):
    raise SystemExit("expected strict real-node profile policy check marker in local heavy matrix summary")
PY

# Regression: #1405
if ! printf '%s\n' "$dry_run_output" | grep -q "local_only_enforced=true"; then
  echo "expected local-only matrix guard marker to remain stable" >&2
  exit 1
fi

echo "Kolme local heavy validation matrix runner tests passed."
