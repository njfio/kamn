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
assert_eq "$(extract_output "$docs_output" "run_frontend_dashboard_tests")" "false" "docs_only should not run frontend dashboard tests"
assert_eq "$(extract_output "$docs_output" "run_dashboard_contract_tests")" "false" "docs_only should not run dashboard contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_signer_emulator_contract_tests")" "false" "docs_only should not run signer emulator contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_did_registry_contract_tests")" "false" "docs_only should not run did registry contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_runtime_snapshot_contract_tests")" "false" "docs_only should not run runtime snapshot contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_message_lifecycle_contract_tests")" "false" "docs_only should not run message lifecycle contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_channel_lifecycle_contract_tests")" "false" "docs_only should not run channel lifecycle contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_task_operation_snapshot_contract_tests")" "false" "docs_only should not run task operation snapshot contract tests for unrelated docs"
assert_eq "$(extract_output "$docs_output" "run_bridge_replay_harness")" "false" "docs_only should not run bridge replay harness"
assert_eq "$(extract_output "$docs_output" "bridge_replay_suites")" "" "docs_only should not select bridge replay suites"
assert_eq "$(extract_output "$docs_output" "run_rust_live_transport_contract_tests")" "false" "docs_only should not run rust live transport lane"
assert_eq "$(extract_output "$docs_output" "run_python_live_transport_contract_tests")" "false" "docs_only should not run python live transport lane"
assert_eq "$(extract_output "$docs_output" "run_typescript_live_transport_contract_tests")" "false" "docs_only should not run typescript live transport lane"
assert_eq "$(extract_output "$docs_output" "run_live_transport_parity_contract_tests")" "false" "docs_only should not run live transport parity lane"
assert_eq "$(extract_output "$docs_output" "run_live_transport_parity_rust_contract_tests")" "false" "docs_only should not require rust setup for parity lane"
assert_eq "$(extract_output "$docs_output" "live_transport_parity_languages")" "" "docs_only should not select live transport parity languages"
assert_eq "$(extract_output "$docs_output" "run_sdk_parity_matrix")" "false" "docs_only should not run sdk parity matrix"
assert_eq "$(extract_output "$docs_output" "test_scope")" "none" "docs_only should keep none scope"

