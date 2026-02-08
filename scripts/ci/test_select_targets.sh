#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/select_targets.sh"

extract_output() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { sub($1 "=",""); print; exit }'
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

run_selector() {
  local changed_files="$1"
  env -u GITHUB_OUTPUT -u GITHUB_STEP_SUMMARY \
    CI_CHANGED_FILES="$changed_files" \
    GITHUB_BASE_REF=__missing__ \
    bash "$SCRIPT"
}

docs_output="$(run_selector $'docs/foundation/ci-caching-parallelism.md')"
assert_eq "$(extract_output "$docs_output" "docs_only")" "true" "docs_only selection mismatch"
assert_eq "$(extract_output "$docs_output" "run_rust")" "false" "docs_only should not run rust"
assert_eq "$(extract_output "$docs_output" "run_ci_tool_checks")" "false" "docs_only should not run CI tool checks"
assert_eq "$(extract_output "$docs_output" "run_bridge_replay_harness")" "false" "docs_only should not run bridge replay harness"
assert_eq "$(extract_output "$docs_output" "run_sdk_parity_matrix")" "false" "docs_only should not run sdk parity matrix"
assert_eq "$(extract_output "$docs_output" "test_scope")" "none" "docs_only should keep none scope"

deploy_output="$(run_selector $'scripts/deploy/preflight_topology.sh')"
assert_eq "$(extract_output "$deploy_output" "docs_only")" "false" "deploy-only change must not be docs-only"
assert_eq "$(extract_output "$deploy_output" "run_rust")" "false" "deploy-only changes should avoid rust lane"
assert_eq "$(extract_output "$deploy_output" "run_deploy_preflight_tests")" "true" "deploy-only changes must run deploy preflight tests"
assert_eq "$(extract_output "$deploy_output" "run_bridge_replay_harness")" "false" "deploy-only changes should skip bridge replay harness"
assert_eq "$(extract_output "$deploy_output" "run_sdk_parity_matrix")" "false" "deploy-only changes should skip sdk parity matrix"
assert_eq "$(extract_output "$deploy_output" "test_scope")" "deploy" "deploy-only changes must use deploy scope"

# Regression: #463
runner_output_file="$(mktemp)"
runner_docs_output="$(GITHUB_OUTPUT="$runner_output_file" run_selector $'docs/foundation/ci-caching-parallelism.md')"
rm -f "$runner_output_file"
assert_eq "$(extract_output "$runner_docs_output" "docs_only")" "true" "runner output env must not hide docs_only"

critical_output="$(run_selector $'.github/workflows/ci-fast-gate.yml')"
assert_eq "$(extract_output "$critical_output" "run_rust")" "true" "workflow changes must run rust"
assert_eq "$(extract_output "$critical_output" "run_ci_tool_checks")" "true" "workflow changes must run CI tool checks"
assert_eq "$(extract_output "$critical_output" "test_scope")" "full" "workflow changes must use full scope"

unknown_output="$(run_selector $'config/runtime-policy.json')"
# Regression: #505
assert_eq "$(extract_output "$unknown_output" "run_rust")" "true" "unknown paths must run rust fallback"
assert_eq "$(extract_output "$unknown_output" "run_bridge_replay_harness")" "false" "unknown paths should not trigger bridge replay harness"
assert_eq "$(extract_output "$unknown_output" "run_sdk_parity_matrix")" "false" "unknown paths should not trigger sdk parity matrix"
assert_eq "$(extract_output "$unknown_output" "test_scope")" "full" "unknown paths must use full fallback"

targeted_output="$(run_selector $'crates/kamn-core/src/bridge_adapter.rs')"
assert_eq "$(extract_output "$targeted_output" "run_rust")" "true" "rust path should run rust"
assert_eq "$(extract_output "$targeted_output" "run_ci_tool_checks")" "false" "Regression: #568 non-CI paths should skip CI tool checks"
assert_eq "$(extract_output "$targeted_output" "run_bridge_replay_harness")" "true" "bridge adapter rust paths should run bridge replay harness"
assert_eq "$(extract_output "$targeted_output" "run_sdk_parity_matrix")" "false" "non-sdk rust paths should skip sdk parity matrix"
assert_eq "$(extract_output "$targeted_output" "test_scope")" "targeted" "crate path should be targeted"

test_cmd="$(extract_output "$targeted_output" "test_cmd")"
if ! printf '%s\n' "$test_cmd" | grep -q "run_cargo_test_with_quarantine.sh"; then
  echo "targeted test command must use quarantine wrapper" >&2
  exit 1
fi

python_sdk_output="$(run_selector $'kamn_sdk.py')"
assert_eq "$(extract_output "$python_sdk_output" "run_rust")" "false" "python sdk-only changes should avoid rust lane"
assert_eq "$(extract_output "$python_sdk_output" "run_sdk_parity_matrix")" "true" "python sdk-only changes must run sdk parity matrix"
assert_eq "$(extract_output "$python_sdk_output" "test_scope")" "sdk" "python sdk-only changes should set sdk scope"

typescript_sdk_output="$(run_selector $'packages/kamn-sdk/src/memory_client.ts')"
assert_eq "$(extract_output "$typescript_sdk_output" "run_rust")" "false" "typescript sdk-only changes should avoid rust lane"
assert_eq "$(extract_output "$typescript_sdk_output" "run_sdk_parity_matrix")" "true" "typescript sdk-only changes must run sdk parity matrix"
assert_eq "$(extract_output "$typescript_sdk_output" "test_scope")" "sdk" "typescript sdk-only changes should set sdk scope"

rust_sdk_output="$(run_selector $'crates/kamn-sdk/src/lib.rs')"
assert_eq "$(extract_output "$rust_sdk_output" "run_rust")" "true" "rust sdk changes should run rust lane"
# Regression: #583
assert_eq "$(extract_output "$rust_sdk_output" "run_sdk_parity_matrix")" "true" "rust sdk changes must also run sdk parity matrix"
assert_eq "$(extract_output "$rust_sdk_output" "run_bridge_replay_harness")" "false" "sdk-only paths should skip bridge replay harness"

bridge_script_output="$(run_selector $'scripts/bridge/run_bridge_replay_matrix.sh')"
assert_eq "$(extract_output "$bridge_script_output" "run_rust")" "false" "bridge script-only changes should avoid rust lane"
assert_eq "$(extract_output "$bridge_script_output" "run_bridge_replay_harness")" "true" "bridge script-only changes must run bridge replay harness"
assert_eq "$(extract_output "$bridge_script_output" "test_scope")" "bridge" "bridge script-only changes should set bridge scope"

echo "select_targets matrix regression tests passed."
