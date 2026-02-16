#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_managed_signer_startup_live_validation_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_managed_signer_startup_live_validation_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/managed_signer_startup_live_validation_contract_lane.py"
PREFLIGHT_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
PREFLIGHT_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"
RUNBOOK_DOC="$ROOT_DIR/docs/foundation/upgrade-rollback-runbook.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected managed-signer startup live validation contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected managed-signer startup live validation dispatcher to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected managed-signer startup live validation manifest to exist" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected managed-signer startup live validation contract implementation to exist" >&2
  exit 1
fi

if [ ! -x "$PREFLIGHT_RUNNER" ]; then
  echo "expected deployment preflight runner dependency to be executable" >&2
  exit 1
fi

if [ ! -x "$PREFLIGHT_CHECKER" ]; then
  echo "expected deployment preflight policy checker dependency to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected managed-signer startup live validation runner to be a symlink to shared dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_contract_lane_dispatch.sh" ]; then
  echo "expected managed-signer startup live validation runner symlink target to be run_contract_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected managed-signer startup live validation dispatcher to resolve deterministic manifest path" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected managed-signer startup live validation manifest schema")
if payload.get("lane_id") != "kolme.managed_signer_startup_live_validation.contract":
    raise SystemExit("unexpected managed-signer startup live validation manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/managed_signer_startup_live_validation_contract_lane.py",
]:
    raise SystemExit("unexpected managed-signer startup live validation manifest contract command")
PY

required_markers=(
  "run_managed_signer_startup_live_validation_contract_lane.sh"
  "kamn.kolme.managed-signer-startup-live-validation-contract-report.v1"
  "deployment_preflight_passed"
  "signer_rotation_promotion_stalled"
  "quorum_evidence_custody_sha256_mismatch"
  "checkpoint_failed_signer_profile_contract"
  "checkpoint_failed_signer_provenance_contract"
  "checkpoint_failed_signer_rotation_freshness_contract"
  "signer_key_source_production_managed_external_required"
  "signer_profile_mismatch"
  "signer_rotation_epoch_stale"
  "managed_signer_rotation_promotion_stalled_fail_closed_status=verified"
  "managed_signer_custody_audit_parity_fail_closed_status=verified"
  "managed_signer_rotation_reason_taxonomy_status=verified"
  "managed_signer_rehearsal_output_normalization_status=verified"
  "managed_signer_rotation_reason_taxonomy_version=kamn.kolme.managed-signer-startup-reason-taxonomy.v1"
  "managed_signer_rotation_reason_codes_csv=custody_continuity_bypass_detected,quorum_evidence_custody_sha256_mismatch,signer_rotation_epoch_stale,signer_rotation_promotion_stalled,signer_rotation_rehearsal_drift_detected"
  "ci_local_promotion_budget_boundary_status=verified"
  "execution_scope=local-scheduled"
)

for docs_file in "$DOC_FILE" "$CI_COST_DOC" "$ROADMAP_DOC" "$README_FILE"; do
  for marker in "${required_markers[@]}"; do
    if ! grep -q -- "$marker" "$docs_file"; then
      echo "expected docs parity marker '$marker' in $docs_file" >&2
      exit 1
    fi
  done
done

profile_matrix_markers=(
  "signer_key_source_profile_matrix_status=verified"
  "signer_key_source_production_reject_status=verified"
  "signer_key_source_local_override_allow_status=verified"
  "signer_fallback_private_key_reject_status=verified"
  "production_signer_key_source_env_local_forbidden"
  "KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true"
)

for docs_file in "$ROADMAP_DOC" "$RUNBOOK_DOC"; do
  for marker in "${profile_matrix_markers[@]}"; do
    if ! grep -q -- "$marker" "$docs_file"; then
      echo "expected profile matrix docs marker '$marker' in $docs_file" >&2
      exit 1
    fi
  done
