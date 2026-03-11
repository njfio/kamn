# 6880 — Verify and complete task_operations.rs decomposition on current main

## Objective
Verify the actual `origin/main` state of `crates/kamn-core/src/task_operations.rs` and complete the source decomposition if the file is still oversized. The desired end state is a thin root shell with bounded concern-based modules while preserving current task-operation behavior, snapshot persistence semantics, and existing test coverage.

## Inputs/Outputs
### Inputs
- Current `origin/main` implementation at `crates/kamn-core/src/task_operations.rs`
- Existing task operation tests and snapshot-store integration tests
- Current AGENTS.md size policy and touched-Rust ratchet

### Outputs
- Bounded module layout under `crates/kamn-core/src/task_operations/`
- Thin root `task_operations.rs`
- Hard-fail extraction contract verifying the intended layout and staged root budget
- Updated spec evidence documenting the verified current-main state and any deviation from prior issue history

## Boundaries / Non-goals
- No changes to task semantics, lifecycle rules, or public behavior except decomposition-safe signature reshaping if strictly required
- No broad runtime redesign outside task operations
- No weakening of existing tests or touched-Rust policy
- No speculative architectural extraction into new crates in this issue

## Failure modes
- Extraction contract still passes while the root file remains oversized or inline sections remain in place
- Snapshot store roundtrip or journal recovery behavior regresses during the split
- Public exports drift and downstream tests fail to compile
- Refactor introduces oversized touched files/functions and fails touched-Rust policy
- Current `main` already differs from prior merged issue history; the spec must record that deviation rather than assuming the earlier decomposition landed

## Acceptance criteria
- [ ] Verified current `origin/main` state of `crates/kamn-core/src/task_operations.rs` is recorded in this spec
- [ ] If still oversized, `crates/kamn-core/src/task_operations.rs` is reduced to a thin root shell under the active staged budget
- [ ] Concern-based extracted modules exist for `models`, `engine`, `snapshot_store`, `snapshot_codec`, and `tests`
- [ ] A hard-fail extraction contract enforces root/module layout and staged file budgets
- [ ] Real task operation tests and snapshot/store integration tests pass unchanged in meaning
- [ ] Touched-Rust size policy returns `policy_decision=GO`
- [ ] Any mismatch between prior decomposition claims and the current baseline is documented in this spec

## Files to touch
- `specs/6880-verify-and-complete-task-operations-decomposition.md`
- `crates/kamn-core/src/task_operations.rs`
- `crates/kamn-core/src/task_operations/`
- `crates/kamn-core/tests/task_operations_module_extraction_contract.rs`
- Existing task operation and snapshot tests only if needed to preserve compile-time wiring after the split

## Error semantics
- Extraction contract failures must hard-fail with concrete missing-path, marker, or line-budget messages
- Runtime/task-operation logic must preserve existing typed error behavior and reason codes
- No silent fallback to old inline code paths

## Test plan
1. Add a red extraction contract for the intended module layout and staged root budget
2. Confirm the new contract fails on current `origin/main`
3. Extract the module tree with the minimum code motion required to satisfy the contract
4. Run:
   - `cargo test -p kamn-core --test task_operations_module_extraction_contract -- --nocapture`
   - `cargo test -p kamn-core --test task_operations -- --nocapture`
   - `cargo test -p kamn-core task_operations::tests:: -- --nocapture`
   - `cargo test -p kamn-core --test task_operation_snapshot -- --nocapture`
5. Run touched-Rust size policy against the clean issue branch

## Verified current-main state
- On `origin/main` at issue start, `crates/kamn-core/src/task_operations.rs` is still `1685` LOC
- No extracted `crates/kamn-core/src/task_operations/` module tree exists on the verified baseline
- This means prior decomposition work did not land on the current verified baseline and this issue must complete the split from the live state rather than assuming earlier issue history is authoritative
