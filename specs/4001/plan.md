# Issue #4001 Plan

## Approach
1. Extend `fixtures/ci/performance_hot_path_fixture_matrix.json` with deterministic provenance metadata and drift-threshold seed fields.
2. Add RED assertions in performance fixture generator/checker tests for required metadata/seed markers.
3. Update `generate_performance_smoke_report.sh` to emit baseline provenance/seed markers in report output.
4. Update `check_performance_thresholds.sh` to fail closed when required baseline markers are missing.
5. Update `docs/ci/strategy.md` with baseline refresh/metadata contract markers and add docs contract tests.

## Affected Modules
- `fixtures/ci/performance_hot_path_fixture_matrix.json`
- `scripts/ci/generate_performance_smoke_report.sh`
- `scripts/ci/check_performance_thresholds.sh`
- `scripts/ci/test_generate_performance_smoke_report.sh`
- `scripts/ci/test_check_performance_thresholds.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4001/{spec,plan,tasks}.md`

## Risks and Mitigations
- Risk: Metadata schema drift breaks existing consumers.
  - Mitigation: additive schema extension + fail-closed validation with deterministic errors.
- Risk: Checker behavior changes unexpectedly for legacy reports.
  - Mitigation: targeted regression tests for missing/malformed marker scenarios.
- Risk: Docs and enforcement drift.
  - Mitigation: docs contract test assertions for new baseline marker section.

## Interfaces and Contracts
- Generated performance report contract extends with baseline provenance + seed fields.
- Threshold checker contract requires those fields and fails closed if absent.
- CI strategy doc contract includes explicit baseline metadata/seed marker policy.