deploy_output="$(run_selector $'scripts/deploy/preflight_topology.sh')"
assert_eq "$(extract_output "$deploy_output" "docs_only")" "false" "deploy-only change must not be docs-only"
assert_eq "$(extract_output "$deploy_output" "run_rust")" "false" "deploy-only changes should avoid rust lane"
assert_eq "$(extract_output "$deploy_output" "run_deploy_preflight_tests")" "true" "deploy-only changes must run deploy preflight tests"
assert_eq "$(extract_output "$deploy_output" "run_frontend_dashboard_tests")" "false" "deploy-only changes should skip frontend dashboard tests"
assert_eq "$(extract_output "$deploy_output" "run_dashboard_contract_tests")" "false" "deploy-only changes should skip dashboard contract tests"
assert_eq "$(extract_output "$deploy_output" "run_signer_emulator_contract_tests")" "false" "deploy-only changes should skip signer emulator contract tests"
assert_eq "$(extract_output "$deploy_output" "run_did_registry_contract_tests")" "false" "deploy-only changes should skip did registry contract tests"
assert_eq "$(extract_output "$deploy_output" "run_runtime_snapshot_contract_tests")" "false" "deploy-only changes should skip runtime snapshot contract tests"
assert_eq "$(extract_output "$deploy_output" "run_message_lifecycle_contract_tests")" "false" "deploy-only changes should skip message lifecycle contract tests"
assert_eq "$(extract_output "$deploy_output" "run_channel_lifecycle_contract_tests")" "false" "deploy-only changes should skip channel lifecycle contract tests"
assert_eq "$(extract_output "$deploy_output" "run_task_operation_snapshot_contract_tests")" "false" "deploy-only changes should skip task operation snapshot contract tests"
assert_eq "$(extract_output "$deploy_output" "run_bridge_replay_harness")" "false" "deploy-only changes should skip bridge replay harness"
assert_eq "$(extract_output "$deploy_output" "bridge_replay_suites")" "" "deploy-only changes should not select bridge replay suites"
assert_eq "$(extract_output "$deploy_output" "run_rust_live_transport_contract_tests")" "false" "deploy-only changes should skip rust live transport lane"
assert_eq "$(extract_output "$deploy_output" "run_python_live_transport_contract_tests")" "false" "deploy-only changes should skip python live transport lane"
assert_eq "$(extract_output "$deploy_output" "run_typescript_live_transport_contract_tests")" "false" "deploy-only changes should skip typescript live transport lane"
assert_eq "$(extract_output "$deploy_output" "run_live_transport_parity_contract_tests")" "false" "deploy-only changes should skip live transport parity lane"
assert_eq "$(extract_output "$deploy_output" "run_live_transport_parity_rust_contract_tests")" "false" "deploy-only changes should not require rust parity setup"
assert_eq "$(extract_output "$deploy_output" "live_transport_parity_languages")" "" "deploy-only changes should not select parity languages"
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
assert_eq "$(extract_output "$unknown_output" "run_frontend_dashboard_tests")" "false" "unknown paths should not trigger frontend dashboard tests"
assert_eq "$(extract_output "$unknown_output" "run_dashboard_contract_tests")" "false" "unknown paths should not trigger dashboard contract tests"
assert_eq "$(extract_output "$unknown_output" "run_signer_emulator_contract_tests")" "false" "unknown paths should not trigger signer emulator contract tests"
assert_eq "$(extract_output "$unknown_output" "run_did_registry_contract_tests")" "false" "unknown paths should not trigger did registry contract tests"
assert_eq "$(extract_output "$unknown_output" "run_runtime_snapshot_contract_tests")" "false" "unknown paths should not trigger runtime snapshot contract tests"
assert_eq "$(extract_output "$unknown_output" "run_message_lifecycle_contract_tests")" "false" "unknown paths should not trigger message lifecycle contract tests"
assert_eq "$(extract_output "$unknown_output" "run_channel_lifecycle_contract_tests")" "false" "unknown paths should not trigger channel lifecycle contract tests"
assert_eq "$(extract_output "$unknown_output" "run_task_operation_snapshot_contract_tests")" "false" "unknown paths should not trigger task operation snapshot contract tests"
assert_eq "$(extract_output "$unknown_output" "run_bridge_replay_harness")" "false" "unknown paths should not trigger bridge replay harness"
assert_eq "$(extract_output "$unknown_output" "bridge_replay_suites")" "" "unknown paths should not select bridge replay suites"
assert_eq "$(extract_output "$unknown_output" "run_rust_live_transport_contract_tests")" "false" "unknown paths should not trigger rust live transport lane"
assert_eq "$(extract_output "$unknown_output" "run_python_live_transport_contract_tests")" "false" "unknown paths should not trigger python live transport lane"
assert_eq "$(extract_output "$unknown_output" "run_typescript_live_transport_contract_tests")" "false" "unknown paths should not trigger typescript live transport lane"
assert_eq "$(extract_output "$unknown_output" "run_live_transport_parity_contract_tests")" "false" "unknown paths should not trigger live transport parity lane"
assert_eq "$(extract_output "$unknown_output" "run_live_transport_parity_rust_contract_tests")" "false" "unknown paths should not require rust parity setup"
assert_eq "$(extract_output "$unknown_output" "live_transport_parity_languages")" "" "unknown paths should not select parity languages"
assert_eq "$(extract_output "$unknown_output" "run_sdk_parity_matrix")" "false" "unknown paths should not trigger sdk parity matrix"
assert_eq "$(extract_output "$unknown_output" "test_scope")" "full" "unknown paths must use full fallback"

