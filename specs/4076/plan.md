# Issue #4076 Plan — Retention Checker Taxonomy and Docs Parity

## Approach
1. Add retention checker API surface in `retention_engine`:
   - reason taxonomy/version constants,
   - checker input/decision/reason types,
   - deterministic evaluation function.
2. Export checker APIs via `kamn_core` public surface.
3. Add checker contract test file mirroring quota checker contract style.
4. Add CI-strategy docs section + docs parity test assertions.
5. Execute RED->GREEN and verify with fmt/clippy/targeted tests.

## Affected Modules
- `crates/kamn-core/src/retention_engine.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/retention_policy_checker_contract.rs`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4076/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: taxonomy drift between fixture and checker reason sets.
  - Mitigation: explicit checker-superset regression test against fixture marker list.
- Risk: docs marker drift.
  - Mitigation: dedicated `ci_strategy_docs` parity assertions.
- Risk: checker semantics ambiguity at boundary (`age == window`).
  - Mitigation: integration test defines deterministic boundary (`<=` allow, `>` reject).

## Interfaces / Contracts
- Checker taxonomy version:
  `kamn.runtime.retention-policy-reason-taxonomy.v1`.
- Checker reason codes:
  `retention_domain_unknown,retention_window_non_positive,retention_record_expired`.
- Decision contract:
  - unknown domain -> reject `retention_domain_unknown`
  - zero window -> reject `retention_window_non_positive`
  - record_age_seconds > window_seconds -> reject `retention_record_expired`
  - else allow

## Validation Strategy
- RED: add docs-parity test for retention checker markers before docs section exists.
- GREEN: implement checker APIs/tests/docs and rerun targeted suites.
- VERIFY: `cargo fmt --check`, `cargo clippy -- -D warnings`, and targeted checker/docs tests.
