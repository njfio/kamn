#!/usr/bin/env bash
set -euo pipefail

cargo test -p kamn-core --lib task_operations::tests:: >/dev/null
cargo test -p kamn-core --test task_operations >/dev/null
cargo test -p kamn-core --test task_operation_snapshot >/dev/null
cargo test -p kamn-core --test task_state_machine >/dev/null
cargo test -p kamn-core --test task_state_machine_docs >/dev/null
cargo test -p kamn-core --test task_escrow_transition_contracts >/dev/null
cargo test -p kamn-core --test task_operations_docs >/dev/null

echo "task operation snapshot contract lane tests passed."
