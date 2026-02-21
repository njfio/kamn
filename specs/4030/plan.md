# Issue #4030 Plan - Advisory Parser and Dependency-Threshold Fixture Contracts

## Approach
1. Add CI fixture matrix under `fixtures/ci/` with canonical schema/taxonomy markers, threshold
   metadata, and advisory severity rows.
2. Add parser/helper contract tests in `kamn-core` for metadata parsing, row parsing, severity
   normalization, and threshold mapping behavior.
3. Add CI strategy contract markers + docs parity assertions for the fixture schema/path and guard
   commands.
4. Run RED -> GREEN verification for parser/docs contracts and capture bounded performance evidence.

## Affected Modules
- `fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt`
- `crates/kamn-core/tests/dependency_ci_smoke_advisory_fixture_parser_contract.rs`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4030/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: fixture/doc marker drift.
  - Mitigation: docs-parity assertions for each required strategy marker.
- Risk: severity normalization ambiguity.
  - Mitigation: explicit severity ordering and deterministic unknown-severity fail-closed reason.
- Risk: overlap with checker wiring in `#4031`.
  - Mitigation: keep this issue scoped to fixture + parser helper contracts only.

## Interfaces / Contracts
- Fixture metadata keys are canonical and fail closed on unknown keys.
- Fixture row columns are fixed:
  `case_id|package|advisory_id|severity|expected_status|expected_reason_code`.
- Threshold contract key:
  `dependency_ci_smoke_threshold_max_severity`.
- Deterministic mapping helper contract:
  - unknown severity -> `fail|dependency_advisory_severity_unknown`
  - severity rank above threshold -> `fail|dependency_advisory_threshold_exceeded`
  - otherwise -> `pass|none`

## Validation Strategy
- RED: run docs-contract test asserting new strategy markers before doc updates.
- GREEN: add fixture + parser contracts + docs markers and rerun targeted tests.
- VERIFY: run `cargo fmt --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, and
  targeted parser/docs tests.
