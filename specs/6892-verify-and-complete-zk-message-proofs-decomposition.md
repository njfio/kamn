# 6892 — Verify and complete zk_message_proofs.rs decomposition on current main

## Objective
Verify the exact current-main state of `crates/kamn-core/src/zk_message_proofs.rs` and complete the decomposition if the file is still oversized. The target end state is a thin root shell with bounded concern-based modules while preserving proof-planning, processor-admission, validator-consensus, watchdog-projection, witness-building, and serialization behavior under the current tests.

## Inputs/Outputs
### Inputs
- Current `origin/main` implementation at `crates/kamn-core/src/zk_message_proofs.rs`
- Existing `zk_message_proofs` unit tests and any downstream compile-time consumers
- Current AGENTS.md size policy and touched-Rust ratchet

### Outputs
- Bounded module layout under `crates/kamn-core/src/zk_message_proofs/`
- Thin root `zk_message_proofs.rs`
- Hard-fail extraction contract verifying the intended root/module layout and staged file budgets
- Updated spec evidence documenting the verified current-main state and any mismatch against prior assumptions

## Boundaries / Non-goals
- No changes to proof semantics, crypto policy, or public proof reason codes except decomposition-safe reshaping if strictly required
- No new dependencies
- No broad redesign outside `zk_message_proofs`
- No weakening of existing tests or touched-Rust policy

## Failure modes
- Extraction contract passes while the root file remains oversized or inline monolith sections remain in place
- Proof generation, validator-consensus, or watchdog projection behavior regresses during the split
- Public exports drift and downstream tests fail to compile
- Refactor introduces oversized touched files/functions and fails touched-Rust policy
- The current baseline differs from prior assumptions and the spec fails to record that mismatch

## Acceptance criteria
- [ ] Verified current `origin/main` state of `crates/kamn-core/src/zk_message_proofs.rs` is recorded in this spec
- [ ] If still oversized, `crates/kamn-core/src/zk_message_proofs.rs` is reduced to a thin root shell under the active staged budget
- [ ] Concern-based extracted modules exist for planning/evaluation, processor admission, validator consensus, watchdog projection, witness building, errors, and tests
- [ ] A hard-fail extraction contract enforces root/module layout and staged file budgets
- [ ] Existing `zk_message_proofs` tests pass unchanged in meaning
- [ ] Touched-Rust size policy returns `policy_decision=GO`
- [ ] Any mismatch between prior assumptions and the current verified baseline is documented in this spec

## Files to touch
- `specs/6892-verify-and-complete-zk-message-proofs-decomposition.md`
- `crates/kamn-core/src/zk_message_proofs.rs`
- `crates/kamn-core/src/zk_message_proofs/`
- `crates/kamn-core/tests/zk_message_proofs_module_extraction_contract.rs`
- Existing `zk_message_proofs` tests only if needed to preserve compile-time wiring after the split

## Error semantics
- Extraction contract failures must hard-fail with concrete missing-path, marker, or line-budget messages
- Runtime proof logic must preserve existing typed error behavior and error strings
- No silent fallback to old inline code paths

## Test plan
1. Add a red extraction contract for the intended module layout and staged root budget
2. Confirm the new contract fails on current `origin/main`
3. Extract the module tree with minimum code motion required to satisfy the contract
4. Run:
   - `cargo test -p kamn-core zk_message_proofs::tests:: --lib -- --nocapture`
   - `cargo test -p kamn-core --test zk_message_proofs_module_extraction_contract -- --nocapture`
5. Run touched-Rust size policy against the clean issue branch

## Verified current-main state
- On `origin/main` at issue start, `crates/kamn-core/src/zk_message_proofs.rs` is `1482` LOC
- No extracted `crates/kamn-core/src/zk_message_proofs/` module tree exists on the verified baseline
- Inline `mod tests` remains embedded in the root file
- Current split seams are visible in the live source around planning/evaluation, processor admission, validator consensus, watchdog projection, witness building, and error formatting
