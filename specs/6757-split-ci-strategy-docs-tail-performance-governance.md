# 6757-split-ci-strategy-docs-tail-performance-governance

## Objective
Extract the tail performance/governance tranche from `crates/kamn-core/tests/ci_strategy_docs.rs` into bounded sibling modules while preserving the existing docs-contract assertions and the real `ci_strategy_docs` test entrypoint.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
  - `docs/ci/strategy.md`
  - referenced performance/governance docs and fixtures already consumed by the current tests
- Outputs:
  - root `ci_strategy_docs.rs` with the tail tranche removed
  - bounded sibling modules for performance baseline, performance CI smoke, governance feature-commit ratio, and review document freeze tests
  - extraction contract for the tail layout

## Boundaries/Non-goals
- Do not split unrelated `ci_strategy_docs` sections in this issue
- Do not change performance/governance marker semantics
- Do not modify runtime or CI workflow behavior

## Failure modes
- moved tail test bodies remain inline in the root file
- extracted files exceed the size policy
- moved functions exceed the touched-function policy
- module wiring breaks the `ci_strategy_docs` target
- touched-Rust size policy remains NO-GO

## Acceptance criteria
- [ ] tail performance/governance tests are removed from root `ci_strategy_docs.rs`
- [ ] bounded sibling files exist under `crates/kamn-core/tests/ci_strategy_docs/`
- [ ] extraction contract fails on the old layout and passes on the new layout
- [ ] `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` passes
- [ ] touched-Rust size policy returns `GO`
- [ ] spec records final evidence and deviations

## Files to touch
- `specs/6757-split-ci-strategy-docs-tail-performance-governance.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs/*.rs`
- `crates/kamn-core/tests/ci_strategy_docs_tail_performance_governance_extraction_contract.rs`

## Error semantics
- extraction assertions fail loudly on missing files, missing root markers, or stale inline test bodies
- existing docs-contract assertions remain authoritative and unchanged in meaning
- no silent fallback module wiring

## Test plan
1. Add a red extraction contract for the tail performance/governance tranche
2. Confirm it fails against current `main`
3. Extract the tail functions into bounded sibling files, using small helpers as needed
4. Run:
   - `cargo test -p kamn-core --test ci_strategy_docs_tail_performance_governance_extraction_contract -- --nocapture`
   - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6757-touched-size.json`
5. Record evidence and open the PR

## Phase 6 Evidence
- `cargo test -p kamn-core --test ci_strategy_docs_tail_performance_governance_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6757-touched-size-refactor.json`
- Result: `policy_decision=GO`

## Deviations
- None.
