#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"

required_commands=(
  'bash "$ROOT_DIR/scripts/ci/test_kolme_command_surface_coverage_contract.sh"'
  'bash "$ROOT_DIR/scripts/ci/test_kolme_command_surface_asymmetry_contract.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_generate_fork_compatibility_evidence.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_fork_compatibility_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_runtime_commit_adapter_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_profile_preflight_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_self_test_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_checkout_bootstrap_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_kolme_fork_real_process_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_check_local_e2e_integration_policy.sh"'
  'bash "$ROOT_DIR/scripts/kolme/test_run_local_e2e_integration_contract_lane.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_assert_local_heavy_opt_in.sh"'
  'bash "$ROOT_DIR/scripts/framework/test_generate_local_lane_summary.sh"'
)

for command in "${required_commands[@]}"; do
  if ! grep -Fq "$command" "$CI_TOOLS_SCRIPT"; then
    echo "expected ci tools regression lane to include command: $command" >&2
    exit 1
  fi
done

echo "ci tools command surface contract tests passed."
