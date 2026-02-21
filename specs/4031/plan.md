# Issue #4031 Plan - Dependency CI Smoke Checker and Docs Parity

## Approach
1. Add dependency CI smoke checker module in `kamn-core` with deterministic taxonomy markers,
   fail-closed decision types, and threshold evaluation logic.
2. Export checker APIs from `lib.rs`.
3. Add checker contract tests, including fixture-taxonomy superset regression against
   `fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt`.
4. Extend `docs/ci/strategy.md` with checker taxonomy + remediation markers and add docs parity
   assertions in `ci_strategy_docs.rs`.
5. Wire checker contract command into `scripts/ci/test_ci_tools.sh` fast/full command surfaces.

## Affected Modules
- `crates/kamn-core/src/dependency_ci_smoke_policy.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/dependency_ci_smoke_checker_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `scripts/ci/test_ci_tools.sh`
- `specs/4031/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: fixture/checker taxonomy drift.
  - Mitigation: fixture-superset regression test against fixture reason codes.
- Risk: docs parity drift for threshold/remediation markers.
  - Mitigation: explicit docs-contract tests for checker markers and remediation map coverage.
- Risk: CI command-surface drift.
  - Mitigation: wire checker contract test command into fast/full CI tools script lists.

## Interfaces / Contracts
- Checker taxonomy version:
  `kamn.ci.dependency-ci-smoke-reason-taxonomy.v1`
- Checker reason codes:
  `dependency_advisory_input_empty,dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded`
- Decision contract:
  - empty advisory input -> reject `dependency_advisory_input_empty`
  - unknown severity -> reject `dependency_advisory_severity_unknown`
  - severity rank above threshold -> reject `dependency_advisory_threshold_exceeded`
  - otherwise -> allow

## Validation Strategy
- RED: add docs-contract marker assertions before docs update.
- GREEN: implement checker/tests/docs/CI wiring and rerun targeted suites.
- VERIFY: run `cargo fmt --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, targeted
  tests, and CI-tool script checks.
