# Objective

Extract the oversized test surface from `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs` into bounded sibling modules while preserving the real command verification-matrix coverage and runtime wiring in `kamn-e2e-harness`.

# Inputs/Outputs

Inputs:
- `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs`
- existing verification-matrix helpers from `support::command_contract_support`
- the real `kamn-e2e-harness` verification-matrix test target

Outputs:
- bounded module tree under `crates/kamn-e2e-harness/tests/command_contract_verify_matrix/`
- extraction contract covering the staged root budget and required module layout
- reduced root `command_contract_verify_matrix.rs`

# Boundaries/Non-goals

- Do not change production `kamn-e2e-harness` command behavior or report semantics.
- Do not rewrite unrelated e2e harness tests.
- Do not weaken existing verify/run-output persistence or probe-failure assertions.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted sibling files exceed the active file-size budget
- moved tests lose verification-matrix coverage or fixture cleanup semantics
- run/verify assertions drift during extraction
- touched-Rust ratchet fails on newly oversized touched files or functions

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs` into bounded sibling modules organized by concern
- [x] root `command_contract_verify_matrix.rs` is reduced below a staged extraction cap enforced by a new extraction contract
- [x] extracted sibling files stay within the active file-size budget
- [x] the real `kamn-e2e-harness` verification-matrix target remains wired and passes
- [x] `cargo test -p kamn-e2e-harness --test command_contract_verify_matrix -- --nocapture` passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs`
- `crates/kamn-e2e-harness/tests/command_contract_verify_matrix/**`
- `crates/kamn-e2e-harness/tests/*extraction_contract*.rs`
- `specs/6792-split-command-contract-verify-matrix.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module, marker, or budget diagnostics.
- Existing verification-matrix failures remain ordinary Rust assertion failures with no silent fallbacks.
- Fixture setup/cleanup failures remain hard-fail test errors.

# Test plan

1. Add a red extraction contract asserting the module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the root file into bounded sibling modules and nested helper modules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-e2e-harness --test command_contract_verify_matrix -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6792-touched-size.json`.

# Planned module seams

- `verify_manifest_contract_tests.rs` for manifest marker validation failures
- `verify_evidence_contract_tests.rs` for evidence artifact verification-block validation failures
- `verify_chain_dump_contract_tests.rs` for chain dump and continuity failures
- `run_probe_contract_tests.rs` for external execution probe/run-output failure projection coverage
- `run_persistence_contract_tests.rs` for pass/fail evidence persistence coverage

# Phase 6 evidence

- Root shell reduced to `14` LOC at `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs`.
- Extracted sibling modules are all within the active `<=200` LOC file budget.
- Real integration path verified with:
  - `cargo test -p kamn-e2e-harness --test command_contract_verify_matrix -- --nocapture`
  - `cargo test -p kamn-e2e-harness --test command_contract_verify_matrix_extraction_contract -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6792-touched-size-refactor.json`
- Touched-Rust result: `policy_decision=GO`

# Deviations

- None.
