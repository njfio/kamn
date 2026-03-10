# 6755-split-ci-strategy-docs-fairness-deletion

## Objective
Extract the fairness/deletion docs-parity tranche from `crates/kamn-core/tests/ci_strategy_docs.rs` into bounded sibling modules while preserving the existing docs-contract assertions and real test entrypoint coverage.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
  - `docs/ci/strategy.md`
  - `docs/ops/configuration.md`
  - fairness/deletion fixtures and policy sources already referenced by the current tests
- Outputs:
  - root `ci_strategy_docs.rs` with the fairness/deletion tranche removed
  - bounded sibling modules for fairness and deletion docs parity
  - extraction contract for the tranche layout

## Boundaries/Non-goals
- Do not split unrelated `ci_strategy_docs` sections in this issue
- Do not change fairness or deletion marker semantics
- Do not modify runtime or workflow behavior

## Failure modes
- moved fairness/deletion test bodies remain inline in the root file
- extracted files exceed the size policy
- moved functions exceed the touched-function policy
- module wiring breaks the `ci_strategy_docs` target
- touched-Rust size policy remains NO-GO

## Acceptance criteria
- [ ] fairness/deletion tests are removed from the root `ci_strategy_docs.rs`
- [ ] bounded sibling files exist under `crates/kamn-core/tests/ci_strategy_docs/`
- [ ] extraction contract fails on the old layout and passes on the new layout
- [ ] `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` passes
- [ ] touched-Rust size policy returns `GO`
- [ ] spec records final evidence and deviations

## Files to touch
- `specs/6755-split-ci-strategy-docs-fairness-deletion.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs/*.rs`
- `crates/kamn-core/tests/ci_strategy_docs_fairness_deletion_extraction_contract.rs`

## Error semantics
- extraction assertions fail loudly on missing files, missing root markers, or stale inline test bodies
- existing docs-contract assertions remain authoritative and unchanged in meaning
- no silent fallback module wiring

## Test plan
1. Add a red extraction contract for the fairness/deletion tranche
2. Confirm it fails against current `main`
3. Extract fairness and deletion into bounded sibling files, using small helpers as needed
4. Run:
   - `cargo test -p kamn-core --test ci_strategy_docs_fairness_deletion_extraction_contract -- --nocapture`
   - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6755-touched-size.json`
5. Record evidence and open the PR

## Phase 6 Evidence
- `cargo test -p kamn-core --test ci_strategy_docs_fairness_deletion_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6755-touched-size-refactor.json`
- Result: `policy_decision=GO`

## Deviations
- None.