targeted_output="$(run_selector $'crates/kamn-core/src/bridge_adapter.rs')"
assert_eq "$(extract_output "$targeted_output" "run_rust")" "true" "rust path should run rust"
assert_eq "$(extract_output "$targeted_output" "run_ci_tool_checks")" "false" "Regression: #568 non-CI paths should skip CI tool checks"
assert_eq "$(extract_output "$targeted_output" "run_frontend_dashboard_tests")" "false" "bridge adapter rust paths should skip frontend dashboard tests"
assert_eq "$(extract_output "$targeted_output" "run_dashboard_contract_tests")" "false" "bridge adapter paths should skip dashboard contract tests"
assert_eq "$(extract_output "$targeted_output" "run_runtime_snapshot_contract_tests")" "false" "bridge adapter paths should skip runtime snapshot contract tests"
assert_eq "$(extract_output "$targeted_output" "run_message_lifecycle_contract_tests")" "false" "bridge adapter paths should skip message lifecycle contract tests"
assert_eq "$(extract_output "$targeted_output" "run_channel_lifecycle_contract_tests")" "false" "bridge adapter paths should skip channel lifecycle contract tests"
assert_eq "$(extract_output "$targeted_output" "run_task_operation_snapshot_contract_tests")" "false" "bridge adapter paths should skip task operation snapshot contract tests"
assert_eq "$(extract_output "$targeted_output" "run_bridge_replay_harness")" "true" "bridge adapter rust paths should run bridge replay harness"
assert_eq "$(extract_output "$targeted_output" "bridge_replay_suites")" "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" "bridge adapter path should select all bridge suites"
assert_eq "$(extract_output "$targeted_output" "run_sdk_parity_matrix")" "false" "non-sdk rust paths should skip sdk parity matrix"
assert_eq "$(extract_output "$targeted_output" "test_scope")" "targeted" "crate path should be targeted"

test_cmd="$(extract_output "$targeted_output" "test_cmd")"
if ! printf '%s\n' "$test_cmd" | grep -q "run_cargo_test_with_quarantine.sh"; then
  echo "targeted test command must use quarantine wrapper" >&2
  exit 1
fi

python_sdk_output="$(run_selector $'kamn_sdk.py')"
assert_eq "$(extract_output "$python_sdk_output" "run_rust")" "false" "python sdk-only changes should avoid rust lane"
assert_eq "$(extract_output "$python_sdk_output" "run_rust_live_transport_contract_tests")" "false" "python sdk-only changes should skip rust live transport lane"
assert_eq "$(extract_output "$python_sdk_output" "run_python_live_transport_contract_tests")" "true" "python sdk-only changes must run python live transport lane"
assert_eq "$(extract_output "$python_sdk_output" "run_typescript_live_transport_contract_tests")" "false" "python sdk-only changes should skip typescript live transport lane"
assert_eq "$(extract_output "$python_sdk_output" "run_live_transport_parity_contract_tests")" "false" "python sdk-only changes should skip parity lane"
assert_eq "$(extract_output "$python_sdk_output" "run_live_transport_parity_rust_contract_tests")" "false" "python sdk-only changes should not require rust parity setup"
assert_eq "$(extract_output "$python_sdk_output" "live_transport_parity_languages")" "" "python sdk-only changes should not select parity languages"
assert_eq "$(extract_output "$python_sdk_output" "run_sdk_parity_matrix")" "false" "python sdk-only changes should skip sdk parity matrix"
assert_eq "$(extract_output "$python_sdk_output" "test_scope")" "sdk-live-python" "python sdk-only changes should set sdk-live-python scope"

typescript_sdk_output="$(run_selector $'packages/kamn-sdk/src/memory_client.ts')"
assert_eq "$(extract_output "$typescript_sdk_output" "run_rust")" "false" "typescript sdk-only changes should avoid rust lane"
assert_eq "$(extract_output "$typescript_sdk_output" "run_rust_live_transport_contract_tests")" "false" "typescript sdk-only changes should skip rust live transport lane"
assert_eq "$(extract_output "$typescript_sdk_output" "run_python_live_transport_contract_tests")" "false" "typescript sdk-only changes should skip python live transport lane"
assert_eq "$(extract_output "$typescript_sdk_output" "run_typescript_live_transport_contract_tests")" "true" "typescript sdk-only changes must run typescript live transport lane"
assert_eq "$(extract_output "$typescript_sdk_output" "run_live_transport_parity_contract_tests")" "false" "typescript sdk-only changes should skip parity lane"
assert_eq "$(extract_output "$typescript_sdk_output" "run_live_transport_parity_rust_contract_tests")" "false" "typescript sdk-only changes should not require rust parity setup"
assert_eq "$(extract_output "$typescript_sdk_output" "live_transport_parity_languages")" "" "typescript sdk-only changes should not select parity languages"
assert_eq "$(extract_output "$typescript_sdk_output" "run_sdk_parity_matrix")" "false" "typescript sdk-only changes should skip sdk parity matrix"
assert_eq "$(extract_output "$typescript_sdk_output" "test_scope")" "sdk-live-typescript" "typescript sdk-only changes should set sdk-live-typescript scope"

