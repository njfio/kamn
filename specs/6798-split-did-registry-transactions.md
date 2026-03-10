# Objective

Extract the oversized DID-registry transaction test surface from `crates/kamn-core/tests/did_registry_transactions.rs` into bounded sibling modules while preserving the real `did_registry_transactions` test target and all current transaction, retry, finality, chain-submission, and lifecycle assertions.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/did_registry_transactions.rs`
- existing `kamn-core` DID-registry transaction helpers and typed outcomes
- the real `cargo test -p kamn-core --test did_registry_transactions -- --nocapture` target

Outputs:
- bounded module tree under `crates/kamn-core/tests/did_registry_transactions/`
- extraction contract covering the staged root budget and required module layout
- reduced root `did_registry_transactions.rs`

# Boundaries/Non-goals

- Do not change production `kamn-core` DID-registry behavior.
- Do not weaken or delete existing transaction coverage.
- Do not rewrite unrelated DID-registry or other data-layer tests.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted sibling files exceed the active file-size budget
- moved tests stop exercising the real `did_registry_transactions` target
- retry/finality or lifecycle assertions drift during extraction
- touched-Rust ratchet fails on newly oversized touched files or functions

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-core/tests/did_registry_transactions.rs` into bounded sibling modules organized by transaction concern
- [x] root `did_registry_transactions.rs` is reduced below a staged extraction cap enforced by a new extraction contract
- [x] extracted sibling files stay within the active file-size budget
- [x] the real `cargo test -p kamn-core --test did_registry_transactions -- --nocapture` target remains wired and passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/did_registry_transactions.rs`
- `crates/kamn-core/tests/did_registry_transactions/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6798-split-did-registry-transactions.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module, marker, or budget diagnostics.
- Existing DID-registry transaction failures remain ordinary Rust assertion failures with no silent fallbacks.
- Typed retry/finality and lifecycle outcome assertions must continue to fail loudly when behavior drifts.

# Test plan

1. Add a red extraction contract asserting the root shell budget and required module layout.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the root file into bounded sibling modules and nested helpers where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test did_registry_transactions -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6798-touched-size.json`.

# Planned module seams

- `registry_flow_contract_tests.rs` for register/update/revoke base flows
- `retry_finality_contract_tests.rs` for duplicate/retry/finality boundary coverage
- `chain_submission_contract_tests.rs` for chain-submission adapter outcomes and malformed-payload regressions
- `lifecycle_mutation_contract_tests.rs` for nonce, rotate, recover, replay, and mutation contract-lane coverage

# Phase 6 evidence

- Root shell reduced to `10` LOC at `crates/kamn-core/tests/did_registry_transactions.rs`.
- Extracted module tree totals `422` LOC across bounded sibling files; the largest touched file is `145` LOC.
- Real integration path verified with:
  - `cargo test -p kamn-core --test did_registry_transactions_extraction_contract -- --nocapture`
  - `cargo test -p kamn-core --test did_registry_transactions -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6798-touched-size-refactor.json`
- Touched-Rust result: `policy_decision=GO`

# Deviations

- None.
