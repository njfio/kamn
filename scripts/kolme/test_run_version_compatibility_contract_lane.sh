#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_version_compatibility_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_version_compatibility_contract_lane.json"
DEEP_LANE_MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_version_compatibility_replay_deep_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/version_compatibility_contract_lane.py"
MATRIX_CHECKER="$ROOT_DIR/scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py"
VALIDATOR="$ROOT_DIR/scripts/kolme/validate_version_compatibility.py"
FORK_EVIDENCE_GENERATOR="$ROOT_DIR/scripts/kolme/generate_fork_compatibility_evidence.py"
FORK_POLICY_CHECKER="$ROOT_DIR/scripts/kolme/check_fork_compatibility_policy.py"
DEEP_LANE="$ROOT_DIR/scripts/kolme/run_version_compatibility_replay_deep_lane.sh"
DEPLOY_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$TMP_DIR/version-compatibility-replay-deep-report.json"
DEPLOY_DOC_BACKUP="$TMP_DIR/kolme_devnet_ops.md.backup"

restore_deploy_doc() {
  if [ -f "$DEPLOY_DOC_BACKUP" ]; then
    cp "$DEPLOY_DOC_BACKUP" "$DEPLOY_DOC"
  fi
}

cleanup() {
  restore_deploy_doc
  rm -rf "$TMP_DIR"
}

trap cleanup EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected Kolme version compatibility contract lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected Kolme version compatibility deep lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected Kolme run lane dispatcher script to be executable" >&2
  exit 1
fi

if [ ! -L "$DEEP_LANE" ]; then
  echo "expected Kolme version compatibility deep lane script to be a symlink to shared run lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$DEEP_LANE")" != "run_lane_dispatch.sh" ]; then
  echo "expected Kolme version compatibility deep lane script symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected Kolme version compatibility contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected Kolme version compatibility contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/version_compatibility_contract_lane.py",
]:
    raise SystemExit("expected version compatibility manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected Kolme version compatibility contract implementation to exist" >&2
  exit 1
fi

if [ ! -f "$DEPLOY_DOC" ]; then
  echo "expected deploy ops doc to exist" >&2
  exit 1
fi

cp "$DEPLOY_DOC" "$DEPLOY_DOC_BACKUP"

if [ ! -x "$MATRIX_CHECKER" ]; then
  echo "expected upgrade compatibility marker matrix checker to be executable" >&2
  exit 1
fi

if [ ! -f "$DEEP_LANE_MANIFEST" ]; then
  echo "expected Kolme version compatibility replay deep lane manifest to exist" >&2
  exit 1
fi

python3 - "$DEEP_LANE_MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_version_compatibility_replay_deep_lane_impl.sh",
]:
    raise SystemExit("expected version compatibility replay deep lane manifest run command")
PY

resolved_deep_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [ "$resolved_deep_manifest" != "$DEEP_LANE_MANIFEST" ]; then
  echo "expected Kolme version compatibility deep lane wrapper to resolve deterministic manifest path" >&2
  exit 1
fi

