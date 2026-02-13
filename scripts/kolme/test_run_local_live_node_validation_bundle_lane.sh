#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_live_node_validation_bundle_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_SUMMARY="$(mktemp)"
TMP_ERR="$(mktemp)"
TMP_INTEGRATION_REPORT="$(mktemp)"
TMP_INTEGRATION_POLICY="$(mktemp)"
TMP_INTEGRATION_RUNTIME_POLICY="$(mktemp)"
TMP_INTEGRATION_RUNTIME_LIVE_SUMMARY="$(mktemp)"
TMP_PROCESS_REPORT="$(mktemp)"
TMP_PROCESS_POLICY="$(mktemp)"
TMP_ROLLBACK_EVIDENCE="$(mktemp)"
TMP_RECOVERY_EVIDENCE="$(mktemp)"
trap 'rm -f "$TMP_SUMMARY" "$TMP_ERR" "$TMP_INTEGRATION_REPORT" "$TMP_INTEGRATION_POLICY" "$TMP_INTEGRATION_RUNTIME_POLICY" "$TMP_INTEGRATION_RUNTIME_LIVE_SUMMARY" "$TMP_PROCESS_REPORT" "$TMP_PROCESS_POLICY" "$TMP_ROLLBACK_EVIDENCE" "$TMP_RECOVERY_EVIDENCE"' EXIT

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
  echo "expected local live-node validation bundle runner to be executable" >&2
  exit 1
fi

# Regression: #2132
if ! grep -q "scripts/framework/assert_local_heavy_opt_in.sh" "$RUNNER"; then
  echo "expected local live-node validation bundle runner to use shared local-heavy opt-in guard helper" >&2
  exit 1
fi

if ! grep -q "run_local_kamn_live_runtime_integration_lane.sh" "$RUNNER"; then
  echo "expected bundle runner to compose local KAMN live runtime integration lane command" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_process_lifecycle_lane.sh" "$RUNNER"; then
  echo "expected bundle runner to compose local fork process lifecycle lane command" >&2
  exit 1
fi

if ! grep -q -- "--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider" "$RUNNER"; then
  echo "expected bundle runner integration command to include explicit runtime provider contract marker" >&2
  exit 1
fi

if ! grep -q -- "--runtime-profile real-node" "$RUNNER"; then
  echo "expected bundle runner integration command to pin real-node runtime profile marker" >&2
  exit 1
fi

if ! grep -q "run_local_live_node_validation_bundle_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local live-node validation bundle runner" >&2
  exit 1
fi

if ! grep -q "run_local_live_node_validation_bundle_lane.sh" "$README_FILE"; then
  echo "expected README to reference local live-node validation bundle runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --integration-report "$TMP_INTEGRATION_REPORT" \
    --integration-policy-report "$TMP_INTEGRATION_POLICY" \
    --integration-runtime-policy-report "$TMP_INTEGRATION_RUNTIME_POLICY" \
    --integration-runtime-commit-live-summary "$TMP_INTEGRATION_RUNTIME_LIVE_SUMMARY" \
    --process-lifecycle-report "$TMP_PROCESS_REPORT" \
    --process-lifecycle-policy-report "$TMP_PROCESS_POLICY" \
    --rollback-evidence-file "$TMP_ROLLBACK_EVIDENCE" \
    --recovery-evidence-file "$TMP_RECOVERY_EVIDENCE" \
    --output-json "$TMP_SUMMARY"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run bundle lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run bundle lane mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run bundle reason marker"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only marker for bundle lane"
assert_eq "$(extract_value "$dry_run_output" "ci_fast_gate_eligible")" "false" "expected local-only fast-gate boundary marker for bundle lane"

python3 - "$TMP_SUMMARY" "$TMP_INTEGRATION_REPORT" "$TMP_INTEGRATION_POLICY" "$TMP_INTEGRATION_RUNTIME_POLICY" "$TMP_INTEGRATION_RUNTIME_LIVE_SUMMARY" "$TMP_PROCESS_REPORT" "$TMP_PROCESS_POLICY" "$TMP_ROLLBACK_EVIDENCE" "$TMP_RECOVERY_EVIDENCE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
expected_artifacts = [str(pathlib.Path(arg).resolve()) for arg in sys.argv[2:]]

if report.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-summary.v1":
    raise SystemExit("unexpected local live-node validation bundle summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in bundle summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in bundle summary")
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code in bundle summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in bundle summary")
if report.get("ci_fast_gate_eligible") is not False:
    raise SystemExit("expected ci_fast_gate_eligible=false in bundle summary")

contracts = report.get("contracts")
if not isinstance(contracts, dict):
    raise SystemExit("expected contracts object in bundle summary")
if contracts.get("ci_fast_gate_scope") != "local-only":
    raise SystemExit("expected ci_fast_gate_scope=local-only in bundle summary contracts")
if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime provider contract marker in bundle summary contracts")

integration_command = report.get("integration_command", "")
if "run_local_kamn_live_runtime_integration_lane.sh" not in integration_command:
    raise SystemExit("expected integration command marker in bundle summary")
if "--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider" not in integration_command:
    raise SystemExit("expected explicit runtime provider marker in integration command")
if "--runtime-profile real-node" not in integration_command:
    raise SystemExit("expected explicit real-node runtime profile marker in integration command")

process_command = report.get("process_lifecycle_command", "")
if "run_local_kolme_fork_process_lifecycle_lane.sh" not in process_command:
    raise SystemExit("expected process lifecycle command marker in bundle summary")
if "--integration-runtime-commit-live-policy-report" not in process_command:
    raise SystemExit("expected integration runtime policy pass-through marker in process lifecycle command")