done

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
for marker in \
  "status=pass" \
  "final_decision=GO" \
  "managed_signer_profile_status=verified" \
  "managed_signer_missing_key_source_fail_closed_status=verified" \
  "managed_signer_invalid_profile_fail_closed_status=verified" \
  "managed_signer_stale_rotation_fail_closed_status=verified" \
  "managed_signer_rotation_promotion_stalled_fail_closed_status=verified" \
  "managed_signer_custody_audit_parity_fail_closed_status=verified" \
  "managed_signer_reason_code_status=verified" \
  "managed_signer_rotation_reason_taxonomy_status=verified" \
  "managed_signer_rehearsal_output_normalization_status=verified" \
  "managed_signer_rotation_reason_taxonomy_version=kamn.kolme.managed-signer-startup-reason-taxonomy.v1" \
  "managed_signer_rotation_reason_codes_csv=custody_continuity_bypass_detected,quorum_evidence_custody_sha256_mismatch,signer_rotation_epoch_stale,signer_rotation_promotion_stalled,signer_rotation_rehearsal_drift_detected" \
  "signer_key_source_profile_matrix_status=verified" \
  "signer_key_source_production_reject_status=verified" \
  "signer_key_source_local_override_allow_status=verified" \
  "signer_fallback_private_key_reject_status=verified" \
  "signer_key_source_managed_external_allow_status=verified" \
  "execution_scope=local-scheduled" \
  "ci_local_promotion_budget_boundary_status=verified" \
  "performance_budget_status=verified"; do
  if ! printf '%s\n' "$run_output" | grep -q "^${marker}$"; then
    echo "expected managed-signer startup live validation output marker: $marker" >&2
    exit 1
  fi
done

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.managed-signer-startup-live-validation-contract-report.v1":
    raise SystemExit("unexpected managed-signer startup live validation contract report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected managed-signer startup live validation final decision GO")
if payload.get("execution_scope") != "local-scheduled":
    raise SystemExit("expected execution_scope=local-scheduled")
if payload.get("ci_fast_gate_eligible") is not False:
    raise SystemExit("expected ci_fast_gate_eligible=false")
if payload.get("managed_signer_profile_status") != "verified":
    raise SystemExit("expected managed_signer_profile_status=verified")
if payload.get("managed_signer_missing_key_source_fail_closed_status") != "verified":
    raise SystemExit("expected managed_signer_missing_key_source_fail_closed_status=verified")
if payload.get("managed_signer_invalid_profile_fail_closed_status") != "verified":
    raise SystemExit("expected managed_signer_invalid_profile_fail_closed_status=verified")
if payload.get("managed_signer_stale_rotation_fail_closed_status") != "verified":
    raise SystemExit("expected managed_signer_stale_rotation_fail_closed_status=verified")
if payload.get("managed_signer_rotation_promotion_stalled_fail_closed_status") != "verified":
    raise SystemExit("expected managed_signer_rotation_promotion_stalled_fail_closed_status=verified")
if payload.get("managed_signer_custody_audit_parity_fail_closed_status") != "verified":
    raise SystemExit("expected managed_signer_custody_audit_parity_fail_closed_status=verified")
if payload.get("managed_signer_reason_code_status") != "verified":
    raise SystemExit("expected managed_signer_reason_code_status=verified")
if payload.get("managed_signer_rotation_reason_taxonomy_status") != "verified":
    raise SystemExit("expected managed_signer_rotation_reason_taxonomy_status=verified")
if payload.get("managed_signer_rehearsal_output_normalization_status") != "verified":
    raise SystemExit("expected managed_signer_rehearsal_output_normalization_status=verified")
if payload.get("managed_signer_rotation_reason_taxonomy_version") != "kamn.kolme.managed-signer-startup-reason-taxonomy.v1":
    raise SystemExit("expected managed_signer_rotation_reason_taxonomy_version marker")
if payload.get("managed_signer_rotation_reason_codes_csv") != "custody_continuity_bypass_detected,quorum_evidence_custody_sha256_mismatch,signer_rotation_epoch_stale,signer_rotation_promotion_stalled,signer_rotation_rehearsal_drift_detected":
    raise SystemExit("expected deterministic managed_signer_rotation_reason_codes_csv marker")
if payload.get("signer_key_source_profile_matrix_status") != "verified":
    raise SystemExit("expected signer_key_source_profile_matrix_status=verified")
if payload.get("signer_key_source_production_reject_status") != "verified":
    raise SystemExit("expected signer_key_source_production_reject_status=verified")
if payload.get("signer_key_source_local_override_allow_status") != "verified":
    raise SystemExit("expected signer_key_source_local_override_allow_status=verified")
if payload.get("signer_fallback_private_key_reject_status") != "verified":
    raise SystemExit("expected signer_fallback_private_key_reject_status=verified")
if payload.get("signer_key_source_managed_external_allow_status") != "verified":
    raise SystemExit("expected signer_key_source_managed_external_allow_status=verified")
if payload.get("ci_local_promotion_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_promotion_budget_boundary_status=verified")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
observed_reason_codes_csv = payload.get("managed_signer_rotation_observed_reason_codes_csv")
if not isinstance(observed_reason_codes_csv, str):
    raise SystemExit("expected managed_signer_rotation_observed_reason_codes_csv marker")
if observed_reason_codes_csv == "none":
    raise SystemExit("expected managed_signer_rotation_observed_reason_codes_csv to include fail-closed rehearsal reasons")
