# Objective
Split `crates/kamn-core/src/message_lifecycle.rs` into bounded, concern-based modules while preserving message lifecycle behavior, snapshot-store semantics, parser behavior, and existing tests/contracts.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/message_lifecycle.rs`
  - existing lifecycle store API, snapshot-store backends, codec helpers, and tests
- Outputs:
  - a thin `message_lifecycle.rs` root shell under the active file-size budget
  - bounded sibling modules for lifecycle state/store logic, snapshot-store backends, snapshot codec/journal helpers, and tests
  - extraction contract coverage that enforces the new module layout and active size limits

## Boundaries/Non-goals
- Do not change message lifecycle semantics or public behavior.
- Do not change snapshot payload schema or journal record format.
- Do not change public crate APIs except internal module placement.
- Do not introduce new dependencies.

## Failure modes
- Root file remains oversized after the split.
- Extracted modules cross concerns and duplicate lifecycle logic.
- Snapshot parsing, journaling, or store recovery behavior drifts.
- Extracted functions or files still exceed active size limits.

## Acceptance criteria
- [ ] The root file is reduced to a thin shell under the active file-size budget.
- [ ] Concern-based submodules are introduced for the extracted logic.
- [ ] Existing parser/runtime/tests/docs contracts remain green.
- [ ] No extracted file exceeds the active file-size limit.
- [ ] No extracted function exceeds the active function-size limit.

## Files to touch
- `specs/6850-split-message-lifecycle.md`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/tests/message_lifecycle_module_extraction_contract.rs`
- optional sibling modules under `crates/kamn-core/src/message_lifecycle/`

## Error semantics
- Existing typed lifecycle and snapshot-store errors must remain fail-closed.
- Parsing and recovery helpers must keep returning deterministic typed errors.
- No silent fallbacks may be introduced during the split.

## Test plan
1. Add a red extraction contract that fails while the root file remains oversized and the planned module layout is absent.
2. Re-run the existing `message_lifecycle` test target to preserve behavior while splitting.
3. Extract the file into bounded concern-based modules.
4. Re-run the extraction contract and the real `message_lifecycle` test target until green.
5. Run the touched-Rust size ratchet on the final write set.

## Phase 6 Evidence
- `TMPDIR=/home/n/Code/kamn/.tmp-home CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test message_lifecycle_module_extraction_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/.tmp-home CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core message_lifecycle::tests:: -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/.tmp-home CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test message_lifecycle_queries -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/.tmp-home CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test message_lifecycle_proof_admission -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/.tmp-home CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test sqlite_snapshot_store_adapters -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6850-remote-clean4 --base-ref origin/main --output-json /tmp/6850-clean4-python-size.json`

## Deviations
- The shell wrapper `scripts/ci/check_touched_rust_size_policy.sh` resolved the primary checkout instead of the clean clone during validation. Final ratchet evidence therefore used the Python entrypoint with explicit `--repo-root /home/n/Code/kamn-6850-remote-clean4`.
