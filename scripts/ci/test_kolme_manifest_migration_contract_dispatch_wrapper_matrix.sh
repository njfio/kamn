#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected Kolme manifest-migration dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

wrapper_scripts=(
  "test_kolme_tranche1_manifest_migration_contract.sh"
  "test_kolme_runtime_nonce_manifest_migration_contract.sh"
  "test_kolme_version_matrix_manifest_migration_contract.sh"
  "test_kolme_profile_selftest_portability_manifest_migration_contract.sh"
  "test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh"
  "test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh"
  "test_kolme_parity_demo_real_process_manifest_migration_contract.sh"
)
group_keys=(
  "tranche1"
  "runtime_nonce"
  "version_matrix"
  "profile_selftest_portability"
  "runtime_triadic_bootstrap_e2e"
  "bootstrap_conformance_runtime_process"
  "parity_demo_real_process"
)

for i in "${!wrapper_scripts[@]}"; do
  wrapper_path="$ROOT_DIR/scripts/ci/${wrapper_scripts[$i]}"
  expected_group="${group_keys[$i]}"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected manifest-migration wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if ! grep -q "scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh" "$wrapper_path"; then
    echo "expected wrapper to dispatch through shared manifest-migration dispatcher: ${wrapper_scripts[$i]}" >&2
    exit 1
  fi

  if ! grep -q -- "--group ${expected_group}" "$wrapper_path"; then
    echo "expected wrapper to pass group key ${expected_group}: ${wrapper_scripts[$i]}" >&2
    exit 1
  fi
done

echo "Kolme manifest-migration contract dispatcher wrapper matrix tests passed."
