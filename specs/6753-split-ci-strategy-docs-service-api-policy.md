# 6753-split-ci-strategy-docs-service-api-policy

## Objective
Extract the service-API policy tranche from `crates/kamn-core/tests/ci_strategy_docs.rs` into bounded sibling modules while preserving all existing docs-contract assertions and test entrypoints.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
  - `docs/ci/strategy.md`
  - `docs/ops/configuration.md`
  - referenced service-API policy sources and fixtures already consumed by the docs-contract tests
- Outputs:
  - thin root `ci_strategy_docs.rs` with the moved service-API tests removed
  - bounded sibling modules for:
    - request-path authz
    - scope policy
    - tenant isolation
    - API version policy
    - request/response schema compatibility
  - extraction contract for the tranche layout

## Boundaries/Non-goals
- Do not split unrelated `ci_strategy_docs` sections in this issue
- Do not change docs marker semantics, reason-code taxonomies, or fixture expectations
- Do not modify runtime/workflow behavior outside the docs-contract test surface

## Failure modes
- root file still contains moved service-API test bodies
- extracted files exceed size policy
- touched test helpers still exceed function-size policy
- import/module wiring breaks the `ci_strategy_docs` test target
- touched-Rust size policy remains NO-GO

## Acceptance criteria
- [ ] the service-API tranche is removed from the root `ci_strategy_docs.rs`
- [ ] bounded sibling files exist under `crates/kamn-core/tests/ci_strategy_docs/`
- [ ] extraction contract fails on the old layout and passes on the new layout
- [ ] `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test ci_strategy_docs_service_api_policy_extraction_contract -- --nocapture` passes
- [ ] touched-Rust size policy returns `GO`
- [ ] spec records final evidence and deviations

## Files to touch
- `specs/6753-split-ci-strategy-docs-service-api-policy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs/*.rs`
- `crates/kamn-core/tests/ci_strategy_docs_service_api_policy_extraction_contract.rs`

## Error semantics
- extraction assertions fail loudly on missing module files, missing root markers, or stale inline test bodies
- existing docs-contract assertions remain authoritative and unchanged in meaning
- no silent fallback module inclusion

## Test plan
1. Add a red extraction contract for the five service-API policy families
2. Confirm it fails against the current monolith
3. Extract the five families into bounded sibling files, using small local helpers where needed to satisfy the function-size ratchet
4. Run:
   - `cargo test -p kamn-core --test ci_strategy_docs_service_api_policy_extraction_contract -- --nocapture`
   - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6748 --base-ref origin/main --output-json /tmp/6753-touched-size.json`
5. Record final evidence and deviations, then open the PR
