# Objective

Extract the remaining residual root tests and helper surface from `crates/kamn-core/tests/ci_strategy_docs.rs` into bounded sibling modules so the root file becomes a thin shell while preserving the existing docs-parity assertions against `docs/ci/strategy.md`.

# Inputs/Outputs

Inputs:
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`

Outputs:
- bounded module tree under `crates/kamn-core/tests/ci_strategy_docs/`
- extraction contract covering the residual module layout and staged root budget
- reduced residual root `ci_strategy_docs.rs`

# Boundaries/Non-goals

- Do not change `docs/ci/strategy.md` or `docs/ops/configuration.md` semantics.
- Do not rewrite already-extracted `ci_strategy_docs` sibling modules.
- Do not broaden the issue beyond the remaining residual root tests and root-local helper surface.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted files exceed the active file-size budget
- `ci_strategy_docs` no longer compiles because module markers or helper visibility drift
- residual docs-contract assertions change or disappear during extraction
- touched-Rust ratchet fails on newly oversized touched functions or files

# Acceptance criteria

- [ ] the remaining residual root tests are removed from `crates/kamn-core/tests/ci_strategy_docs.rs` and replaced by bounded sibling modules
- [ ] root `crates/kamn-core/tests/ci_strategy_docs.rs` is reduced below the staged extraction cap enforced by the new contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` passes
- [ ] the residual-root extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs/**`
- `crates/kamn-core/tests/*extraction_contract*.rs`
- `specs/6771-split-ci-strategy-docs-residual-root-tail.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module / marker / budget diagnostics.
- Runtime test failures remain ordinary Rust test assertion failures; no silent fallback helpers.

# Test plan

1. Add a red extraction contract asserting the residual-root module layout and a staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the residual root tests into bounded sibling modules.
4. Run `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6771-touched-size.json`.
