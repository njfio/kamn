#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_e2e_integration_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_e2e_integration_lane.json"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
SUMMARY_HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
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

if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected local e2e integration runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local e2e integration runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local e2e integration lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local e2e integration lane manifest schema")
if payload.get("lane_id") != "kolme.local_e2e_integration.run":
    raise SystemExit("expected local e2e integration lane manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_e2e_integration_lane_impl.sh",
]:
    raise SystemExit("expected local e2e integration lane manifest run command")
PY

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
assert_eq "$manifest_path" "$MANIFEST" "expected local e2e wrapper to resolve deterministic manifest"
if bash "$DISPATCHER" --lane-wrapper run_missing_local_e2e_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown wrapper" >&2
  exit 1
fi

# Regression: #1579
if [ ! -x "$SUMMARY_HELPER" ]; then
  echo "expected shared local-lane summary helper to be executable" >&2
  exit 1
fi

# Regression: #1585
if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
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
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason marker"
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
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code marker in e2e summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in e2e summary")
checkpoints = report.get("checkpoints")
if not isinstance(checkpoints, list) or len(checkpoints) < 6:
    raise SystemExit("expected deterministic checkpoint entries in e2e summary")
checkpoint_ids = [
    entry.get("id")
    for entry in checkpoints
    if isinstance(entry, dict)
]
if "fork_checkout_bootstrap_contract" not in checkpoint_ids:
    raise SystemExit("expected fork_checkout_bootstrap_contract checkpoint id")
if "runtime_commit_adapter" not in checkpoint_ids:
    raise SystemExit("expected runtime_commit_adapter checkpoint id")
if "sdk_live_transport_parity" not in checkpoint_ids:
    raise SystemExit("expected sdk_live_transport_parity checkpoint id")
if "fork_rust_test_matrix" not in checkpoint_ids:
    raise SystemExit("expected fork_rust_test_matrix checkpoint id")
if "fork_live_api_conformance" not in checkpoint_ids:
    raise SystemExit("expected fork_live_api_conformance checkpoint id")
if checkpoint_ids.index("fork_checkout_bootstrap_contract") > checkpoint_ids.index("runtime_commit_adapter"):
    raise SystemExit("expected checkout bootstrap checkpoint before runtime_commit_adapter")
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
