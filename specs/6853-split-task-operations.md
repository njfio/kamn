# Objective
Split `crates/kamn-core/src/task_operations.rs` into bounded, concern-based modules while preserving task lifecycle semantics, dependency gating, snapshot persistence behavior, and existing task-operation tests.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/task_operations.rs`
  - existing task lifecycle and snapshot-journal dependencies
  - current task-operation and data-layer tests
- Outputs:
  - a thin `task_operations.rs` root shell under the active file-size budget
  - bounded sibling modules for engine transitions, snapshot stores, journal codecs, and recovery helpers
  - extraction contract coverage enforcing the new module layout and active size limits

## Boundaries/Non-goals
- Do not change task lifecycle semantics.
- Do not rework unrelated escrow or channel behavior.
- Do not introduce new dependencies.
- Do not change public error semantics or snapshot schema behavior.

## Failure modes
- Root file remains oversized after the split.
- Extracted modules drift task transition semantics or dependency validation.
- Snapshot export/restore or journal replay behavior changes.
- Extracted files or functions still exceed active size limits.

## Acceptance criteria
- [x] The root file is reduced to a thin shell under the active file-size budget.
- [x] Task transition / validation / persistence seams are extracted into bounded modules.
- [x] Existing task-operation tests remain green.
- [x] No extracted file exceeds the active file-size limit.
- [x] No extracted function exceeds the active function-size limit.

## Files to touch
- `specs/6853-split-task-operations.md`
- `crates/kamn-core/src/task_operations.rs`
- `crates/kamn-core/tests/task_operations_module_extraction_contract.rs`
- optional sibling modules under `crates/kamn-core/src/task_operations/`

## Error semantics
- Existing `TaskOperationError` and `TaskOperationSnapshotStoreError` behavior remains fail-closed.
- Snapshot schema version, invalid snapshot detection, and dependency validation must remain deterministic.
- No silent fallback behavior may be introduced during the split.

## Test plan
1. Add a red extraction contract that fails while the root file remains oversized and the planned module layout is absent.
2. Re-run the existing task-operation tests that cover lifecycle transitions, snapshot export/restore, and store backends.
3. Extract the file into bounded concern-based modules.
4. Re-run the extraction contract and task-operation targets until green.
5. Run the clean-clone touched-Rust size ratchet on the final write set.

## Phase 6 Evidence
- `cargo test -p kamn-core --test task_operations_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test task_operations -- --nocapture`
- `cargo test -p kamn-core task_operations::tests:: -- --nocapture`
- `cargo test -p kamn-core --test task_operation_snapshot -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6853-clean-QjFONC --base-ref origin/main --output-json /tmp/6853-touched-size-post-refactor.json`

## Deviations
- The shell wrapper `scripts/ci/check_touched_rust_size_policy.sh` was not used for final clean-clone validation because it resolved the primary checkout rather than the clean-clone repo root. The Python entrypoint was used directly so the full extracted `task_operations/**` write set was evaluated.
