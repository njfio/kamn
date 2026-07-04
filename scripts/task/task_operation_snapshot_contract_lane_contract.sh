#!/usr/bin/env bash
set -euo pipefail

task_operation_snapshot_target_dir="${KAMN_TASK_OPERATION_SNAPSHOT_TARGET_DIR:-target/task-operation-snapshot-contract}"
mkdir -p "$task_operation_snapshot_target_dir"

run_cargo_test() {
  CARGO_TARGET_DIR="$task_operation_snapshot_target_dir" cargo test "$@" >/dev/null
}

run_cargo_test -p kamn-core --lib task_operations::tests::
run_cargo_test -p kamn-core --test task_operations
run_cargo_test -p kamn-core --test task_operation_snapshot
run_cargo_test -p kamn-core --test task_state_machine
run_cargo_test -p kamn-core --test docs_contract_wave3_harness task_state_machine_docs::
run_cargo_test -p kamn-core --test task_escrow_transition_contracts
run_cargo_test -p kamn-core --test task_operations_docs

echo "task operation snapshot contract lane tests passed."