rust_sdk_output="$(run_selector $'crates/kamn-sdk/src/lib.rs')"
assert_eq "$(extract_output "$rust_sdk_output" "run_rust")" "true" "rust sdk changes should run rust lane"
assert_eq "$(extract_output "$rust_sdk_output" "run_rust_live_transport_contract_tests")" "true" "rust sdk changes must run rust live transport lane"
assert_eq "$(extract_output "$rust_sdk_output" "run_python_live_transport_contract_tests")" "false" "rust sdk changes should skip python live transport lane"
assert_eq "$(extract_output "$rust_sdk_output" "run_typescript_live_transport_contract_tests")" "false" "rust sdk changes should skip typescript live transport lane"
assert_eq "$(extract_output "$rust_sdk_output" "run_live_transport_parity_contract_tests")" "false" "rust sdk changes should skip parity lane"
assert_eq "$(extract_output "$rust_sdk_output" "run_live_transport_parity_rust_contract_tests")" "false" "rust sdk changes should not require rust parity setup"
assert_eq "$(extract_output "$rust_sdk_output" "live_transport_parity_languages")" "" "rust sdk changes should not select parity languages"
assert_eq "$(extract_output "$rust_sdk_output" "run_sdk_parity_matrix")" "false" "rust sdk changes should skip sdk parity matrix unless shared fixtures change"
assert_eq "$(extract_output "$rust_sdk_output" "run_frontend_dashboard_tests")" "false" "sdk-only paths should skip frontend dashboard tests"
assert_eq "$(extract_output "$rust_sdk_output" "run_bridge_replay_harness")" "false" "sdk-only paths should skip bridge replay harness"
assert_eq "$(extract_output "$rust_sdk_output" "test_scope")" "targeted" "rust sdk changes should keep targeted rust scope"

multi_lang_sdk_output="$(run_selector $'kamn_sdk.py\npackages/kamn-sdk/src/memory_client.ts')"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_rust")" "false" "multi-language non-rust sdk changes should avoid rust lane"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_rust_live_transport_contract_tests")" "false" "multi-language non-rust sdk changes should skip rust live lane"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_python_live_transport_contract_tests")" "false" "multi-language sdk changes should consolidate into parity lane"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_typescript_live_transport_contract_tests")" "false" "multi-language sdk changes should consolidate into parity lane"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_live_transport_parity_contract_tests")" "true" "multi-language sdk changes must run parity lane"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_live_transport_parity_rust_contract_tests")" "false" "multi-language non-rust sdk changes should avoid rust parity setup"
assert_eq "$(extract_output "$multi_lang_sdk_output" "live_transport_parity_languages")" "python,typescript" "multi-language non-rust sdk changes should run parity subset only"
assert_eq "$(extract_output "$multi_lang_sdk_output" "run_sdk_parity_matrix")" "false" "multi-language sdk changes should avoid expensive parity matrix lane"
assert_eq "$(extract_output "$multi_lang_sdk_output" "test_scope")" "sdk-live-parity" "multi-language sdk changes should set sdk-live-parity scope"

multi_lang_rust_sdk_output="$(run_selector $'crates/kamn-sdk/src/lib.rs\nkamn_sdk.py')"
assert_eq "$(extract_output "$multi_lang_rust_sdk_output" "run_live_transport_parity_contract_tests")" "true" "rust + python sdk changes must run parity lane"
assert_eq "$(extract_output "$multi_lang_rust_sdk_output" "run_live_transport_parity_rust_contract_tests")" "true" "rust + python sdk changes should require rust parity setup"
assert_eq "$(extract_output "$multi_lang_rust_sdk_output" "live_transport_parity_languages")" "rust,python" "rust + python sdk changes should run rust+python parity subset"
assert_eq "$(extract_output "$multi_lang_rust_sdk_output" "test_scope")" "targeted" "rust + python sdk changes should preserve targeted rust scope"

