#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_local_heavy_validation_matrix.sh"
BOOTSTRAP_RUNNER="$ROOT_DIR/scripts/kolme/run_local_bootstrap_health_checks.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
SUMMARY_HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_heavy_validation_matrix_lane.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected Kolme local heavy validation matrix runner to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected local heavy validation matrix run-lane manifest to exist" >&2
  exit 1
fi
if [ ! -L "$MATRIX_RUNNER" ]; then
  echo "expected local heavy validation matrix runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi
if [ "$(readlink "$MATRIX_RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local heavy validation matrix runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper run_local_heavy_validation_matrix.sh --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected local heavy validation matrix wrapper to resolve deterministic run-lane manifest path" >&2
  exit 1
fi
if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_heavy_validation_matrix.sh --resolve-manifest-path" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local heavy matrix run-wrapper dispatcher mapping" >&2
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

# Regression: #1585
if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
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
if report.get("scenario_matrix_schema_version") != "kamn.kolme.local-heavy-validation-scenario-matrix.v1":
    raise SystemExit("expected local heavy matrix scenario schema marker")
if report.get("scenario_runtime_mode") != "dry-run":
    raise SystemExit("expected scenario runtime mode marker in local heavy matrix summary")
scenario_profiles = report.get("scenario_runtime_profiles")
if not isinstance(scenario_profiles, list) or scenario_profiles != ["real-node"]:
    raise SystemExit("expected scenario runtime profile matrix marker in local heavy matrix summary")
scenario_ids = report.get("scenario_ids")
if not isinstance(scenario_ids, list) or len(scenario_ids) != 9:
    raise SystemExit("expected deterministic scenario id matrix with 9 entries in local heavy matrix summary")
if report.get("scenario_count") != len(scenario_ids):
    raise SystemExit("expected scenario_count to match scenario_ids length in local heavy matrix summary")
for required_scenario in (
    "bootstrap_health",
    "version_compatibility_replay",
    "fork_rust_matrix",
    "live_api_conformance",
    "signature_parity",
    "runtime_commit_finality",
    "native_api_parity",
    "real_node_runtime_integration",
    "real_node_runtime_policy",
):
    if required_scenario not in scenario_ids:
        raise SystemExit(f"expected scenario id marker {required_scenario} in local heavy matrix summary")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in local heavy matrix summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in local heavy matrix summary")
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code marker in local heavy matrix summary")
commands = report.get("commands")
if not isinstance(commands, list) or len(commands) < 9:
    raise SystemExit("expected local heavy matrix summary to contain command entries")
if not any("run_local_bootstrap_health_checks.sh" in cmd for cmd in commands):
    raise SystemExit("expected bootstrap health-check command marker in local heavy matrix summary")
if not any("run_version_compatibility_replay_deep_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected deep replay command marker in local heavy matrix summary")
if not any("run_local_kolme_fork_rust_test_matrix_contract_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected local fork rust matrix contract-lane command marker in local heavy matrix summary")
if not any("run_local_kolme_live_api_conformance_contract_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected local live API conformance contract-lane command marker in local heavy matrix summary")
# Regression: #2629
if not any("run_signature_parity_contract_lane.sh" in cmd for cmd in commands):
    raise SystemExit("expected signature parity contract-lane command marker in local heavy matrix summary")
if not any(
    "run_signature_parity_contract_lane.sh" in cmd
    and "KAMN_KOLME_SIGNATURE_PARITY_MAX_SECONDS=120" in cmd
    for cmd in commands
):
    raise SystemExit("expected signature parity budget marker in local heavy matrix summary")
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
artifacts = report.get("artifact_paths")
if not isinstance(artifacts, list) or not any(
    "/tmp/kolme-signature-parity-policy-report.json" in artifact for artifact in artifacts
):
    raise SystemExit("expected signature parity policy artifact marker in local heavy matrix summary")
PY

# Regression: #1405
if ! printf '%s\n' "$dry_run_output" | grep -q "local_only_enforced=true"; then
  echo "expected local-only matrix guard marker to remain stable" >&2
  exit 1
fi

echo "Kolme local heavy validation matrix runner tests passed."
