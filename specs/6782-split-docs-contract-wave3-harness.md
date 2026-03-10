# Objective

Extract the oversized test surface from `crates/kamn-core/tests/docs_contract_wave3_harness.rs` into bounded sibling modules while preserving the existing docs-harness contract assertions.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/docs_contract_wave3_harness.rs`
- referenced docs, fixtures, and harness helpers exercised by the target

Outputs:
- bounded module tree under `crates/kamn-core/tests/docs_contract_wave3_harness/`
- extraction contract covering the new layout and staged root budget
- reduced root `docs_contract_wave3_harness.rs`

# Boundaries/Non-goals

- Do not change docs policy semantics or marker expectations.
- Do not broaden the issue outside `docs_contract_wave3_harness.rs` and its new sibling modules.
- Do not rewrite unrelated docs contract targets.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- module markers drift and the target no longer compiles
- existing docs assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-core/tests/docs_contract_wave3_harness.rs` into bounded sibling modules
- [x] root `docs_contract_wave3_harness.rs` is reduced below the staged extraction cap enforced by the new contract
- [x] extracted sibling files stay within the active file-size budget
- [x] `cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture` passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/docs_contract_wave3_harness.rs`
- `crates/kamn-core/tests/docs_contract_wave3_harness/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6782-split-docs-contract-wave3-harness.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Existing docs harness failures remain ordinary Rust assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules and nested submodules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6778-remote --base-ref origin/main --output-json /tmp/6782-touched-size.json`.

# Phase 6 Evidence

- Root shell reduced to `73` LOC in `crates/kamn-core/tests/docs_contract_wave3_harness.rs`.
- Extracted 35 sibling modules under `crates/kamn-core/tests/docs_contract_wave3_harness/`.
- Verified with:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test docs_contract_wave3_harness_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6778-remote --base-ref origin/main --output-json /tmp/6782-touched-size-staged.json`
- Final touched-Rust result: `policy_decision=GO`

# Deviations

- Mechanical extraction required `include_str!` path rewrites from `../../../` to `../../../../` because the contract modules moved one directory deeper.
