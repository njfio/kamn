# 6838-split-message-lifecycle

## Objective
Reduce `crates/kamn-core/src/message_lifecycle.rs` from a 1832 LOC monolith to a thin root shell plus bounded concern modules without changing message lifecycle behavior, public API semantics, or runtime error behavior.

## Inputs/Outputs
- Input: existing message lifecycle store, snapshot, proof-admission, and journal helpers in `crates/kamn-core/src/message_lifecycle.rs`
- Output: bounded sibling modules and a root shell that wires the message lifecycle surface together
- Output: structural ratchet coverage that enforces the staged extraction layout on touched code

## Boundaries/Non-goals
- No new dependencies
- No changes to public API contracts unless required by file/module movement alone
- No behavior changes in lifecycle transitions, snapshot semantics, journal persistence, or proof-admission rules
- No weakening of existing tests or invariants

## Failure Modes
- Root shell remains above the staged extraction cap
- Any touched extracted file exceeds 200 LOC
- Public exports drift and downstream crates fail to compile
- Snapshot validation, transition validation, or journal persistence semantics change during extraction
- Touched-Rust size policy remains `NO-GO`

## Acceptance Criteria
- [ ] `crates/kamn-core/src/message_lifecycle.rs` is reduced to a thin root shell at or below a staged extraction cap defined by the red contract
- [ ] Extracted sibling modules are organized by message lifecycle concern rather than arbitrary line slicing
- [ ] All touched extracted files remain at or below 200 LOC
- [ ] Existing tests that exercise message lifecycle behavior still pass
- [ ] At least one new extraction contract enforces the staged root shell and module layout
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

## Files To Touch
- `specs/6838-split-message-lifecycle.md`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/message_lifecycle/`
- `crates/kamn-core/tests/message_lifecycle_module_extraction_contract.rs`

## Error Semantics
- Preserve all existing `MessageLifecycleError`, `MessageLifecycleSnapshotError`, `MessageProofAdmissionError`, and related error text/variants unchanged
- Preserve current fail-closed behavior for invalid snapshots, transitions, proof admission, and journal operations
- Structural contract failures must fail with explicit missing-file, missing-marker, or root-budget assertions

## Test Plan
1. Add a red extraction contract for `crates/kamn-core/src/message_lifecycle.rs`.
2. Split the file into bounded modules by concern:
   - lifecycle types and shared structs
   - store registration/status/participant indexing
   - snapshot export/restore validation
   - proof admission and validation wiring
   - journal persistence and replay helpers
   - error and formatting types if needed
3. Run the extraction contract and existing message lifecycle tests.
4. Run the touched-Rust size ratchet and require `policy_decision=GO`.