parity_script_output="$(run_selector $'scripts/sdk/run_live_transport_parity_contract_lane.sh')"
assert_eq "$(extract_output "$parity_script_output" "run_rust")" "false" "parity script-only changes should avoid rust lane"
assert_eq "$(extract_output "$parity_script_output" "run_live_transport_parity_contract_tests")" "true" "parity script-only changes must run parity lane"
assert_eq "$(extract_output "$parity_script_output" "run_live_transport_parity_rust_contract_tests")" "true" "parity script-only changes should require rust parity setup"
assert_eq "$(extract_output "$parity_script_output" "live_transport_parity_languages")" "rust,python,typescript" "parity script-only changes should run full parity language set"
assert_eq "$(extract_output "$parity_script_output" "run_sdk_parity_matrix")" "false" "parity script-only changes should skip sdk parity matrix"
assert_eq "$(extract_output "$parity_script_output" "test_scope")" "sdk-live-parity" "parity script-only changes should set sdk-live-parity scope"

shared_matrix_output="$(run_selector $'fixtures/sdk_parity/register_validation_cases.json')"
assert_eq "$(extract_output "$shared_matrix_output" "run_rust")" "false" "shared sdk matrix fixture changes should avoid rust lane"
assert_eq "$(extract_output "$shared_matrix_output" "run_rust_live_transport_contract_tests")" "false" "shared sdk matrix fixture changes should skip rust live lane"
assert_eq "$(extract_output "$shared_matrix_output" "run_python_live_transport_contract_tests")" "false" "shared sdk matrix fixture changes should skip python live lane"
assert_eq "$(extract_output "$shared_matrix_output" "run_typescript_live_transport_contract_tests")" "false" "shared sdk matrix fixture changes should skip typescript live lane"
assert_eq "$(extract_output "$shared_matrix_output" "run_live_transport_parity_contract_tests")" "false" "shared sdk matrix fixture changes should skip parity lane"
assert_eq "$(extract_output "$shared_matrix_output" "run_live_transport_parity_rust_contract_tests")" "false" "shared sdk matrix fixture changes should skip rust parity setup"
assert_eq "$(extract_output "$shared_matrix_output" "live_transport_parity_languages")" "" "shared sdk matrix fixture changes should not select parity languages"
assert_eq "$(extract_output "$shared_matrix_output" "run_sdk_parity_matrix")" "true" "shared sdk matrix fixture changes must run sdk parity matrix"
assert_eq "$(extract_output "$shared_matrix_output" "test_scope")" "sdk" "shared sdk matrix fixture changes should set sdk scope"

bridge_script_output="$(run_selector $'scripts/bridge/run_bridge_replay_matrix.sh')"
assert_eq "$(extract_output "$bridge_script_output" "run_rust")" "false" "bridge script-only changes should avoid rust lane"
assert_eq "$(extract_output "$bridge_script_output" "run_frontend_dashboard_tests")" "false" "bridge script-only changes should skip frontend dashboard tests"
assert_eq "$(extract_output "$bridge_script_output" "run_bridge_replay_harness")" "true" "bridge script-only changes must run bridge replay harness"
assert_eq "$(extract_output "$bridge_script_output" "bridge_replay_suites")" "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" "bridge script-only changes should select all bridge suites"
assert_eq "$(extract_output "$bridge_script_output" "test_scope")" "bridge" "bridge script-only changes should set bridge scope"

telegram_bridge_output="$(run_selector $'crates/kamn-core/src/telegram_bridge.rs')"
assert_eq "$(extract_output "$telegram_bridge_output" "run_bridge_replay_harness")" "true" "telegram bridge changes must run bridge replay harness"
assert_eq "$(extract_output "$telegram_bridge_output" "bridge_replay_suites")" "bridge_adapter,telegram_bridge" "telegram bridge changes should select telegram subset plus bridge adapter suite"

discord_bridge_output="$(run_selector $'crates/kamn-core/src/discord_bridge.rs')"
assert_eq "$(extract_output "$discord_bridge_output" "run_bridge_replay_harness")" "true" "discord bridge changes must run bridge replay harness"
assert_eq "$(extract_output "$discord_bridge_output" "bridge_replay_suites")" "bridge_adapter,discord_bridge" "discord bridge changes should select discord subset plus bridge adapter suite"