required_coverage_markers=(
  "check_upgrade_compatibility_marker_matrix_policy.py"
  "kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
  "kamn.kolme.upgrade-compatibility-runbook-reason-taxonomy.v1"
  "upgrade_compatibility_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
  "upgrade_compatibility_runbook_marker_parity_status=verified"
  "docs/deploy/kolme_devnet_ops.md"
  "expected deploy ops doc to reference compatibility marker matrix checker command"
  "expected deploy ops doc to reference compatibility marker matrix taxonomy marker"
  "expected deploy ops doc to reference upgrade compatibility runbook taxonomy marker"
  "expected deploy ops doc to reference upgrade compatibility runbook reason-codes marker"
  "expected deploy ops doc to reference upgrade compatibility runbook marker-parity status marker"
  "run_runtime_commit_contract_lane.sh"
  "run_runtime_commit_replay_contract_lane.sh"
  "run_nonce_broadcast_parity_contract_lane.sh"
  "run_block_fallback_reconciliation_contract_lane.sh"
  "run_local_runtime_commit_live_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "check_kamn_core_live_https_dependency_posture.sh"
  "dry_run_no_commands_executed"
  "ci-fast-gate and ci-tools fast mode"
  "generate_fork_compatibility_evidence.py"
  "check_fork_compatibility_policy.py"
  "kamn.kolme.fork-compatibility-reason-taxonomy.v1"
  "upgrade_rehearsal_bypass_guard_status"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected Kolme version compatibility contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

tamper_deploy_marker() {
  local marker="$1"
  python3 - "$DEPLOY_DOC" "$marker" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
marker = sys.argv[2]
text = path.read_text(encoding="utf-8")
if marker not in text:
    raise SystemExit(f"missing marker in deploy doc fixture: {marker}")
path.write_text(text.replace(marker, "", 1), encoding="utf-8")
PY
}

contract_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$contract_output" | grep -q "Kolme version compatibility contract lane tests passed."; then
  echo "expected Kolme version compatibility contract lane success marker" >&2
  exit 1
fi

restore_deploy_doc
tamper_deploy_marker "kamn.kolme.upgrade-compatibility-runbook-reason-taxonomy.v1"
set +e
runbook_taxonomy_tampered_output="$(python3 "$CONTRACT_IMPL" 2>&1)"
runbook_taxonomy_tampered_code=$?
set -e
if [ "$runbook_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected deploy runbook taxonomy tamper fixture to fail compatibility contract lane" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_taxonomy_tampered_output" | grep -q "expected deploy ops doc to reference upgrade compatibility runbook taxonomy marker"; then
  echo "expected deterministic runbook taxonomy parity failure marker" >&2
  exit 1
fi

restore_deploy_doc
tamper_deploy_marker "check_upgrade_compatibility_marker_matrix_policy.py"
set +e
runbook_command_tampered_output="$(python3 "$CONTRACT_IMPL" 2>&1)"
runbook_command_tampered_code=$?
set -e
if [ "$runbook_command_tampered_code" -eq 0 ]; then
  echo "expected deploy runbook command tamper fixture to fail compatibility contract lane" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_command_tampered_output" | grep -q "expected deploy ops doc to reference compatibility marker matrix checker command"; then
  echo "expected deterministic runbook command parity failure marker" >&2
  exit 1
fi

restore_deploy_doc
deep_output="$(bash "$DEEP_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$deep_output" | grep -q "Kolme version compatibility replay deep lane tests passed."; then
  echo "expected Kolme version compatibility deep lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.version-compatibility-replay-report.v1":
    raise SystemExit("unexpected deep replay report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected Kolme replay deep report to pass")
PY

matrix_version_report="$TMP_DIR/version-go-report.json"
matrix_fork_report="$TMP_DIR/fork-go-report.json"
matrix_fork_policy_report="$TMP_DIR/fork-policy-go-report.json"

python3 "$VALIDATOR" \
  --kamn-version "1.1.0" \
  --kolme-release-tag "v0.15.2" \
  --ci-fast-gate PASS \
  --output-json "$matrix_version_report" >/dev/null

python3 "$FORK_EVIDENCE_GENERATOR" \
  --upstream-release-tag "v0.15.2" \
  --fork-release-tag "v0.15.2" \
  --fork-repo "njfio/kolme_fork" \
  --fork-ref "refs/heads/main" \
  --ci-fast-gate PASS \
  --output-json "$matrix_fork_report" >/dev/null

python3 "$FORK_POLICY_CHECKER" \
  --report-file "$matrix_fork_report" \
  --expected-upstream-release-tag "v0.15.2" \
  --expected-fork-release-tag "v0.15.2" \
  --expected-fork-repo "njfio/kolme_fork" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$matrix_fork_policy_report" >/dev/null

matrix_go_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_report" \
    --fork-policy-report-file "$matrix_fork_policy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/compatibility-marker-matrix-go-report.json"
)"
if ! printf '%s\n' "$matrix_go_output" | grep -q '^status=ok$'; then
  echo "expected compatibility marker matrix checker to emit status=ok for baseline reports" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_go_output" | grep -q '^final_decision=GO$'; then
  echo "expected compatibility marker matrix checker to emit final_decision=GO for baseline reports" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_go_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected compatibility marker matrix checker reason_codes_value=none for baseline reports" >&2
  exit 1
fi

matrix_version_schema_tampered="$TMP_DIR/version-schema-tampered.json"
python3 - "$matrix_version_report" "$matrix_version_schema_tampered" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["schema_version"] = "kamn.kolme.version-compatibility-report.v0"
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
matrix_schema_tampered_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_schema_tampered" \
    --fork-policy-report-file "$matrix_fork_policy_report" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/compatibility-marker-matrix-schema-tampered-report.json" 2>&1
)"
matrix_schema_tampered_code=$?
set -e
if [ "$matrix_schema_tampered_code" -eq 0 ]; then
  echo "expected tampered version report schema fixture to fail compatibility marker matrix checker" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_schema_tampered_output" | grep -q "version_report_schema_mismatch"; then
  echo "expected deterministic version_report_schema_mismatch reason code" >&2
  exit 1
