# Objective

Extract the oversized docs-contract surface from `crates/kamn-e2e-harness/tests/docs_contract_phase_group.rs` into bounded sibling modules while preserving the real `docs_contract_phase_group` test target and all current milestone/docs marker assertions.

# Inputs/Outputs

Inputs:
- `crates/kamn-e2e-harness/tests/docs_contract_phase_group.rs`
- existing docs and milestone index files referenced by the current test target
- the real `cargo test -p kamn-e2e-harness --test docs_contract_phase_group -- --nocapture` target

Outputs:
- bounded module tree under `crates/kamn-e2e-harness/tests/docs_contract_phase_group/`
- extraction contract covering the staged root budget and required module layout
- reduced root `docs_contract_phase_group.rs`

# Boundaries/Non-goals

- Do not change production `kamn-e2e-harness` behavior.
- Do not weaken or delete existing docs marker or milestone index assertions.
- Do not rewrite unrelated docs harness tests outside this target.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted sibling files exceed the active file-size budget
- moved tests stop exercising the real `docs_contract_phase_group` target
- docs marker or milestone issue assertions drift during extraction
- touched-Rust ratchet fails on newly oversized touched files or functions

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-e2e-harness/tests/docs_contract_phase_group.rs` into bounded sibling modules organized by phase group
- [x] root `docs_contract_phase_group.rs` is reduced below a staged extraction cap enforced by a new extraction contract
- [x] extracted sibling files stay within the active file-size budget
- [x] the real `cargo test -p kamn-e2e-harness --test docs_contract_phase_group -- --nocapture` target remains wired and passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-e2e-harness/tests/docs_contract_phase_group.rs`
- `crates/kamn-e2e-harness/tests/docs_contract_phase_group/**`
- `crates/kamn-e2e-harness/tests/*extraction_contract*.rs`
- `specs/6794-split-docs-contract-phase-group.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module, marker, or budget diagnostics.
- Existing docs-contract failures remain ordinary Rust assertion failures with no silent fallbacks.
- Docs and milestone fixtures must continue to fail loudly when required files or markers are missing.

# Test plan

1. Add a red extraction contract asserting the root shell budget and required module layout.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the root file into bounded sibling modules and nested helpers where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-e2e-harness --test docs_contract_phase_group -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6794-touched-size.json`.

# Planned module seams

- `phase4_docs_contract_tests.rs` for phase 4b-4j marker and milestone checks
- `phase5_docs_contract_tests.rs` for phase 5a-5d marker and milestone checks
- `runtime_phase6_docs_contract_tests.rs` for phase-6 runtime integration/lifecycle/orchestration/validation checks
- `phase6_docs_contract_tests.rs` for phase 6a-6d marker and milestone checks
- `structure_docs_contract_tests.rs` for shared structure and phase4a harness checks

# Phase 6 evidence

- Root shell reduced to `12` LOC at `crates/kamn-e2e-harness/tests/docs_contract_phase_group.rs`.
- Extracted module tree totals `357` LOC across bounded sibling files; the largest touched file is `62` LOC.
- Real integration path verified with:
  - `cargo test -p kamn-e2e-harness --test docs_contract_phase_group_extraction_contract -- --nocapture`
  - `cargo test -p kamn-e2e-harness --test docs_contract_phase_group -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6794-touched-size-go.json`
- Touched-Rust result: `policy_decision=GO`

# Deviations

- None.
