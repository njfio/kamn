#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kamn_live_runtime_real_node_profile_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kamn_live_runtime_real_node_profile_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_DRIFT_REPORT="$(mktemp)"
TMP_SIGNER_DRIFT_REPORT="$(mktemp)"
TMP_SYNTHETIC_REPORT="$(mktemp)"
TMP_INMEMORY_REPORT="$(mktemp)"
TMP_FALLBACK_PRESENT_REPORT="$(mktemp)"
TMP_SECONDARY_REPORT="$(mktemp)"
TMP_SECONDARY_POLICY_REPORT="$(mktemp)"
TMP_SECONDARY_KEY_ENV_DRIFT_REPORT="$(mktemp)"
TMP_KEY_SOURCE_MATRIX_DRIFT_REPORT="$(mktemp)"
TMP_MANAGED_EXTERNAL_RAW_KEY_REPORT="$(mktemp)"
TMP_NEGATIVE_POLICY="$(mktemp)"
TMP_NEGATIVE_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT" "$TMP_DRIFT_REPORT" "$TMP_SIGNER_DRIFT_REPORT" "$TMP_SYNTHETIC_REPORT" "$TMP_INMEMORY_REPORT" "$TMP_FALLBACK_PRESENT_REPORT" "$TMP_SECONDARY_REPORT" "$TMP_SECONDARY_POLICY_REPORT" "$TMP_SECONDARY_KEY_ENV_DRIFT_REPORT" "$TMP_KEY_SOURCE_MATRIX_DRIFT_REPORT" "$TMP_MANAGED_EXTERNAL_RAW_KEY_REPORT" "$TMP_NEGATIVE_POLICY" "$TMP_NEGATIVE_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local KAMN live runtime real-node profile contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local KAMN live runtime real-node profile policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local KAMN live runtime real-node profile contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local KAMN live runtime real-node profile contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kamn_live_runtime_real_node_profile_contract_lane.py",
]:
    raise SystemExit("expected local KAMN live runtime real-node profile manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local KAMN live runtime real-node profile contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kamn_live_runtime_integration_lane.sh"
  "check_local_kamn_live_runtime_real_node_profile_policy.py"
  "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"
  "--require-non-synthetic-run-evidence"
  "--runtime-signer-profile"
  "docs/planning/kolme-devnet-ops.md"
  "docs/ci/strategy.md"
  "README.md"
  "runtime_signer_profile=ops-secondary"
  "runtime_signer_failover_active=true"
  "runtime_signer_previous_profile=ops-primary"
  "runtime_signer_rotation_epoch=2"
  "runtime_signing_profile"
  "runtime_signer_key_source_contract_version"
  "runtime_signer_key_source"
  "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1"
  "runtime_signer_attestation_bundle"
  "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF"
  "runtime_signer_fallback_guard_contract_version=v2"
  "runtime_signer_fallback_guard_mode=reject_if_present"
  "runtime_signer_fallback_private_key_present=false"
  "runtime_signer_raw_private_key_present=false"
  "runtime_signer_fallback_private_key_present_violation"
  "runtime_signer_managed_external_raw_private_key_present_violation"
  "runtime_signer_attestation_approved_signers_not_unique"
  "runtime_signer_attestation_quorum_shortfall"
  "runtime_signer_attestation_schema_invalid"
  "runtime_signer_key_source_profile_pair_disallowed"
  "runtime_signer_private_key_env_mismatch"
  "runtime_commit_signer_key_source_marker_missing"
  "runtime_commit_managed_external_signer_key_reference_marker_missing"
  "runtime_commit_managed_external_signer_public_key_marker_missing"
  "runtime_commit_managed_external_private_key_command_marker_detected"
  "runtime_commit_command_profile_mismatch"
  "runtime_commit_signer_profile_split_brain_detected"
  "runtime_signer_failover_profile_unchanged"
  "runtime_signer_rotation_epoch_stale"
  "runtime_commit_signer_profile_marker_missing"
  "runtime_commit_non_synthetic_submit_probe_missing"
  "runtime_commit_in_memory_provider_reference_detected"
  "forced_failover_go_summary"
  "split_brain_negative_summary"
  "Regression: #2302"
  "Regression: #2337"
  "Regression: #2325"
  "Regression: #2327"
  "Regression: #2324"
  "Regression: #2139"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q -- "$marker" "$CONTRACT_IMPL"; then
    echo "expected local KAMN live runtime real-node profile contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q -- "--runtime-profile real-node" "$docs_file"; then
    echo "expected docs parity to include real-node runtime profile marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q -- "--runtime-signer-profile ops-secondary" "$docs_file"; then
    echo "expected docs parity to include secondary signer profile invocation marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_local_kamn_live_runtime_real_node_profile_policy.py" "$docs_file"; then
    echo "expected docs parity to include real-node profile policy checker marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to include real-node profile contract lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_profile=ops-secondary" "$docs_file"; then
    echo "expected docs parity to include secondary signer profile marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_failover_active=true" "$docs_file"; then
    echo "expected docs parity to include forced failover-active marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_previous_profile=ops-primary" "$docs_file"; then
    echo "expected docs parity to include forced failover previous signer marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_rotation_epoch=2" "$docs_file"; then
    echo "expected docs parity to include forced failover rotation epoch marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signing_profile=kolme-fork-secp256k1-v1" "$docs_file"; then
    echo "expected docs parity to include runtime signing profile marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_key_source_contract_version" "$docs_file"; then
    echo "expected docs parity to include signer key-source contract version marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_key_source" "$docs_file"; then
    echo "expected docs parity to include signer key-source marker in $docs_file" >&2
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
  if ! grep -q "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF" "$docs_file"; then
    echo "expected docs parity to include signer key reference env marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_fallback_guard_contract_version=v2" "$docs_file"; then
    echo "expected docs parity to include fallback signer guard contract version marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_fallback_guard_mode=reject_if_present" "$docs_file"; then
    echo "expected docs parity to include fallback signer guard mode marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_fallback_private_key_present=false" "$docs_file"; then
    echo "expected docs parity to include fallback signer private key presence marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_raw_private_key_present=false" "$docs_file"; then
    echo "expected docs parity to include runtime signer raw private key presence marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_fallback_private_key_present_violation" "$docs_file"; then
    echo "expected docs parity to include fallback signer private key violation marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_managed_external_raw_private_key_present_violation" "$docs_file"; then
    echo "expected docs parity to include managed-external raw signer key violation marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_approved_signers_not_unique" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation duplicate-signer reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_quorum_shortfall" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation quorum-shortfall reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_commit_signer_profile_split_brain_detected" "$docs_file"; then
    echo "expected docs parity to include split-brain signer profile reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_attestation_schema_invalid" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation schema invalid reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_key_source_profile_pair_disallowed" "$docs_file"; then
    echo "expected docs parity to include signer key-source/profile pair disallowed reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signer_private_key_env_mismatch" "$docs_file"; then
    echo "expected docs parity to include signer private key mismatch reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_commit_signer_key_source_marker_missing" "$docs_file"; then
    echo "expected docs parity to include signer key-source command marker missing reason in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_commit_managed_external_signer_key_reference_marker_missing" "$docs_file"; then
    echo "expected docs parity to include managed-external key-reference command marker missing reason in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_commit_managed_external_signer_public_key_marker_missing" "$docs_file"; then
    echo "expected docs parity to include managed-external public-key command marker missing reason in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_commit_managed_external_private_key_command_marker_detected" "$docs_file"; then
    echo "expected docs parity to include managed-external private key command marker detected reason in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signing_profile_mismatch" "$docs_file"; then
    echo "expected docs parity to include runtime signing profile mismatch reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_signing_profile_contract_mismatch" "$docs_file"; then
    echo "expected docs parity to include runtime signing profile contract mismatch reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider" "$docs_file"; then
    echo "expected docs parity to include runtime provider contract checkpoint marker in $docs_file" >&2
    exit 1
  fi
  # Regression: #2337
  if ! grep -q "runtime_commit_in_memory_provider_reference_detected" "$docs_file"; then
    echo "expected docs parity to include in-memory provider rollback trigger marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "runtime_commit_policy_check_in_memory_provider_reference_detected" "$docs_file"; then
    echo "expected docs parity to include policy in-memory provider rollback trigger marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2302" "$docs_file"; then
    echo "expected docs parity to include fallback signer runtime regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2337" "$docs_file"; then
    echo "expected docs parity to include failover scenario matrix regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2325" "$docs_file"; then
    echo "expected docs parity to include runtime signer attestation regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2327" "$docs_file"; then
    echo "expected docs parity to include attestation replay-tamper-stale regression matrix marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2324" "$docs_file"; then
    echo "expected docs parity to include managed-external raw signer key regression marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2139" "$docs_file"; then
    echo "expected docs parity to include real-node profile regression marker in $docs_file" >&2
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
if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("unexpected real-node profile contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected real-node profile contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code in real-node profile contract-lane summary")
if summary.get("runtime_profile") != "real-node":
    raise SystemExit("expected runtime_profile=real-node in real-node profile contract-lane summary")
runtime_commit_command = summary.get("runtime_commit_command")
if not isinstance(runtime_commit_command, str):
    raise SystemExit("expected runtime_commit_command in real-node profile contract-lane summary")
if "--require-non-synthetic-run-evidence" not in runtime_commit_command:
    raise SystemExit("expected strict non-synthetic runtime marker in real-node profile contract-lane summary")
if "integration_kolme_fork_live_node_submit_reaches_endpoint" not in runtime_commit_command:
    raise SystemExit("expected non-synthetic runtime submit probe marker in real-node profile contract-lane summary")
if "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1" not in runtime_commit_command:
    raise SystemExit("expected real signing profile marker in real-node profile contract-lane summary")
if "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary" not in runtime_commit_command:
    raise SystemExit("expected signer profile marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected signer profile selector env marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_profile") != "ops-primary":
    raise SystemExit("expected signer profile marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_previous_profile") != "ops-primary":
    raise SystemExit("expected signer previous profile marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_failover_active") is not False:
    raise SystemExit("expected signer failover marker false in real-node profile contract-lane summary")
if summary.get("runtime_signer_rotation_epoch") != 1:
    raise SystemExit("expected signer rotation epoch marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_previous_rotation_epoch") != 1:
    raise SystemExit("expected signer previous rotation epoch marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_key_source_contract_version") != "v1":
    raise SystemExit("expected signer key-source contract version marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_key_source") != "env-local":
    raise SystemExit("expected signer key-source marker in real-node profile contract-lane summary")
if summary.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected runtime_signing_profile marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected signer private key env marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_key_reference_env") != "KAMN_KOLME_LIVE_SIGNER_KEY_REF":
    raise SystemExit("expected signer key reference env marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_fallback_guard_contract_version") != "v2":
    raise SystemExit("expected fallback signer guard contract version marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_fallback_guard_mode") != "reject_if_present":
    raise SystemExit("expected fallback signer guard mode marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_fallback_private_key_present") is not False:
    raise SystemExit("expected fallback signer private key presence marker false in real-node profile contract-lane summary")
if summary.get("runtime_signer_raw_private_key_present") is not False:
    raise SystemExit("expected runtime signer raw private key presence marker false in real-node profile contract-lane summary")
if summary.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected runtime signer attestation schema marker in real-node profile contract-lane summary")
attestation_bundle = summary.get("runtime_signer_attestation_bundle")
if not isinstance(attestation_bundle, dict):
    raise SystemExit("expected runtime signer attestation bundle in real-node profile contract-lane summary")
if attestation_bundle.get("schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected runtime signer attestation bundle schema marker in real-node profile contract-lane summary")
if attestation_bundle.get("required_approvals") != 1:
    raise SystemExit("expected runtime signer attestation required approvals marker in real-node profile contract-lane summary")
if attestation_bundle.get("approved_signers") != ["ops-primary"]:
    raise SystemExit("expected runtime signer attestation approved signer marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_quorum_linkage_contract_version") != "v1":
    raise SystemExit("expected runtime signer quorum linkage contract version marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_quorum_required_approvals") != 1:
    raise SystemExit("expected runtime signer quorum required approvals marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_quorum_approved_signers_count") != 1:
    raise SystemExit("expected runtime signer quorum approved signers count marker in real-node profile contract-lane summary")
if summary.get("runtime_signer_quorum_profile_linked") is not True:
    raise SystemExit("expected runtime signer quorum profile-linked marker true in real-node profile contract-lane summary")
if summary.get("runtime_signer_quorum_satisfied") is not True:
    raise SystemExit("expected runtime signer quorum satisfied marker true in real-node profile contract-lane summary")
if summary.get("runtime_signer_quorum_linked") is not True:
    raise SystemExit("expected runtime signer quorum linked marker true in real-node profile contract-lane summary")
checks = summary.get("checks", [])
if not any(
    isinstance(check, dict)
    and check.get("id") == "runtime_signer_fallback_private_key_contract"
    and check.get("status") == "planned"
    for check in checks
):
    raise SystemExit("expected fallback signer private key planned check marker in real-node profile contract-lane summary")
contracts = summary.get("contracts", {})
if contracts.get("runtime_profile") != "real-node":
    raise SystemExit("expected contracts.runtime_profile=real-node in real-node profile contract-lane summary")
if contracts.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
    raise SystemExit("expected contracts signer profile selector env marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_profile") != "ops-primary":
    raise SystemExit("expected contracts signer profile marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_failover_requires_profile_change") is not True:
    raise SystemExit("expected contracts failover profile-change guard marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_rotation_epoch_must_increase_on_failover") is not True:
    raise SystemExit("expected contracts rotation epoch guard marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_key_source_contract_version") != "v1":
    raise SystemExit("expected contracts signer key-source contract version marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_key_source") != "env-local":
    raise SystemExit("expected contracts signer key-source marker in real-node profile contract-lane summary")
if contracts.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected contracts runtime_signing_profile marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
    raise SystemExit("expected contracts signer private key env marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_key_reference_env") != "KAMN_KOLME_LIVE_SIGNER_KEY_REF":
    raise SystemExit("expected contracts signer key reference env marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_fallback_guard_contract_version") != "v2":
    raise SystemExit("expected contracts fallback signer guard contract version marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_fallback_guard_mode") != "reject_if_present":
    raise SystemExit("expected contracts fallback signer guard mode marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_fallback_private_key_allowed") is not False:
    raise SystemExit("expected contracts fallback signer private key allowed=false marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_fallback_private_key_command_marker_allowed") is not False:
    raise SystemExit(
        "expected contracts fallback signer private key command marker allowed=false marker in real-node profile contract-lane summary"
    )
if contracts.get("runtime_signer_managed_external_raw_private_key_allowed") is not False:
    raise SystemExit("expected contracts managed-external raw private key allowed=false marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
    raise SystemExit("expected contracts runtime signer attestation schema marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_attestation_signer_uniqueness_required") is not True:
    raise SystemExit("expected contracts runtime signer attestation signer-uniqueness requirement marker")
if contracts.get("runtime_signer_attestation_threshold_required") is not True:
    raise SystemExit("expected contracts runtime signer attestation threshold requirement marker")
if contracts.get("runtime_signer_quorum_linkage_contract_version") != "v1":
    raise SystemExit("expected contracts runtime signer quorum linkage contract version marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_quorum_required_approvals") != 1:
    raise SystemExit("expected contracts runtime signer quorum required approvals marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_quorum_linked_required") is not True:
    raise SystemExit("expected contracts runtime signer quorum linked-required marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_quorum_threshold_required") is not True:
    raise SystemExit("expected contracts runtime signer quorum threshold-required marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_quorum_profile_membership_required") is not True:
    raise SystemExit("expected contracts runtime signer quorum profile-membership marker in real-node profile contract-lane summary")
if contracts.get("runtime_signer_quorum_linked") is not True:
    raise SystemExit("expected contracts runtime signer quorum linked marker in real-node profile contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1":
    raise SystemExit("unexpected real-node profile contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected real-node profile contract-lane policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no policy reason codes for real-node profile contract-lane dry-run composition")
PY

bash "$RUNNER" \
  --runtime-signer-profile ops-secondary \
  --output-json "$TMP_SECONDARY_REPORT" \
  --policy-output-json "$TMP_SECONDARY_POLICY_REPORT" >/dev/null

python3 - "$TMP_SECONDARY_REPORT" "$TMP_SECONDARY_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
runtime_commit_command = summary.get("runtime_commit_command")
if not isinstance(runtime_commit_command, str):
    raise SystemExit("expected runtime_commit_command in secondary signer contract-lane summary")
if "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary" not in runtime_commit_command:
    raise SystemExit("expected secondary signer profile marker in secondary signer contract-lane summary")
if summary.get("runtime_signer_profile") != "ops-secondary":
    raise SystemExit("expected secondary signer profile marker in contract-lane summary")
if summary.get("runtime_signer_previous_profile") != "ops-secondary":
    raise SystemExit("expected secondary signer previous-profile marker in contract-lane summary")
if summary.get("runtime_signer_key_source_contract_version") != "v1":
    raise SystemExit("expected secondary signer key-source contract version marker in contract-lane summary")
if summary.get("runtime_signer_key_source") != "env-local":
    raise SystemExit("expected secondary signer key-source marker in contract-lane summary")
if summary.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected runtime_signing_profile marker in secondary contract-lane summary")
if summary.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY":
    raise SystemExit("expected secondary signer private key env marker in contract-lane summary")
if summary.get("runtime_signer_key_reference_env") != "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY":
    raise SystemExit("expected secondary signer key reference env marker in contract-lane summary")
if summary.get("runtime_signer_fallback_guard_contract_version") != "v2":
    raise SystemExit("expected secondary signer fallback guard contract version marker in contract-lane summary")
if summary.get("runtime_signer_fallback_guard_mode") != "reject_if_present":
    raise SystemExit("expected secondary signer fallback guard mode marker in contract-lane summary")
if summary.get("runtime_signer_fallback_private_key_present") is not False:
    raise SystemExit("expected secondary signer fallback private key presence marker false in contract-lane summary")
if summary.get("runtime_signer_raw_private_key_present") is not False:
    raise SystemExit("expected secondary signer raw private key presence marker false in contract-lane summary")
if summary.get("runtime_signer_quorum_linkage_contract_version") != "v1":
    raise SystemExit("expected secondary signer quorum linkage contract version marker in contract-lane summary")
if summary.get("runtime_signer_quorum_required_approvals") != 1:
    raise SystemExit("expected secondary signer quorum required approvals marker in contract-lane summary")
if summary.get("runtime_signer_quorum_approved_signers_count") != 1:
    raise SystemExit("expected secondary signer quorum approved signers count marker in contract-lane summary")
if summary.get("runtime_signer_quorum_profile_linked") is not True:
    raise SystemExit("expected secondary signer quorum profile-linked marker true in contract-lane summary")
if summary.get("runtime_signer_quorum_satisfied") is not True:
    raise SystemExit("expected secondary signer quorum satisfied marker true in contract-lane summary")
if summary.get("runtime_signer_quorum_linked") is not True:
    raise SystemExit("expected secondary signer quorum linked marker true in contract-lane summary")
contracts = summary.get("contracts", {})
if contracts.get("runtime_signer_profile") != "ops-secondary":
    raise SystemExit("expected contracts secondary signer profile marker in contract-lane summary")
if contracts.get("runtime_signer_key_source_contract_version") != "v1":
    raise SystemExit("expected contracts secondary signer key-source contract version marker in contract-lane summary")
if contracts.get("runtime_signer_key_source") != "env-local":
    raise SystemExit("expected contracts secondary signer key-source marker in contract-lane summary")
if contracts.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
    raise SystemExit("expected contracts runtime_signing_profile marker in secondary contract-lane summary")
if contracts.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY":
    raise SystemExit("expected contracts secondary signer private key env marker in contract-lane summary")
if contracts.get("runtime_signer_key_reference_env") != "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY":
    raise SystemExit("expected contracts secondary signer key reference env marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_guard_contract_version") != "v2":
    raise SystemExit("expected contracts secondary signer fallback guard contract version marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_guard_mode") != "reject_if_present":
    raise SystemExit("expected contracts secondary signer fallback guard mode marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_private_key_allowed") is not False:
    raise SystemExit("expected contracts secondary signer fallback private key allowed=false marker in contract-lane summary")
if contracts.get("runtime_signer_fallback_private_key_command_marker_allowed") is not False:
    raise SystemExit(
        "expected contracts secondary signer fallback private key command marker allowed=false marker in contract-lane summary"
    )
if contracts.get("runtime_signer_managed_external_raw_private_key_allowed") is not False:
    raise SystemExit("expected contracts secondary signer managed-external raw private key allowed=false marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_linkage_contract_version") != "v1":
    raise SystemExit("expected contracts secondary signer quorum linkage contract version marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_required_approvals") != 1:
    raise SystemExit("expected contracts secondary signer quorum required approvals marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_linked_required") is not True:
    raise SystemExit("expected contracts secondary signer quorum linked-required marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_threshold_required") is not True:
    raise SystemExit("expected contracts secondary signer quorum threshold-required marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_profile_membership_required") is not True:
    raise SystemExit("expected contracts secondary signer quorum profile-membership marker in contract-lane summary")
if contracts.get("runtime_signer_quorum_linked") is not True:
    raise SystemExit("expected contracts secondary signer quorum linked marker in contract-lane summary")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected secondary signer policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no policy reason codes for secondary signer dry-run composition")
PY

python3 - "$TMP_REPORT" "$TMP_DRIFT_REPORT" "$TMP_SIGNER_DRIFT_REPORT" "$TMP_SYNTHETIC_REPORT" "$TMP_INMEMORY_REPORT" "$TMP_FALLBACK_PRESENT_REPORT" "$TMP_SECONDARY_KEY_ENV_DRIFT_REPORT" "$TMP_KEY_SOURCE_MATRIX_DRIFT_REPORT" "$TMP_MANAGED_EXTERNAL_RAW_KEY_REPORT" <<'PY'
import json
import pathlib
import sys

base_summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))

drift_summary = dict(base_summary)
drift_summary["runtime_commit_command_profile"] = "standard-default-v1"
pathlib.Path(sys.argv[2]).write_text(
    json.dumps(drift_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

signer_drift_summary = dict(base_summary)
signer_drift_summary["runtime_signer_profile"] = "ops-primary"
signer_drift_summary["runtime_signer_previous_profile"] = "ops-primary"
signer_drift_summary["runtime_signer_failover_active"] = True
signer_drift_summary["runtime_signer_rotation_epoch"] = 3
signer_drift_summary["runtime_signer_previous_rotation_epoch"] = 3
pathlib.Path(sys.argv[3]).write_text(
    json.dumps(signer_drift_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

synthetic_summary = dict(base_summary)
synthetic_summary["runtime_commit_command"] = (
    "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh "
    "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider "
    "--require-non-synthetic-run-evidence "
    "--live-command \"printf 'runtime=synthetic\\\\n'\" "
    "--output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json"
)
pathlib.Path(sys.argv[4]).write_text(
    json.dumps(synthetic_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

inmemory_summary = dict(base_summary)
inmemory_summary["runtime_commit_command"] = (
    "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh "
    "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider "
    "--require-non-synthetic-run-evidence "
    "--live-command \"KAMN_KOLME_LIVE_BASE_URL=http://127.0.0.1:3000 "
    "KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport "
    "-- --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\\\n'\" "
    "--provider-hint InMemoryKolmeRuntimeCommitClient "
    "--output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json"
)
pathlib.Path(sys.argv[5]).write_text(
    json.dumps(inmemory_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

fallback_present_summary = dict(base_summary)
fallback_present_summary["mode"] = "run"
fallback_present_summary["status"] = "fail"
fallback_present_summary["reason_code"] = "runtime_signer_fallback_private_key_present_violation"
fallback_present_summary["runtime_signer_fallback_private_key_present"] = True
fallback_present_summary["bootstrap_reason_code"] = "fallback_signer_secret_present_violation"
fallback_present_summary["localhost_signed_reason_code"] = "fallback_signer_secret_present_violation"
fallback_present_summary["conformance_reason_code"] = "fallback_signer_secret_present_violation"
fallback_present_summary["runtime_commit_reason_code"] = "fallback_signer_secret_present_violation"
fallback_present_summary["runtime_commit_policy_reason_code"] = "fallback_signer_secret_present_violation"
pathlib.Path(sys.argv[6]).write_text(
    json.dumps(fallback_present_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

secondary_key_env_drift_summary = dict(base_summary)
secondary_key_env_drift_summary["runtime_signer_profile"] = "ops-secondary"
secondary_key_env_drift_summary["runtime_signer_previous_profile"] = "ops-secondary"
secondary_key_env_drift_summary["runtime_signer_private_key_env"] = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
secondary_key_env_drift_summary["contracts"] = dict(base_summary.get("contracts", {}))
secondary_key_env_drift_summary["contracts"]["runtime_signer_profile"] = "ops-secondary"
secondary_key_env_drift_summary["contracts"]["runtime_signer_private_key_env"] = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
secondary_key_env_drift_summary["runtime_commit_command"] = str(base_summary.get("runtime_commit_command", "")).replace(
    "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary",
    "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary",
)
pathlib.Path(sys.argv[7]).write_text(
    json.dumps(secondary_key_env_drift_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

key_source_matrix_drift_summary = dict(base_summary)
key_source_matrix_drift_summary["runtime_signer_profile"] = "ops-secondary"
key_source_matrix_drift_summary["runtime_signer_previous_profile"] = "ops-secondary"
key_source_matrix_drift_summary["runtime_signer_key_source"] = "managed-external"
key_source_matrix_drift_summary["runtime_signer_private_key_env"] = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
key_source_matrix_drift_summary["contracts"] = dict(base_summary.get("contracts", {}))
key_source_matrix_drift_summary["contracts"]["runtime_signer_profile"] = "ops-secondary"
key_source_matrix_drift_summary["contracts"]["runtime_signer_key_source"] = "managed-external"
key_source_matrix_drift_summary["contracts"]["runtime_signer_private_key_env"] = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"
key_source_matrix_drift_summary["runtime_commit_command"] = str(base_summary.get("runtime_commit_command", "")).replace(
    "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary",
    "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary",
)
pathlib.Path(sys.argv[8]).write_text(
    json.dumps(key_source_matrix_drift_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)

managed_external_raw_key_summary = dict(base_summary)
managed_external_raw_key_summary["mode"] = "run"
managed_external_raw_key_summary["status"] = "fail"
managed_external_raw_key_summary["reason_code"] = "runtime_signer_managed_external_raw_private_key_present_violation"
managed_external_raw_key_summary["runtime_signer_key_source"] = "managed-external"
managed_external_raw_key_summary["runtime_signer_raw_private_key_present"] = True
managed_external_raw_key_summary["bootstrap_reason_code"] = "managed_signer_raw_private_key_present_violation"
managed_external_raw_key_summary["localhost_signed_reason_code"] = "managed_signer_raw_private_key_present_violation"
managed_external_raw_key_summary["conformance_reason_code"] = "managed_signer_raw_private_key_present_violation"
managed_external_raw_key_summary["runtime_commit_reason_code"] = "managed_signer_raw_private_key_present_violation"
managed_external_raw_key_summary["runtime_commit_policy_reason_code"] = "managed_signer_raw_private_key_present_violation"
managed_external_raw_key_contracts = dict(base_summary.get("contracts", {}))
managed_external_raw_key_contracts["runtime_signer_key_source"] = "managed-external"
managed_external_raw_key_summary["contracts"] = managed_external_raw_key_contracts
pathlib.Path(sys.argv[9]).write_text(
    json.dumps(managed_external_raw_key_summary, sort_keys=True, indent=2) + "\n",
    encoding="utf-8",
)
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
drift_exit_code=$?
set -e

if [ "$drift_exit_code" -eq 0 ]; then
  echo "expected marker-drift negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_commit_command_profile_mismatch" "$TMP_NEGATIVE_ERR"; then
  echo "expected runtime command profile drift reason in negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_SIGNER_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
signer_drift_exit_code=$?
set -e

if [ "$signer_drift_exit_code" -eq 0 ]; then
  echo "expected signer profile drift negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_signer_failover_profile_unchanged" "$TMP_NEGATIVE_ERR"; then
  echo "expected signer failover unchanged reason in negative proof output" >&2
  exit 1
fi

if ! grep -q "runtime_signer_rotation_epoch_stale" "$TMP_NEGATIVE_ERR"; then
  echo "expected signer stale rotation epoch reason in negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_SYNTHETIC_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
synthetic_exit_code=$?
set -e

if [ "$synthetic_exit_code" -eq 0 ]; then
  echo "expected synthetic-command negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_commit_non_synthetic_submit_probe_missing" "$TMP_NEGATIVE_ERR"; then
  echo "expected non-synthetic submit probe marker reason in synthetic-command negative proof output" >&2
  exit 1
fi

if ! grep -q "runtime_commit_real_signing_profile_marker_missing" "$TMP_NEGATIVE_ERR"; then
  echo "expected real signing profile marker reason in synthetic-command negative proof output" >&2
  exit 1
fi

if ! grep -q "runtime_commit_signer_profile_marker_missing" "$TMP_NEGATIVE_ERR"; then
  echo "expected signer profile marker reason in synthetic-command negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_INMEMORY_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
inmemory_exit_code=$?
set -e

if [ "$inmemory_exit_code" -eq 0 ]; then
  echo "expected in-memory provider reference negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_commit_in_memory_provider_reference_detected" "$TMP_NEGATIVE_ERR"; then
  echo "expected in-memory provider reference reason in negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_FALLBACK_PRESENT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
fallback_present_exit_code=$?
set -e

if [ "$fallback_present_exit_code" -eq 0 ]; then
  echo "expected fallback signer private key presence negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_signer_fallback_private_key_present_violation" "$TMP_NEGATIVE_ERR"; then
  echo "expected fallback signer private key violation reason in negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_SECONDARY_KEY_ENV_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
secondary_key_env_exit_code=$?
set -e

if [ "$secondary_key_env_exit_code" -eq 0 ]; then
  echo "expected secondary signer key-env drift negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_signer_private_key_env_mismatch" "$TMP_NEGATIVE_ERR"; then
  echo "expected signer private key env mismatch reason in secondary signer negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_KEY_SOURCE_MATRIX_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
key_source_matrix_exit_code=$?
set -e

if [ "$key_source_matrix_exit_code" -eq 0 ]; then
  echo "expected disallowed signer key-source/profile pair negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_signer_key_source_profile_pair_disallowed" "$TMP_NEGATIVE_ERR"; then
  echo "expected signer key-source/profile pair disallowed reason in negative proof output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_MANAGED_EXTERNAL_RAW_KEY_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-non-synthetic-run-evidence \
  --output-json "$TMP_NEGATIVE_POLICY" >"$TMP_NEGATIVE_ERR" 2>&1
managed_external_raw_key_exit_code=$?
set -e

if [ "$managed_external_raw_key_exit_code" -eq 0 ]; then
  echo "expected managed-external raw signer key negative proof to fail closed" >&2
  exit 1
fi

if ! grep -q "runtime_signer_managed_external_raw_private_key_present_violation" "$TMP_NEGATIVE_ERR"; then
  echo "expected managed-external raw signer key violation reason in negative proof output" >&2
  exit 1
fi

echo "local KAMN live runtime real-node profile contract lane tests passed."