fi

matrix_version_taxonomy_tampered="$TMP_DIR/version-taxonomy-tampered.json"
python3 - "$matrix_version_report" "$matrix_version_taxonomy_tampered" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["reason_taxonomy_version"] = "kamn.kolme.version-compatibility-reason-taxonomy.v0"
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
matrix_taxonomy_tampered_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_taxonomy_tampered" \
    --fork-policy-report-file "$matrix_fork_policy_report" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/compatibility-marker-matrix-taxonomy-tampered-report.json" 2>&1
)"
matrix_taxonomy_tampered_code=$?
set -e
if [ "$matrix_taxonomy_tampered_code" -eq 0 ]; then
  echo "expected tampered version taxonomy fixture to fail compatibility marker matrix checker" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_taxonomy_tampered_output" | grep -q "version_report_reason_taxonomy_mismatch"; then
  echo "expected deterministic version_report_reason_taxonomy_mismatch reason code" >&2
  exit 1
fi

matrix_fork_csv_tampered="$TMP_DIR/fork-csv-tampered.json"
python3 - "$matrix_fork_policy_report" "$matrix_fork_csv_tampered" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["reason_codes_csv"] = "fork_reason_codes_tampered"
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
matrix_fork_csv_tampered_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_report" \
    --fork-policy-report-file "$matrix_fork_csv_tampered" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/compatibility-marker-matrix-fork-csv-tampered-report.json" 2>&1
)"
matrix_fork_csv_tampered_code=$?
set -e
if [ "$matrix_fork_csv_tampered_code" -eq 0 ]; then
  echo "expected tampered fork policy csv fixture to fail compatibility marker matrix checker" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_fork_csv_tampered_output" | grep -q "fork_policy_report_reason_codes_csv_mismatch"; then
  echo "expected deterministic fork_policy_report_reason_codes_csv_mismatch reason code" >&2
  exit 1
fi

matrix_fork_bypass_tampered="$TMP_DIR/fork-bypass-tampered.json"
python3 - "$matrix_fork_policy_report" "$matrix_fork_bypass_tampered" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["upgrade_rehearsal_bypass_guard_status"] = "tampered"
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
matrix_fork_bypass_tampered_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_report" \
    --fork-policy-report-file "$matrix_fork_bypass_tampered" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/compatibility-marker-matrix-fork-bypass-tampered-report.json" 2>&1
)"
matrix_fork_bypass_tampered_code=$?
set -e
if [ "$matrix_fork_bypass_tampered_code" -eq 0 ]; then
  echo "expected tampered fork bypass guard fixture to fail compatibility marker matrix checker" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_fork_bypass_tampered_output" | grep -q "fork_policy_report_rehearsal_bypass_guard_status_mismatch"; then
  echo "expected deterministic fork_policy_report_rehearsal_bypass_guard_status_mismatch reason code" >&2
  exit 1
fi

set +e
matrix_expected_decision_mismatch_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_report" \
    --fork-policy-report-file "$matrix_fork_policy_report" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/compatibility-marker-matrix-expected-decision-mismatch-report.json" 2>&1
)"
matrix_expected_decision_mismatch_code=$?
set -e
if [ "$matrix_expected_decision_mismatch_code" -eq 0 ]; then
  echo "expected final-decision expectation mismatch to fail compatibility marker matrix checker" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_expected_decision_mismatch_output" | grep -q "expected_final_decision_mismatch"; then
  echo "expected deterministic expected_final_decision_mismatch reason code" >&2
  exit 1
fi

set +e
matrix_ci_fast_gate_fail_output="$(
  python3 "$MATRIX_CHECKER" \
    --version-report-file "$matrix_version_report" \
    --fork-policy-report-file "$matrix_fork_policy_report" \
    --expected-final-decision NO-GO \
    --ci-fast-gate FAIL \
    --output-json "$TMP_DIR/compatibility-marker-matrix-ci-fast-gate-fail-report.json" 2>&1
)"
matrix_ci_fast_gate_fail_code=$?
set -e
if [ "$matrix_ci_fast_gate_fail_code" -eq 0 ]; then
  echo "expected ci-fast-gate FAIL fixture to fail compatibility marker matrix checker" >&2
  exit 1
fi
if ! printf '%s\n' "$matrix_ci_fast_gate_fail_output" | grep -q "ci_fast_gate_failed"; then
  echo "expected deterministic ci_fast_gate_failed reason code" >&2
  exit 1
fi

echo "Kolme version compatibility contract lane script tests passed."
