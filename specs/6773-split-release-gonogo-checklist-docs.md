# Objective

Extract the oversized root docs-contract surface from `crates/kamn-core/tests/release_gonogo_checklist_docs.rs` into bounded sibling modules while preserving the existing release checklist assertions against the checked-in docs sources.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- referenced release checklist docs and fixtures

Outputs:
- bounded module tree under `crates/kamn-core/tests/release_gonogo_checklist_docs/`
- extraction contract covering the new layout and staged root budget
- reduced root `release_gonogo_checklist_docs.rs`

# Boundaries/Non-goals

- Do not change release checklist doc semantics.
- Do not broaden the issue outside `release_gonogo_checklist_docs.rs` and its new sibling modules.
- Do not rewrite unrelated docs-contract test files.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- the docs-contract target no longer compiles because module markers or helper visibility drift
- existing release checklist assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [ ] root docs-contract tests are extracted from `crates/kamn-core/tests/release_gonogo_checklist_docs.rs` into bounded sibling modules
- [ ] root `release_gonogo_checklist_docs.rs` is reduced below the staged extraction cap enforced by the new contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] `cargo test -p kamn-core --test release_gonogo_checklist_docs -- --nocapture` passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6773-split-release-gonogo-checklist-docs.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Runtime test failures remain ordinary Rust test assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and a staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the file into bounded sibling modules.
4. Run `cargo test -p kamn-core --test release_gonogo_checklist_docs -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6773-touched-size.json`.

# Phase 6 Evidence

- `cargo test -p kamn-core --test release_gonogo_checklist_docs_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test release_gonogo_checklist_docs -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6773-touched-size-refactor.json`
- Root shell result: `crates/kamn-core/tests/release_gonogo_checklist_docs.rs` = `25` LOC.
- Largest extracted file in the staged write set: `runtime_reconciliation_contract_tests.rs` = `171` LOC.
- Touched-Rust result: `policy_decision=GO`.

# Deviations

- No behavioral deviations from the original docs-contract surface.
- The regression-governance block required a second-level split under `regression_governance_launch_contract_tests/` to keep every touched file within the active 200 LOC cap.
