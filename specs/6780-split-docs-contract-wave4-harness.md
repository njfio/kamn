# Objective

Extract the oversized test surface from `crates/kamn-core/tests/docs_contract_wave4_harness.rs` into bounded sibling modules while preserving the existing docs-harness contract assertions.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/docs_contract_wave4_harness.rs`
- referenced docs, fixtures, and harness helpers exercised by the target

Outputs:
- bounded module tree under `crates/kamn-core/tests/docs_contract_wave4_harness/`
- extraction contract covering the new layout and staged root budget
- reduced root `docs_contract_wave4_harness.rs`

# Boundaries/Non-goals

- Do not change docs policy semantics or marker expectations.
- Do not broaden the issue outside `docs_contract_wave4_harness.rs` and its new sibling modules.
- Do not rewrite unrelated docs contract targets.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- module markers drift and the target no longer compiles
- existing docs assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [ ] root test surface is extracted from `crates/kamn-core/tests/docs_contract_wave4_harness.rs` into bounded sibling modules
- [ ] root `docs_contract_wave4_harness.rs` is reduced below the staged extraction cap enforced by the new contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] `cargo test -p kamn-core --test docs_contract_wave4_harness -- --nocapture` passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/docs_contract_wave4_harness.rs`
- `crates/kamn-core/tests/docs_contract_wave4_harness/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6780-split-docs-contract-wave4-harness.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Existing docs harness failures remain ordinary Rust assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules and nested submodules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-core --test docs_contract_wave4_harness -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6778-remote --base-ref origin/main --output-json /tmp/6780-touched-size.json`.

# Phase 6 evidence

- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test docs_contract_wave4_harness_extraction_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test docs_contract_wave4_harness -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6778-remote --base-ref origin/main --output-json /tmp/6780-touched-size-go.json`

# Deviations

- Extraction required rebasing all `include_str!` paths one level deeper because the harness modules moved from the test root into `tests/docs_contract_wave4_harness/`.
- Verification stayed in isolated remote clone `/tmp/kamn-6778-remote` to keep the touched-Rust gate authoritative for the `#6780` write set.