cross_chain_bridge_output="$(run_selector $'crates/kamn-core/src/cross_chain_bridge.rs')"
assert_eq "$(extract_output "$cross_chain_bridge_output" "run_bridge_replay_harness")" "true" "cross-chain bridge changes must run bridge replay harness"
assert_eq "$(extract_output "$cross_chain_bridge_output" "bridge_replay_suites")" "bridge_adapter,cross_chain_bridge" "cross-chain bridge changes should select cross-chain subset plus bridge adapter suite"

frontend_output="$(run_selector $'packages/kamn-dashboard/tests/dashboard.test.ts')"
assert_eq "$(extract_output "$frontend_output" "run_rust")" "false" "frontend-only changes should avoid rust lane"
assert_eq "$(extract_output "$frontend_output" "run_frontend_dashboard_tests")" "true" "frontend-only changes must run dashboard tests"
assert_eq "$(extract_output "$frontend_output" "run_dashboard_contract_tests")" "false" "frontend-only changes should skip dashboard contract lane"
assert_eq "$(extract_output "$frontend_output" "run_bridge_replay_harness")" "false" "frontend-only changes should skip bridge harness"
assert_eq "$(extract_output "$frontend_output" "run_sdk_parity_matrix")" "false" "frontend-only changes should skip sdk matrix"
assert_eq "$(extract_output "$frontend_output" "test_scope")" "frontend" "frontend-only changes should set frontend scope"

dashboard_contract_output="$(run_selector $'docs/foundation/operator-dashboard-backend-apis.md')"
assert_eq "$(extract_output "$dashboard_contract_output" "run_rust")" "false" "dashboard contract docs should avoid rust lane"
assert_eq "$(extract_output "$dashboard_contract_output" "run_frontend_dashboard_tests")" "false" "dashboard contract docs should skip frontend dashboard tests"
assert_eq "$(extract_output "$dashboard_contract_output" "run_dashboard_contract_tests")" "true" "dashboard contract docs must run dashboard contract lane"
assert_eq "$(extract_output "$dashboard_contract_output" "test_scope")" "frontend-contract" "dashboard contract docs should set frontend-contract scope"

signer_contract_output="$(run_selector $'docs/foundation/signer-backend-abstraction.md')"
assert_eq "$(extract_output "$signer_contract_output" "run_rust")" "false" "signer contract docs should avoid rust lane"
assert_eq "$(extract_output "$signer_contract_output" "run_signer_emulator_contract_tests")" "true" "signer contract docs must run signer emulator contract lane"
assert_eq "$(extract_output "$signer_contract_output" "test_scope")" "signer-contract" "signer contract docs should set signer-contract scope"

signer_rust_output="$(run_selector $'crates/kamn-core/src/signer_backend.rs')"
assert_eq "$(extract_output "$signer_rust_output" "run_rust")" "true" "signer backend rust changes should run rust lane"
assert_eq "$(extract_output "$signer_rust_output" "run_signer_emulator_contract_tests")" "true" "signer backend rust changes must run signer emulator contract lane"
assert_eq "$(extract_output "$signer_rust_output" "test_scope")" "targeted" "signer backend rust changes should stay targeted"

signer_contract_script_output="$(run_selector $'scripts/signer/run_signer_emulator_contract_lane.sh')"
assert_eq "$(extract_output "$signer_contract_script_output" "run_rust")" "false" "signer contract script-only changes should avoid rust lane"
assert_eq "$(extract_output "$signer_contract_script_output" "run_signer_emulator_contract_tests")" "true" "signer contract script changes must run signer emulator contract lane"
assert_eq "$(extract_output "$signer_contract_script_output" "test_scope")" "signer-contract" "signer contract script changes should set signer-contract scope"

did_contract_docs_output="$(run_selector $'docs/foundation/did-registry-transactions.md')"
assert_eq "$(extract_output "$did_contract_docs_output" "run_rust")" "false" "did contract docs should avoid rust lane"
assert_eq "$(extract_output "$did_contract_docs_output" "run_did_registry_contract_tests")" "true" "did contract docs must run did registry contract lane"
assert_eq "$(extract_output "$did_contract_docs_output" "test_scope")" "did-contract" "did contract docs should set did-contract scope"

