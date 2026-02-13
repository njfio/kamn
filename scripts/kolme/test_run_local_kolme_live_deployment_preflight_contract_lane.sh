#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"
LANE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kolme_live_deployment_preflight_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kolme_live_deployment_preflight_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_NEGATIVE_REPORT="$(mktemp)"
TMP_NEGATIVE_POLICY="$(mktemp)"
TMP_NEGATIVE_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_NEGATIVE_REPORT" "$TMP_NEGATIVE_POLICY" "$TMP_NEGATIVE_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local Kolme live deployment preflight contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local Kolme live deployment preflight policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LANE_RUNNER" ]; then
  echo "expected local Kolme live deployment preflight lane runner to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local Kolme live deployment preflight contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local Kolme live deployment preflight contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kolme_live_deployment_preflight_contract_lane.py",
]:
    raise SystemExit("expected local Kolme live deployment preflight manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local Kolme live deployment preflight contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kolme_live_deployment_preflight_lane.sh"
  "check_local_kolme_live_deployment_preflight_policy.py"
  "run_local_kolme_live_deployment_preflight_contract_lane.sh"
  "runtime_mode_mismatch"
  "checkpoint_failed_signer_secret_contract"
  "checkpoint_failed_signer_quorum_contract"
  "fallback_signer_secret_checkpoint_reason_mismatch"
  "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"
  "contracts.fallback_signer_secret_rejected_profile_class=production"
  "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract"
  "checkpoint_failed_quorum_evidence_contract"
  "checkpoint_failed_custody_evidence_contract"
  "checkpoint_failed_signer_provenance_contract"
  "checkpoint_failed_signer_rotation_freshness_contract"
  "signer_key_source_contract_version"
  "signer_key_source"
  "signer_provenance_file"
  "signer_rotation_epoch_stale"
  "signer_quorum_shortfall"
  "quorum_evidence_missing"
  "quorum_evidence_signer_roles_missing"
  "quorum_evidence_signer_roles_invalid"
  "quorum_evidence_rotation_metadata_missing"
  "quorum_evidence_rotation_metadata_invalid"
  "quorum_evidence_approvals_mismatch"
  "quorum_evidence_custody_sha256_mismatch"
  "quorum_evidence_signer_roles_present"
  "quorum_evidence_signer_roles_valid"
  "quorum_evidence_rotation_metadata_present"
  "quorum_evidence_rotation_metadata_valid"
  "contracts.quorum_evidence_signer_roles_required=true"
  "contracts.quorum_evidence_rotation_metadata_required=true"
  "custody_evidence_missing"
  "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1"
  "runtime_signer_attestation_bundle"
  "runtime_signer_attestation_approved_signers_not_unique"
  "runtime_signer_attestation_quorum_shortfall"
  "runtime_signer_attestation_schema_invalid"
  "runtime_signer_drift_telemetry_schema_version=kamn.kolme.runtime-signer-drift-telemetry.v1"
  "runtime_signer_drift_telemetry"
  "runtime_signer_drift_telemetry_missing"
  "runtime_signer_drift_telemetry_schema_version_mismatch"
  "runtime_signer_drift_telemetry_rotation_delta_invalid"
  "contracts.runtime_signer_drift_telemetry_required=true"
  "Regression: #2226"
  "Regression: #2337"
  "Regression: #2300"
  "Regression: #2301"
  "Regression: #2326"
  "Regression: #2327"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q -- "$marker" "$CONTRACT_IMPL"; then
    echo "expected local Kolme live deployment preflight contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q "run_local_kolme_live_deployment_preflight_lane.sh" "$docs_file"; then
    echo "expected docs parity to include deployment preflight lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_local_kolme_live_deployment_preflight_policy.py" "$docs_file"; then
    echo "expected docs parity to include deployment preflight policy checker marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "run_local_kolme_live_deployment_preflight_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to include deployment preflight contract lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation schema marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_bundle" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation bundle marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_approved_signers_not_unique" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation duplicate-signer reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_quorum_shortfall" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation quorum shortfall reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_drift_telemetry_schema_version=kamn.kolme.runtime-signer-drift-telemetry.v1" "$docs_file"; then
    echo "expected docs parity to include runtime signer drift telemetry schema marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_drift_telemetry" "$docs_file"; then
    echo "expected docs parity to include runtime signer drift telemetry bundle marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_drift_telemetry_missing" "$docs_file"; then
    echo "expected docs parity to include runtime signer drift telemetry missing reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_drift_telemetry_schema_version_mismatch" "$docs_file"; then
    echo "expected docs parity to include runtime signer drift telemetry schema-version mismatch reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_drift_telemetry_rotation_delta_invalid" "$docs_file"; then
    echo "expected docs parity to include runtime signer drift telemetry rotation-delta invalid reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.runtime_signer_drift_telemetry_required=true" "$docs_file"; then
    echo "expected docs parity to include runtime signer drift telemetry contract requirement marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "quorum_evidence_signer_roles_missing" "$docs_file"; then
    echo "expected docs parity to include quorum signer-role metadata missing reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "quorum_evidence_rotation_metadata_missing" "$docs_file"; then
    echo "expected docs parity to include quorum rotation metadata missing reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "quorum_evidence_signer_roles_present" "$docs_file"; then
    echo "expected docs parity to include quorum signer-role metadata presence marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "quorum_evidence_rotation_metadata_present" "$docs_file"; then
    echo "expected docs parity to include quorum rotation metadata presence marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.quorum_evidence_signer_roles_required=true" "$docs_file"; then
    echo "expected docs parity to include quorum signer-role requirement marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.quorum_evidence_rotation_metadata_required=true" "$docs_file"; then
    echo "expected docs parity to include quorum rotation metadata requirement marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "fallback_signer_secret_checkpoint_reason_mismatch" "$docs_file"; then
    echo "expected docs parity to include fallback signer secret checkpoint reason mismatch marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK" "$docs_file"; then
    echo "expected docs parity to include fallback signer secret remediation marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.fallback_signer_secret_rejected_profile_class=production" "$docs_file"; then
    echo "expected docs parity to include fallback signer rejected profile class marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract" "$docs_file"; then
    echo "expected docs parity to include fallback signer checkpoint reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "reason_code=checkpoint_failed_signer_secret_contract" "$docs_file"; then
    echo "expected docs parity to include explicit go/no-go checkpoint reason marker for signer secret in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "reason_code=checkpoint_failed_signer_quorum_contract" "$docs_file"; then
    echo "expected docs parity to include explicit go/no-go checkpoint reason marker for signer quorum in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "reason_code=checkpoint_failed_quorum_evidence_contract" "$docs_file"; then
    echo "expected docs parity to include explicit go/no-go checkpoint reason marker for quorum evidence in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "reason_code=checkpoint_failed_custody_evidence_contract" "$docs_file"; then
    echo "expected docs parity to include explicit go/no-go checkpoint reason marker for custody evidence in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "reason_code=checkpoint_failed_signer_provenance_contract" "$docs_file"; then
    echo "expected docs parity to include explicit go/no-go checkpoint reason marker for signer provenance in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "reason_code=checkpoint_failed_signer_rotation_freshness_contract" "$docs_file"; then
    echo "expected docs parity to include explicit go/no-go checkpoint reason marker for signer rotation freshness in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2226" "$docs_file"; then
    echo "expected docs parity to include deployment preflight contract-lane regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2337" "$docs_file"; then
    echo "expected docs parity to include quorum metadata drift regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2300" "$docs_file"; then
    echo "expected docs parity to include signer provenance/rotation regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2301" "$docs_file"; then
    echo "expected docs parity to include signer quorum/custody regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2326" "$docs_file"; then
    echo "expected docs parity to include runtime/deployment attestation-alignment regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2327" "$docs_file"; then
    echo "expected docs parity to include attestation replay-tamper-stale regression matrix marker in $docs_file" >&2
    exit 1
  fi
done

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-summary.v1":
    raise SystemExit("unexpected deployment preflight contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected deployment preflight contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deployment preflight dry-run reason code in contract-lane summary")
if summary.get("ci_fast_gate_eligible") is not True:
    raise SystemExit("expected deployment preflight contract-lane summary ci_fast_gate_eligible=true")
if summary.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected deployment preflight summary runtime signer attestation schema marker")
attestation_bundle = summary.get("runtime_signer_attestation_bundle")
if not isinstance(attestation_bundle, dict):
    raise SystemExit("expected deployment preflight summary runtime signer attestation bundle")
if attestation_bundle.get("schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected deployment preflight summary attestation bundle schema marker")
if attestation_bundle.get("required_approvals") != summary.get("required_approvals"):
    raise SystemExit("expected deployment preflight summary attestation required approvals to mirror quorum threshold")
if attestation_bundle.get("approved_signers") != ["ops-primary", "ops-secondary"]:
    raise SystemExit("expected deployment preflight summary attestation approved signers marker")
if summary.get("runtime_signer_drift_telemetry_schema_version") != "kamn.kolme.runtime-signer-drift-telemetry.v1":
    raise SystemExit("expected deployment preflight summary runtime signer drift telemetry schema marker")
runtime_signer_drift_telemetry = summary.get("runtime_signer_drift_telemetry")
if not isinstance(runtime_signer_drift_telemetry, dict):
    raise SystemExit("expected deployment preflight summary runtime signer drift telemetry bundle")
if runtime_signer_drift_telemetry.get("schema_version") != "kamn.kolme.runtime-signer-drift-telemetry.v1":
    raise SystemExit("expected deployment preflight summary runtime signer drift telemetry schema marker in bundle")
if runtime_signer_drift_telemetry.get("signer_rotation_delta_epochs") != summary.get("signer_rotation_delta_epochs"):
    raise SystemExit("expected deployment preflight summary runtime signer drift telemetry rotation delta marker")
if runtime_signer_drift_telemetry.get("required_approvals") != summary.get("required_approvals"):
    raise SystemExit("expected deployment preflight summary runtime signer drift telemetry required approvals marker")
if runtime_signer_drift_telemetry.get("received_approvals") != summary.get("received_approvals"):
    raise SystemExit("expected deployment preflight summary runtime signer drift telemetry received approvals marker")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "ci-fast-gate":
    raise SystemExit("expected deployment preflight contracts ci_fast_gate_scope=ci-fast-gate")
if contracts.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected deployment preflight contracts runtime signer attestation schema marker")
if contracts.get("runtime_signer_attestation_signer_uniqueness_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer attestation uniqueness marker")
if contracts.get("runtime_signer_attestation_threshold_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer attestation threshold marker")
if contracts.get("runtime_signer_attestation_profile_membership_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer attestation profile membership marker")
if contracts.get("runtime_signer_drift_telemetry_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer drift telemetry requirement marker")
if contracts.get("runtime_signer_drift_telemetry_schema_version") != "kamn.kolme.runtime-signer-drift-telemetry.v1":
    raise SystemExit("expected deployment preflight contracts runtime signer drift telemetry schema marker")
if contracts.get("runtime_signer_drift_telemetry_rotation_delta_match_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer drift telemetry rotation delta match marker")
if contracts.get("runtime_signer_drift_telemetry_stale_flag_match_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer drift telemetry stale flag match marker")
if contracts.get("runtime_signer_drift_telemetry_quorum_flag_match_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer drift telemetry quorum flag match marker")
if contracts.get("runtime_signer_drift_telemetry_approval_counts_match_required") is not True:
    raise SystemExit("expected deployment preflight contracts runtime signer drift telemetry approval count match marker")
if policy.get("schema_version") != "kamn.kolme.local-live-deployment-preflight-policy-report.v1":
    raise SystemExit("unexpected deployment preflight contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected deployment preflight contract-lane policy final_decision GO")
PY

python3 - "$TMP_REPORT" "$TMP_NEGATIVE_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
summary["runtime_mode"] = "kolme-standard"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_NEGATIVE_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
negative_exit_code=$?
set -e

if [ "$negative_exit_code" -eq 0 ]; then
  echo "expected deployment preflight contract-lane negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_mode_mismatch" "$TMP_NEGATIVE_ERR"; then
  echo "expected runtime mode mismatch reason in deployment preflight contract-lane negative proof output" >&2
  exit 1
fi

echo "local Kolme live deployment preflight contract lane tests passed."