scenario_reports = payload.get("scenario_reports")
if not isinstance(scenario_reports, list) or len(scenario_reports) != 6:
    raise SystemExit("expected six scenario reports")
expected = {
    "go_baseline": ("GO", "deployment_preflight_passed"),
    "no_go_missing_key_source": ("NO-GO", "checkpoint_failed_signer_provenance_contract"),
    "no_go_invalid_signer_profile": ("NO-GO", "checkpoint_failed_signer_profile_contract"),
    "no_go_stale_rotation_metadata": ("NO-GO", "checkpoint_failed_signer_rotation_freshness_contract"),
    "no_go_rotation_promotion_stalled": ("NO-GO", "deployment_preflight_passed"),
    "no_go_custody_audit_parity_drift": ("NO-GO", "deployment_preflight_passed"),
}
for entry in scenario_reports:
    if not isinstance(entry, dict):
        raise SystemExit("scenario report entry must be an object")
    scenario_id = entry.get("scenario_id")
    if scenario_id not in expected:
        raise SystemExit(f"unexpected scenario id: {scenario_id}")
    final_decision, reason_code = expected[scenario_id]
    if entry.get("final_decision") != final_decision:
        raise SystemExit(f"unexpected final decision for {scenario_id}")
    if entry.get("expected_reason_code") != reason_code:
        raise SystemExit(f"unexpected expected_reason_code for {scenario_id}")
    if entry.get("reason_taxonomy_version") != "kamn.kolme.managed-signer-startup-reason-taxonomy.v1":
        raise SystemExit(f"unexpected reason taxonomy version for {scenario_id}")
    if entry.get("reason_codes_csv") != "custody_continuity_bypass_detected,quorum_evidence_custody_sha256_mismatch,signer_rotation_epoch_stale,signer_rotation_promotion_stalled,signer_rotation_rehearsal_drift_detected":
        raise SystemExit(f"unexpected reason taxonomy codes csv for {scenario_id}")
    observed_reason_codes_csv = entry.get("observed_reason_codes_csv")
    if not isinstance(observed_reason_codes_csv, str):
        raise SystemExit(f"expected observed_reason_codes_csv string for {scenario_id}")
    expected_policy_reason_code = entry.get("expected_policy_reason_code")
    if isinstance(expected_policy_reason_code, str):
        if expected_policy_reason_code not in observed_reason_codes_csv.split(","):
            raise SystemExit(f"expected observed_reason_codes_csv to include expected_policy_reason_code for {scenario_id}")

matrix_reports = payload.get("signer_key_source_matrix_reports")
if not isinstance(matrix_reports, list) or len(matrix_reports) != 4:
    raise SystemExit("expected four signer key-source matrix reports")
expected_matrix = {
    "production_strict_env_local_rejected": ("NO-GO", "production_signer_key_source_env_local_forbidden"),
    "fallback_private_key_env_rejected": ("NO-GO", "fallback_signer_secret_present_violation"),
    "local_override_env_local_allowed": ("GO", "local_override_enabled"),
    "production_strict_managed_external_allowed": ("GO", "managed_external_required"),
}
for entry in matrix_reports:
    if not isinstance(entry, dict):
        raise SystemExit("matrix report entry must be an object")
    scenario_id = entry.get("scenario_id")
    if scenario_id not in expected_matrix:
        raise SystemExit(f"unexpected matrix scenario id: {scenario_id}")
    expected_outcome, expected_reason = expected_matrix[scenario_id]
    if entry.get("expected_policy_outcome") != expected_outcome:
        raise SystemExit(f"unexpected expected policy outcome for {scenario_id}")
    if entry.get("expected_reason_code") != expected_reason:
        raise SystemExit(f"unexpected expected reason code for {scenario_id}")
    if entry.get("status") != "pass":
        raise SystemExit(f"expected matrix scenario status pass for {scenario_id}")
PY

set +e
invalid_budget_output="$(
  bash "$RUNNER" \
    --max-seconds invalid 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected managed-signer startup live validation runner to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q "invalid int value"; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
oversized_budget_output="$(
  bash "$RUNNER" \
    --max-seconds 181 2>&1
)"
oversized_budget_code=$?
set -e
if [ "$oversized_budget_code" -eq 0 ]; then
  echo "expected managed-signer startup live validation runner to fail closed when ci-local promotion budget boundary is exceeded" >&2
  exit 1
fi
if ! printf '%s\n' "$oversized_budget_output" | grep -q "ci-local promotion budget boundary exceeded"; then
  echo "expected deterministic ci-local promotion budget boundary rejection marker" >&2
  exit 1
fi

echo "managed-signer startup live validation contract lane tests passed."