did_contract_rust_output="$(run_selector $'crates/kamn-core/src/did_registry.rs')"
assert_eq "$(extract_output "$did_contract_rust_output" "run_rust")" "true" "did registry rust changes should run rust lane"
assert_eq "$(extract_output "$did_contract_rust_output" "run_did_registry_contract_tests")" "true" "did registry rust changes must run did registry contract lane"
assert_eq "$(extract_output "$did_contract_rust_output" "test_scope")" "targeted" "did registry rust changes should stay targeted"

did_contract_script_output="$(run_selector $'scripts/did/run_did_registry_contract_lane.sh')"
assert_eq "$(extract_output "$did_contract_script_output" "run_rust")" "false" "did contract script-only changes should avoid rust lane"
assert_eq "$(extract_output "$did_contract_script_output" "run_did_registry_contract_tests")" "true" "did contract script changes must run did registry contract lane"
assert_eq "$(extract_output "$did_contract_script_output" "test_scope")" "did-contract" "did contract script changes should set did-contract scope"

runtime_contract_docs_output="$(run_selector $'docs/foundation/runtime-network.md')"
assert_eq "$(extract_output "$runtime_contract_docs_output" "run_rust")" "false" "runtime contract docs should avoid rust lane"
assert_eq "$(extract_output "$runtime_contract_docs_output" "run_runtime_snapshot_contract_tests")" "true" "runtime contract docs must run runtime snapshot contract lane"
assert_eq "$(extract_output "$runtime_contract_docs_output" "test_scope")" "runtime-contract" "runtime contract docs should set runtime-contract scope"

runtime_watchdog_contract_docs_output="$(run_selector $'docs/foundation/runtime-watchdog-attestation.md')"
assert_eq "$(extract_output "$runtime_watchdog_contract_docs_output" "run_rust")" "false" "runtime watchdog contract docs should avoid rust lane"
assert_eq "$(extract_output "$runtime_watchdog_contract_docs_output" "run_runtime_snapshot_contract_tests")" "true" "runtime watchdog contract docs must run runtime snapshot contract lane"
assert_eq "$(extract_output "$runtime_watchdog_contract_docs_output" "test_scope")" "runtime-contract" "runtime watchdog contract docs should set runtime-contract scope"

runtime_contract_script_output="$(run_selector $'scripts/runtime/run_runtime_snapshot_contract_lane.sh')"
assert_eq "$(extract_output "$runtime_contract_script_output" "run_rust")" "false" "runtime contract script-only changes should avoid rust lane"
assert_eq "$(extract_output "$runtime_contract_script_output" "run_runtime_snapshot_contract_tests")" "true" "runtime contract script changes must run runtime snapshot contract lane"
assert_eq "$(extract_output "$runtime_contract_script_output" "test_scope")" "runtime-contract" "runtime contract script changes should set runtime-contract scope"

message_contract_docs_output="$(run_selector $'docs/foundation/message-lifecycle.md')"
assert_eq "$(extract_output "$message_contract_docs_output" "run_rust")" "false" "message lifecycle contract docs should avoid rust lane"
assert_eq "$(extract_output "$message_contract_docs_output" "run_message_lifecycle_contract_tests")" "true" "message lifecycle contract docs must run message lifecycle contract lane"
assert_eq "$(extract_output "$message_contract_docs_output" "test_scope")" "message-contract" "message lifecycle contract docs should set message-contract scope"

message_contract_script_output="$(run_selector $'scripts/message/run_message_lifecycle_contract_lane.sh')"
assert_eq "$(extract_output "$message_contract_script_output" "run_rust")" "false" "message lifecycle contract script-only changes should avoid rust lane"
assert_eq "$(extract_output "$message_contract_script_output" "run_message_lifecycle_contract_tests")" "true" "message lifecycle contract script changes must run message lifecycle contract lane"
assert_eq "$(extract_output "$message_contract_script_output" "test_scope")" "message-contract" "message lifecycle contract script changes should set message-contract scope"

