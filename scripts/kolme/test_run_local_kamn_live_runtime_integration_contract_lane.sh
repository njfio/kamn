#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kamn_live_runtime_integration_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json"
RUNTIME_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
RUNTIME_DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
RUNTIME_MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kamn_live_runtime_integration_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_POLICY_ERR="$(mktemp)"
TMP_SIMULATED_SUMMARY="$(mktemp)"
TMP_FALLBACK_SUMMARY="$(mktemp)"
TMP_COMPOSITE_TAMPER_SUMMARY="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_POLICY_ERR" "$TMP_SIMULATED_SUMMARY" "$TMP_FALLBACK_SUMMARY" "$TMP_COMPOSITE_TAMPER_SUMMARY"' EXIT
COMPOSITE_GATE_REASON_TAXONOMY_VERSION="kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1"
COMPOSITE_GATE_REASON_CODES_CSV="dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded"

if [ ! -x "$RUNNER" ]; then
  echo "expected local KAMN live runtime integration contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local KAMN live runtime integration policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local KAMN live runtime integration contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -x "$RUNTIME_DISPATCHER" ]; then
  echo "expected shared runtime lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNTIME_RUNNER" ]; then
  echo "expected local KAMN live runtime integration runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNTIME_RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local KAMN live runtime integration runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$RUNTIME_MANIFEST" ]; then
  echo "expected local KAMN live runtime integration lane manifest to exist" >&2
  exit 1
fi

python3 - "$RUNTIME_MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local KAMN live runtime integration lane manifest schema")
if payload.get("lane_id") != "kolme.local_kamn_live_runtime_integration.run":
    raise SystemExit("expected local KAMN live runtime integration lane manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_kamn_live_runtime_integration_lane_impl.sh",
]:
    raise SystemExit("expected local KAMN live runtime integration lane manifest run command")
PY

