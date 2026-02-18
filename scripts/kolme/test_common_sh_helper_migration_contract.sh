#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

FALLBACK_REASON_TAXONOMY_VERSION="kamn.shell.common-sh-helper-migration-contract-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV="target_file_missing,missing_common_sh_source,local_extract_value_definition_present,local_assert_eq_definition_present"

TARGET_FILES=(
  "scripts/kolme/test_check_fork_compatibility_policy.sh"
  "scripts/kolme/test_check_local_runtime_commit_live_evidence_policy.sh"
  "scripts/kolme/test_check_nonce_broadcast_parity_policy.sh"
  "scripts/kolme/test_check_runtime_commit_replay_policy.sh"
  "scripts/kolme/test_check_snapshot_drift.sh"
  "scripts/kolme/test_run_local_bootstrap_health_checks.sh"
  "scripts/kolme/test_run_local_e2e_integration_lane.sh"
  "scripts/kolme/test_run_local_fork_smoke_evidence_lane.sh"
  "scripts/kolme/test_run_local_fork_sync_metadata_lane.sh"
  "scripts/kolme/test_run_local_heavy_validation_matrix.sh"
  "scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh"
  "scripts/kolme/test_run_local_kolme_api_probe_lane.sh"
  "scripts/kolme/test_run_local_kolme_api_smoke_lane.sh"
  "scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_lane.sh"
  "scripts/kolme/test_run_local_kolme_fork_portability_preflight_lane.sh"
  "scripts/kolme/test_run_local_kolme_fork_profile_preflight_lane.sh"
  "scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh"
  "scripts/kolme/test_run_local_kolme_fork_self_test_lane.sh"
  "scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh"
  "scripts/kolme/test_run_local_live_node_validation_bundle_lane.sh"
  "scripts/kolme/test_run_local_native_api_parity_live_proof_lane.sh"
  "scripts/kolme/test_run_local_runtime_commit_live_lane.sh"
  "scripts/kolme/test_validate_triadic_devnet_smoke.sh"
  "scripts/kolme/test_validate_version_compatibility.sh"
)

reason_codes=()

matches_pattern() {
  local pattern="$1"
  local target="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q --no-heading "$pattern" "$target"
    return $?
  fi
  grep -Eq "$pattern" "$target"
}

add_reason() {
  local reason="$1"
  for existing in "${reason_codes[@]:-}"; do
    if [[ "$existing" == "$reason" ]]; then
      return 0
    fi
  done
  reason_codes+=("$reason")
}

for rel_path in "${TARGET_FILES[@]}"; do
  target="$ROOT_DIR/$rel_path"
  if [[ ! -f "$target" ]]; then
    add_reason "target_file_missing"
    continue
  fi

  if ! matches_pattern "source .*scripts/lib/common\\.sh" "$target"; then
    add_reason "missing_common_sh_source"
  fi

  if matches_pattern "^extract_value\\(\\)" "$target"; then
    add_reason "local_extract_value_definition_present"
  fi

  if matches_pattern "^assert_eq\\(\\)" "$target"; then
    add_reason "local_assert_eq_definition_present"
  fi
done

if ((${#reason_codes[@]} > 0)); then
  {
    echo "status=fail"
    echo "migration_status=fail"
    echo "reason_taxonomy_version=$FALLBACK_REASON_TAXONOMY_VERSION"
    echo "reason_codes_csv=${reason_codes[*]}"
  } | sed 's/ /,/g'
  exit 1
fi

echo "status=ok"
echo "migration_status=verified"
echo "reason_taxonomy_version=$FALLBACK_REASON_TAXONOMY_VERSION"
echo "reason_codes_csv=none"
