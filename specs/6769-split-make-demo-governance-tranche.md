# Objective

Extract the remaining oversized `doc_contains_make_and_demo_scope_contract_rules()` block from `crates/kamn-core/tests/ci_strategy_docs.rs` into bounded sibling modules while preserving the existing docs-parity assertions against `docs/ci/strategy.md`.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`

Outputs:
- bounded module tree under `crates/kamn-core/tests/ci_strategy_docs/`
- extraction contract covering the new layout and staged root budget
- reduced root `ci_strategy_docs.rs`

# Boundaries/Non-goals

- Do not change `docs/ci/strategy.md` content or semantics.
- Do not rewrite already-extracted `ci_strategy_docs` sibling modules.
- Do not broaden the issue beyond the remaining `doc_contains_make_and_demo_scope_contract_rules()` block.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- `ci_strategy_docs` no longer compiles because module markers or helper visibility drift
- docs-contract assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [ ] `doc_contains_make_and_demo_scope_contract_rules()` is removed from the root file and replaced by bounded sibling modules
- [ ] root `crates/kamn-core/tests/ci_strategy_docs.rs` is reduced below the staged extraction cap enforced by the new contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6769-split-make-demo-governance-tranche.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Runtime test failures remain ordinary Rust test assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the new module layout and a staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the block into bounded sibling modules.
4. Run `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6769-touched-size.json`.