resolved_runtime_manifest="$(bash "$RUNTIME_DISPATCHER" --lane-wrapper "$(basename "$RUNTIME_RUNNER")" --resolve-manifest-path)"
if [ "$resolved_runtime_manifest" != "$RUNTIME_MANIFEST" ]; then
  echo "expected runtime integration wrapper to resolve deterministic runtime manifest" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local KAMN live runtime integration contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kamn_live_runtime_integration_contract_lane.py",
]:
    raise SystemExit("expected local KAMN live runtime integration manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local KAMN live runtime integration contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kamn_live_runtime_integration_lane.sh"
  "check_local_kamn_live_runtime_integration_policy.py"
  "run_localhost_signed_integration_contract_lane.sh"
  "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
  "runtime_signer_fallback_private_key_contract"
  "runtime_signer_fallback_private_key_present_violation"
  "runtime_signer_managed_external_raw_private_key_contract"
  "runtime_signer_managed_external_raw_private_key_present_violation"
  "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF"
  "runtime_signer_raw_private_key_present=false"
  "composite_gate_reason_taxonomy_version=${COMPOSITE_GATE_REASON_TAXONOMY_VERSION}"
  "composite_gate_reason_codes_csv=${COMPOSITE_GATE_REASON_CODES_CSV}"
  "composite_gate_evidence_convergence_status=verified"
  "composite_gate_ci_smoke_local_heavy_boundary_status=verified"
  "composite_gate_ci_smoke_lane_cost_profile=low"
  "composite_gate_local_heavy_execution_mode=not_requested"
  "runtime_commit_failure_taxonomy_mismatch:finality.timeout"
  "runtime_profile_run_mode_mismatch"
  "runtime_signer_fallback_guard_contract_version=v2"
  "runtime_signer_fallback_guard_mode=reject_if_present"
  "runtime_signer_fallback_private_key_present=false"
  "Regression: #1489"
  "Regression: #1971"
  "Regression: #2101"
  "Regression: #2112"
  "Regression: #2113"
  "Regression: #2114"
  "Regression: #2302"
  "Regression: #2324"
  "Regression: #2296"
  "Regression: #2298"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local KAMN live runtime integration contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "check_local_kamn_live_runtime_integration_policy.py" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local KAMN live runtime integration policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kamn_live_runtime_integration_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local KAMN live runtime integration contract lane" >&2
  exit 1
fi

if ! grep -q -- "--runtime-commit-finality-command" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document runtime finality pass-through command option" >&2
  exit 1
fi

if ! grep -q -- "--runtime-provider-client-contract" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document runtime provider contract option" >&2
  exit 1
fi

if ! grep -q "ci_fast_gate_eligible" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to document local-only fast-gate eligibility marker" >&2
  exit 1
fi

required_runbook_doc_markers=(
  "Live Provider Operator Runbook (Issue #2114)"
  "Prerequisites (Local)"
  "Execution Flow"
  "Rollback and Recovery Evidence"
  "Troubleshooting"
)
for marker in "${required_runbook_doc_markers[@]}"; do
  if ! grep -q "$marker" "$DOC_FILE"; then
    echo "expected Kolme devnet ops doc to include runbook marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference runtime finality evidence contract lane composition in local KAMN integration lane" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_guard_contract_version=v2" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include fallback signer guard contract version marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_guard_mode=reject_if_present" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include fallback signer guard mode marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_private_key_present=false" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include fallback signer private key presence marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_private_key_present_violation" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include fallback signer private key violation marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_managed_external_raw_private_key_present_violation" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include managed-external raw signer key violation marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include signer key reference env marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_raw_private_key_present=false" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include runtime signer raw private key presence marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_reason_taxonomy_version=${COMPOSITE_GATE_REASON_TAXONOMY_VERSION}" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include composite gate reason taxonomy marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_reason_codes_csv=${COMPOSITE_GATE_REASON_CODES_CSV}" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include composite gate reason codes marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_ci_smoke_local_heavy_boundary_status=verified" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include composite gate ci/local boundary marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_evidence_convergence_status=verified" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include composite gate evidence convergence marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_ci_smoke_lane_cost_profile=low" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include composite gate lane cost marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_local_heavy_execution_mode=not_requested" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include composite gate local-heavy execution mode marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2302" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include fallback signer runtime regression marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2324" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include managed-external raw signer key regression marker" >&2
  exit 1
fi

if ! grep -q "check_local_kamn_live_runtime_integration_policy.py" "$README_FILE"; then
  echo "expected README to reference local KAMN live runtime integration policy checker" >&2
  exit 1
fi

if ! grep -q "run_local_kamn_live_runtime_integration_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local KAMN live runtime integration contract lane" >&2
  exit 1
fi

if ! grep -q -- "--runtime-commit-finality-command" "$README_FILE"; then
  echo "expected README to document runtime finality pass-through command option" >&2
  exit 1
fi

if ! grep -q -- "--runtime-provider-client-contract" "$README_FILE"; then
  echo "expected README to document runtime provider contract option" >&2
  exit 1
fi

if ! grep -q "ci_fast_gate_eligible" "$README_FILE"; then
  echo "expected README to document local-only fast-gate eligibility marker" >&2
  exit 1
fi

if ! grep -q "Live Provider Operator Runbook (Issue #2114)" "$README_FILE"; then
  echo "expected README to reference live provider operator runbook section" >&2
  exit 1
fi

if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference runtime finality evidence contract lane composition in local KAMN integration lane" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_guard_contract_version=v2" "$README_FILE"; then
  echo "expected README to include fallback signer guard contract version marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_guard_mode=reject_if_present" "$README_FILE"; then
  echo "expected README to include fallback signer guard mode marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_private_key_present=false" "$README_FILE"; then
  echo "expected README to include fallback signer private key presence marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_private_key_present_violation" "$README_FILE"; then
  echo "expected README to include fallback signer private key violation marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_managed_external_raw_private_key_present_violation" "$README_FILE"; then
  echo "expected README to include managed-external raw signer key violation marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF" "$README_FILE"; then
  echo "expected README to include signer key reference env marker" >&2
  exit 1
fi

if ! grep -q "runtime_signer_raw_private_key_present=false" "$README_FILE"; then
  echo "expected README to include runtime signer raw private key presence marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_reason_taxonomy_version=${COMPOSITE_GATE_REASON_TAXONOMY_VERSION}" "$README_FILE"; then
  echo "expected README to include composite gate reason taxonomy marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_reason_codes_csv=${COMPOSITE_GATE_REASON_CODES_CSV}" "$README_FILE"; then
  echo "expected README to include composite gate reason codes marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_evidence_convergence_status=verified" "$README_FILE"; then
  echo "expected README to include composite gate evidence convergence marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_ci_smoke_local_heavy_boundary_status=verified" "$README_FILE"; then
  echo "expected README to include composite gate ci/local boundary marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_ci_smoke_lane_cost_profile=low" "$README_FILE"; then
  echo "expected README to include composite gate lane cost marker" >&2
  exit 1
fi

if ! grep -q "composite_gate_local_heavy_execution_mode=not_requested" "$README_FILE"; then
  echo "expected README to include composite gate local-heavy execution mode marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2302" "$README_FILE"; then
  echo "expected README to include fallback signer runtime regression marker" >&2
  exit 1
fi

if ! grep -q "Regression: #2324" "$README_FILE"; then
  echo "expected README to include managed-external raw signer key regression marker" >&2
  exit 1
fi

# Regression: #1489
if ! grep -q "Regression: #1489" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include local KAMN live runtime integration regression marker" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("unexpected local KAMN live runtime integration contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected local KAMN live runtime integration contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry_run_no_commands_executed reason code in contract-lane summary")
if summary.get("runtime_profile") != "real-node":
    raise SystemExit("expected runtime_profile=real-node in contract-lane summary")
if summary.get("composite_gate_reason_taxonomy_version") != "kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1":
    raise SystemExit("expected composite gate reason taxonomy marker in contract-lane summary")
if summary.get("composite_gate_reason_codes_csv") != "dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded":
    raise SystemExit("expected composite gate reason codes marker in contract-lane summary")
if summary.get("composite_gate_evidence_convergence_status") != "verified":
    raise SystemExit("expected composite gate evidence convergence marker in contract-lane summary")
if summary.get("composite_gate_ci_smoke_local_heavy_boundary_status") != "verified":
    raise SystemExit("expected composite gate ci/local boundary marker in contract-lane summary")
if summary.get("composite_gate_ci_smoke_lane_cost_profile") != "low":
    raise SystemExit("expected composite gate ci smoke lane cost marker in contract-lane summary")
if summary.get("composite_gate_local_heavy_execution_mode") != "not_requested":
    raise SystemExit("expected composite gate local-heavy execution mode marker in contract-lane summary")
if summary.get("runtime_signer_fallback_guard_contract_version") != "v2":
    raise SystemExit("expected fallback signer guard contract version marker in contract-lane summary")
if summary.get("runtime_signer_fallback_guard_mode") != "reject_if_present":
    raise SystemExit("expected fallback signer guard mode marker in contract-lane summary")
if summary.get("runtime_signer_fallback_private_key_present") is not False:
    raise SystemExit("expected fallback signer private key presence marker false in contract-lane summary")
if summary.get("runtime_signer_key_reference_env") != "KAMN_KOLME_LIVE_SIGNER_KEY_REF":
    raise SystemExit("expected signer key reference env marker in contract-lane summary")
if summary.get("runtime_signer_raw_private_key_present") is not False:
    raise SystemExit("expected runtime signer raw private key presence marker false in contract-lane summary")
if summary.get("runtime_commit_failure_taxonomy_version") != "v1":
    raise SystemExit("expected runtime commit failure taxonomy version marker in contract-lane summary")
if summary.get("runtime_commit_failure_taxonomy") != "none":
    raise SystemExit("expected runtime commit failure taxonomy none marker in dry-run contract-lane summary")
if summary.get("runtime_commit_nested_reason_code") != "not_run":
    raise SystemExit("expected runtime commit nested reason marker not_run in dry-run contract-lane summary")
diagnostic_hint = summary.get("runtime_commit_failure_diagnostic_hint")
if not isinstance(diagnostic_hint, str) or not diagnostic_hint.strip():
    raise SystemExit("expected runtime commit failure diagnostic hint marker in contract-lane summary")
runtime_policy_report = summary.get("runtime_commit_live_policy_report")
if not isinstance(runtime_policy_report, str) or not runtime_policy_report:
    raise SystemExit("expected runtime commit live policy report marker in contract-lane summary")
if runtime_policy_report not in summary.get("artifact_paths", []):
    raise SystemExit("expected runtime policy report artifact in contract-lane summary artifact paths")
checks = summary.get("checks")
if not isinstance(checks, list) or not any(
    check.get("id") == "runtime_commit_policy" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected runtime commit policy planned check marker in contract-lane summary")
if not any(
    check.get("id") == "runtime_signer_fallback_private_key_contract" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected fallback signer private key planned check marker in contract-lane summary")
if not any(
    check.get("id") == "runtime_signer_managed_external_raw_private_key_contract" and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit(
        "expected managed-external raw signer key planned check marker in contract-lane summary"
    )
if summary.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime provider client contract marker in contract-lane summary")
if summary.get("ci_fast_gate_eligible") is not False:
    raise SystemExit("expected local-only fast-gate exclusion marker in contract-lane summary")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "local-only":
    raise SystemExit("expected local-only fast-gate scope contract marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_guard_contract_version") != "v2":
    raise SystemExit("expected contracts fallback signer guard contract version marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_guard_mode") != "reject_if_present":
    raise SystemExit("expected contracts fallback signer guard mode marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_private_key_allowed") is not False:
    raise SystemExit("expected contracts fallback signer private key allowed=false marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_private_key_command_marker_allowed") is not False:
    raise SystemExit(
        "expected contracts fallback signer private key command marker allowed=false marker in contract-lane summary"
    )
if contracts.get("runtime_signer_key_reference_env") != "KAMN_KOLME_LIVE_SIGNER_KEY_REF":
    raise SystemExit("expected contracts signer key reference env marker in contract-lane summary")
if contracts.get("runtime_signer_managed_external_raw_private_key_allowed") is not False:
    raise SystemExit(
        "expected contracts managed-external raw private key allowed=false marker in contract-lane summary"
    )
if summary.get("runtime_signer_quorum_linkage_contract_version") != "v1":
    raise SystemExit("expected runtime signer quorum linkage contract version marker in contract-lane summary")
if summary.get("runtime_signer_quorum_required_approvals") != 1:
    raise SystemExit("expected runtime signer quorum required approvals marker in contract-lane summary")
if summary.get("runtime_signer_quorum_approved_signers_count") != 1:
    raise SystemExit("expected runtime signer quorum approved signers count marker in contract-lane summary")
if summary.get("runtime_signer_quorum_profile_linked") is not True:
    raise SystemExit("expected runtime signer quorum profile-linked marker true in contract-lane summary")
if summary.get("runtime_signer_quorum_satisfied") is not True:
    raise SystemExit("expected runtime signer quorum satisfied marker true in contract-lane summary")
if summary.get("runtime_signer_quorum_linked") is not True:
    raise SystemExit("expected runtime signer quorum linked marker true in contract-lane summary")
if contracts.get("runtime_signer_quorum_linkage_contract_version") != "v1":
    raise SystemExit("expected contracts runtime signer quorum linkage contract version marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_required_approvals") != 1:
    raise SystemExit("expected contracts runtime signer quorum required approvals marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_linked_required") is not True:
    raise SystemExit("expected contracts runtime signer quorum linked-required marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_threshold_required") is not True:
    raise SystemExit("expected contracts runtime signer quorum threshold-required marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_profile_membership_required") is not True:
    raise SystemExit("expected contracts runtime signer quorum profile-membership marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_linked") is not True:
    raise SystemExit("expected contracts runtime signer quorum linked marker in contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1":
    raise SystemExit("unexpected local KAMN live runtime integration contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected local KAMN live runtime integration contract-lane policy final_decision GO")
if policy.get("composite_gate_reason_taxonomy_version") != "kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1":
    raise SystemExit("expected composite gate reason taxonomy marker in contract-lane policy")
if policy.get("composite_gate_reason_codes_csv") != "dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded":
    raise SystemExit("expected composite gate reason codes marker in contract-lane policy")
if policy.get("composite_gate_evidence_convergence_status") != "verified":
    raise SystemExit("expected composite gate evidence convergence marker in contract-lane policy")
if policy.get("composite_gate_ci_smoke_local_heavy_boundary_status") != "verified":
    raise SystemExit("expected composite gate ci/local boundary marker in contract-lane policy")
if policy.get("composite_gate_ci_smoke_lane_cost_profile") != "low":
    raise SystemExit("expected composite gate ci smoke lane cost marker in contract-lane policy")
if policy.get("composite_gate_local_heavy_execution_mode") != "not_requested":
    raise SystemExit("expected composite gate local-heavy execution mode marker in contract-lane policy")
PY

python3 - "$TMP_REPORT" "$TMP_SIMULATED_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
simulated_path = pathlib.Path(sys.argv[2])
payload = json.loads(summary_path.read_text(encoding="utf-8"))
runtime_command = str(payload.get("runtime_commit_command", ""))
payload["runtime_commit_command"] = runtime_command.replace(
    "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1",
    "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated-v1",
)
simulated_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_SIMULATED_SUMMARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_REPORT" >"$TMP_POLICY_ERR" 2>&1
simulated_profile_policy_code=$?
set -e

if [ "$simulated_profile_policy_code" -eq 0 ]; then
  echo "expected runtime integration policy checker to fail when runtime command uses simulated signing profile marker" >&2
  exit 1
fi

if ! grep -q "runtime_commit_simulated_signing_profile_detected" "$TMP_POLICY_ERR"; then
  echo "expected runtime_commit_simulated_signing_profile_detected reason for runtime integration policy failure" >&2
  exit 1
fi

python3 - "$TMP_REPORT" "$TMP_FALLBACK_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
fallback_path = pathlib.Path(sys.argv[2])
payload = json.loads(summary_path.read_text(encoding="utf-8"))
runtime_command = str(payload.get("runtime_commit_command", ""))
payload["runtime_commit_command"] = (
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK=2222222222222222222222222222222222222222222222222222222222222222 "
    f"{runtime_command}"
)
fallback_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_FALLBACK_SUMMARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_REPORT" >"$TMP_POLICY_ERR" 2>&1
fallback_marker_policy_code=$?
set -e

if [ "$fallback_marker_policy_code" -eq 0 ]; then
  echo "expected runtime integration policy checker to fail when runtime command includes fallback signer private key marker" >&2
  exit 1
fi

if ! grep -q "runtime_commit_fallback_private_key_command_marker_detected" "$TMP_POLICY_ERR"; then
  echo "expected runtime_commit_fallback_private_key_command_marker_detected reason for runtime integration policy failure" >&2
  exit 1
fi

python3 - "$TMP_REPORT" "$TMP_COMPOSITE_TAMPER_SUMMARY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
tampered_path = pathlib.Path(sys.argv[2])
payload = json.loads(summary_path.read_text(encoding="utf-8"))
payload["composite_gate_evidence_convergence_status"] = "drifted"
tampered_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_COMPOSITE_TAMPER_SUMMARY" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY_REPORT" >"$TMP_POLICY_ERR" 2>&1
composite_tamper_policy_code=$?
set -e

if [ "$composite_tamper_policy_code" -eq 0 ]; then
  echo "expected runtime integration policy checker to fail when composite gate evidence convergence marker drifts" >&2
  exit 1
fi

if ! grep -q "composite_gate_evidence_convergence_status_mismatch" "$TMP_POLICY_ERR"; then
  echo "expected composite_gate_evidence_convergence_status_mismatch reason for runtime integration policy failure" >&2
  exit 1
fi

TMP_DIRECT_SUMMARY="$(mktemp)"
TMP_DIRECT_RUNTIME_OUTPUT="$(mktemp)"
TMP_DIRECT_RUNTIME_POLICY="$(mktemp)"
TMP_DIRECT_RUNTIME_FINALITY_OUTPUT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_POLICY_ERR" "$TMP_SIMULATED_SUMMARY" "$TMP_FALLBACK_SUMMARY" "$TMP_COMPOSITE_TAMPER_SUMMARY" "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_RUNTIME_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY" "$TMP_DIRECT_RUNTIME_FINALITY_OUTPUT"' EXIT

bash "$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh" \
  --mode dry-run \
  --runtime-commit-finality-command "printf 'finality=final\n'" \
  --runtime-commit-finality-max-seconds 12 \
  --runtime-commit-finality-output-file "$TMP_DIRECT_RUNTIME_FINALITY_OUTPUT" \
  --runtime-commit-live-policy-report "$TMP_DIRECT_RUNTIME_POLICY" \
  --runtime-commit-output-file "$TMP_DIRECT_RUNTIME_OUTPUT" \
  --runtime-commit-live-summary "$TMP_DIRECT_SUMMARY.runtime.json" \
  --output-json "$TMP_DIRECT_SUMMARY" >/dev/null

python3 - "$TMP_DIRECT_SUMMARY" "$TMP_DIRECT_RUNTIME_FINALITY_OUTPUT" "$TMP_DIRECT_RUNTIME_POLICY" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
runtime_command = summary.get("runtime_commit_command", "")
if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in runtime_command:
    raise SystemExit("expected runtime commit command to compose through runtime finality evidence contract lane")
if "--finality-command" not in runtime_command:
    raise SystemExit("expected runtime commit command to include finality command pass-through")
if "--finality-max-seconds 12" not in runtime_command:
    raise SystemExit("expected runtime commit command to include finality max seconds pass-through")
finality_output_path = pathlib.Path(sys.argv[2]).resolve()
if f"--finality-output-file {finality_output_path}" not in runtime_command:
    raise SystemExit("expected runtime commit command to include finality output pass-through")
policy_output_path = pathlib.Path(sys.argv[3]).resolve()
if f"--policy-output-json {policy_output_path}" not in runtime_command:
    raise SystemExit("expected runtime commit command to include runtime policy report pass-through")
if "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider" not in runtime_command:
    raise SystemExit("expected runtime commit command to include live provider contract pass-through")
if "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1" not in runtime_command:
    raise SystemExit("expected runtime commit command to include real signing profile marker")
if summary.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected integration summary to include runtime_signing_profile marker")
contracts = summary.get("contracts", {})
if not isinstance(contracts, dict):
    raise SystemExit("expected integration summary contracts object")
if contracts.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected integration summary contracts to include runtime_signing_profile marker")
if str(finality_output_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected integration summary artifact paths to include runtime finality output file")
if str(policy_output_path) not in summary.get("artifact_paths", []):
    raise SystemExit("expected integration summary artifact paths to include runtime policy report file")
if summary.get("runtime_commit_live_policy_report") != str(policy_output_path):
    raise SystemExit("expected integration summary to expose runtime commit live policy report path")
PY

echo "local KAMN live runtime integration contract lane tests passed."