if f"--rollback-evidence-file {expected_artifacts[6]}" not in process_command:
    raise SystemExit("expected rollback evidence pass-through marker in process lifecycle command")
if f"--recovery-evidence-file {expected_artifacts[7]}" not in process_command:
    raise SystemExit("expected recovery evidence pass-through marker in process lifecycle command")
if report.get("rollback_evidence_file") != expected_artifacts[6]:
    raise SystemExit("expected rollback_evidence_file marker in bundle summary")
if report.get("recovery_evidence_file") != expected_artifacts[7]:
    raise SystemExit("expected recovery_evidence_file marker in bundle summary")
if contracts.get("rollback_recovery_artifact_lineage_required") is not True:
    raise SystemExit("expected rollback/recovery lineage required contract marker in bundle summary contracts")
if contracts.get("process_lifecycle_rollback_evidence_option") != "--rollback-evidence-file":
    raise SystemExit("expected process lifecycle rollback option contract marker in bundle summary contracts")
if contracts.get("process_lifecycle_recovery_evidence_option") != "--recovery-evidence-file":
    raise SystemExit("expected process lifecycle recovery option contract marker in bundle summary contracts")

checks = report.get("checks")
if not isinstance(checks, list) or len(checks) != 4:
    raise SystemExit("expected four deterministic checks in bundle summary")
ids = [check.get("id") for check in checks if isinstance(check, dict)]
if ids != [
    "integration_bundle",
    "integration_policy",
    "process_lifecycle_bundle",
    "process_lifecycle_policy",
]:
    raise SystemExit("unexpected bundle check ordering")
if any(check.get("status") != "planned" for check in checks if isinstance(check, dict)):
    raise SystemExit("expected planned statuses in dry-run bundle checks")

artifact_paths = report.get("artifact_paths")
if not isinstance(artifact_paths, list):
    raise SystemExit("expected artifact paths list in bundle summary")
for expected in expected_artifacts:
    if expected not in artifact_paths:
        raise SystemExit(f"missing expected artifact path in bundle summary: {expected}")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --integration-command "printf 'integration-pass\n'" \
  --integration-policy-command "printf 'integration-policy-pass\n'" \
  --process-lifecycle-command "printf 'process-pass\n'" \
  --process-lifecycle-policy-command "printf 'process-policy-pass\n'" \
  --output-json "$TMP_SUMMARY" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected bundle lane run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic opt-in failure message for bundle lane run mode" >&2
  exit 1
fi

opt_in_run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 bash "$RUNNER" \
    --mode run \
    --max-seconds 30 \
    --integration-max-seconds 10 \
    --process-lifecycle-max-seconds 10 \
    --integration-report "$TMP_INTEGRATION_REPORT" \
    --integration-policy-report "$TMP_INTEGRATION_POLICY" \
    --integration-runtime-policy-report "$TMP_INTEGRATION_RUNTIME_POLICY" \
    --integration-runtime-commit-live-summary "$TMP_INTEGRATION_RUNTIME_LIVE_SUMMARY" \
    --process-lifecycle-report "$TMP_PROCESS_REPORT" \
    --process-lifecycle-policy-report "$TMP_PROCESS_POLICY" \
    --rollback-evidence-file "$TMP_ROLLBACK_EVIDENCE" \
    --recovery-evidence-file "$TMP_RECOVERY_EVIDENCE" \
    --integration-command "python3 -c \"from pathlib import Path; Path('$TMP_INTEGRATION_REPORT').write_text('{\\\"status\\\":\\\"ok\\\"}\\n', encoding='utf-8')\"" \
    --integration-policy-command "python3 -c \"import json; from pathlib import Path; Path('$TMP_INTEGRATION_POLICY').write_text(json.dumps({'final_decision': 'GO'}) + '\\n', encoding='utf-8')\"" \
    --process-lifecycle-command "python3 -c \"from pathlib import Path; Path('$TMP_PROCESS_REPORT').write_text('{\\\"status\\\":\\\"ok\\\"}\\n', encoding='utf-8')\"" \
    --process-lifecycle-policy-command "python3 -c \"import json; from pathlib import Path; Path('$TMP_PROCESS_POLICY').write_text(json.dumps({'final_decision': 'GO'}) + '\\n', encoding='utf-8')\"" \
    --output-json "$TMP_SUMMARY"
)"

assert_eq "$(extract_value "$opt_in_run_output" "status")" "ok" "expected opt-in bundle lane run to pass with deterministic local commands"
assert_eq "$(extract_value "$opt_in_run_output" "lane_mode")" "run" "expected run mode marker for opt-in bundle execution"
assert_eq "$(extract_value "$opt_in_run_output" "reason_code")" "live_node_validation_bundle_passed" "expected success reason marker for opt-in bundle execution"
assert_eq "$(extract_value "$opt_in_run_output" "ci_fast_gate_eligible")" "false" "expected local-only fast-gate marker for opt-in bundle execution"

python3 - "$TMP_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("mode") != "run":
    raise SystemExit("expected run mode marker in opt-in bundle summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in opt-in bundle summary")
if report.get("reason_code") != "live_node_validation_bundle_passed":
    raise SystemExit("expected success reason code in opt-in bundle summary")
if report.get("budget_status") not in ("within_budget",):
    raise SystemExit("expected within_budget status in opt-in bundle summary")
checks = report.get("checks")
if not isinstance(checks, list) or len(checks) != 4:
    raise SystemExit("expected four checks in opt-in bundle summary")
if any(check.get("status") != "pass" for check in checks if isinstance(check, dict)):
    raise SystemExit("expected all checks to pass in opt-in bundle summary")
PY

echo "local live-node validation bundle lane tests passed."