channel_contract_docs_output="$(run_selector $'docs/foundation/channel-models.md')"
assert_eq "$(extract_output "$channel_contract_docs_output" "run_rust")" "false" "channel lifecycle contract docs should avoid rust lane"
assert_eq "$(extract_output "$channel_contract_docs_output" "run_channel_lifecycle_contract_tests")" "true" "channel lifecycle contract docs must run channel lifecycle contract lane"
assert_eq "$(extract_output "$channel_contract_docs_output" "test_scope")" "channel-contract" "channel lifecycle contract docs should set channel-contract scope"

channel_contract_script_output="$(run_selector $'scripts/channel/run_channel_lifecycle_contract_lane.sh')"
assert_eq "$(extract_output "$channel_contract_script_output" "run_rust")" "false" "channel lifecycle contract script-only changes should avoid rust lane"
assert_eq "$(extract_output "$channel_contract_script_output" "run_channel_lifecycle_contract_tests")" "true" "channel lifecycle contract script changes must run channel lifecycle contract lane"
assert_eq "$(extract_output "$channel_contract_script_output" "test_scope")" "channel-contract" "channel lifecycle contract script changes should set channel-contract scope"

task_contract_docs_output="$(run_selector $'docs/foundation/task-operations.md')"
assert_eq "$(extract_output "$task_contract_docs_output" "run_rust")" "false" "task operation contract docs should avoid rust lane"
assert_eq "$(extract_output "$task_contract_docs_output" "run_task_operation_snapshot_contract_tests")" "true" "task operation contract docs must run task operation snapshot contract lane"
assert_eq "$(extract_output "$task_contract_docs_output" "test_scope")" "task-contract" "task operation contract docs should set task-contract scope"

task_contract_script_output="$(run_selector $'scripts/task/run_task_operation_snapshot_contract_lane.sh')"
assert_eq "$(extract_output "$task_contract_script_output" "run_rust")" "false" "task operation contract script-only changes should avoid rust lane"
assert_eq "$(extract_output "$task_contract_script_output" "run_task_operation_snapshot_contract_tests")" "true" "task operation contract script changes must run task operation snapshot contract lane"
assert_eq "$(extract_output "$task_contract_script_output" "test_scope")" "task-contract" "task operation contract script changes should set task-contract scope"

runtime_rust_output="$(run_selector $'crates/kamn-core/src/runtime.rs')"
assert_eq "$(extract_output "$runtime_rust_output" "run_rust")" "true" "runtime rust changes should run rust lane"
assert_eq "$(extract_output "$runtime_rust_output" "run_runtime_snapshot_contract_tests")" "false" "runtime rust changes should avoid duplicate runtime contract lane"
assert_eq "$(extract_output "$runtime_rust_output" "test_scope")" "targeted" "runtime rust changes should stay targeted"

message_rust_output="$(run_selector $'crates/kamn-core/src/message_lifecycle.rs')"
assert_eq "$(extract_output "$message_rust_output" "run_rust")" "true" "message lifecycle rust changes should run rust lane"
assert_eq "$(extract_output "$message_rust_output" "run_message_lifecycle_contract_tests")" "false" "message lifecycle rust changes should avoid duplicate message contract lane"
assert_eq "$(extract_output "$message_rust_output" "test_scope")" "targeted" "message lifecycle rust changes should stay targeted"

channel_rust_output="$(run_selector $'crates/kamn-core/src/channel_models.rs')"
assert_eq "$(extract_output "$channel_rust_output" "run_rust")" "true" "channel lifecycle rust changes should run rust lane"
assert_eq "$(extract_output "$channel_rust_output" "run_channel_lifecycle_contract_tests")" "false" "channel lifecycle rust changes should avoid duplicate channel contract lane"
assert_eq "$(extract_output "$channel_rust_output" "test_scope")" "targeted" "channel lifecycle rust changes should stay targeted"

task_rust_output="$(run_selector $'crates/kamn-core/src/task_operations.rs')"
assert_eq "$(extract_output "$task_rust_output" "run_rust")" "true" "task operation rust changes should run rust lane"
assert_eq "$(extract_output "$task_rust_output" "run_task_operation_snapshot_contract_tests")" "false" "task operation rust changes should avoid duplicate task contract lane"
assert_eq "$(extract_output "$task_rust_output" "test_scope")" "targeted" "task operation rust changes should stay targeted"

echo "select_targets matrix regression tests passed."
