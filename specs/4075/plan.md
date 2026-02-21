# Issue #4075 Plan — Retention Fixture Matrix and Parser Helper Contracts

## Approach
1. Add a retention fixture matrix under `fixtures/runtime/` with schema/taxonomy markers and
   valid+invalid rows.
2. Add a dedicated parser helper contract test file in `kamn-core` mirroring the existing
   fixture-contract pattern used by quota/fairness governance checks.
3. Add docs section + docs parity test assertions for the new retention markers.
4. Drive RED -> GREEN by running parser contract tests before and after docs/test wiring.

## Affected Modules
- `fixtures/runtime/retention_policy_fixture_matrix.txt`
- `crates/kamn-core/tests/retention_policy_fixture_parser_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4075/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker drift between fixture and docs.
  - Mitigation: dedicated docs parity assertions for every retention marker.
- Risk: parser behavior becoming non-deterministic with malformed lines.
  - Mitigation: explicit unit test for malformed column count and fixed error expectation.
- Risk: overlap with follow-up checker work (`#4076`).
  - Mitigation: keep this issue scoped to fixture+parser helper contracts only.

## Interfaces / Contracts
- Fixture metadata keys are canonical and fail closed on unknown keys.
- Fixture row columns are fixed:
  `case_id|domain|max_age_seconds|expected_status|expected_reason_code`.
- Decision helper contract is deterministic:
  - unknown domain -> `fail|retention_domain_unknown`
  - zero window -> `fail|retention_window_non_positive`
  - otherwise -> `pass|none`

## Validation Strategy
- RED: run new retention fixture parser contract test before docs section exists; docs-parity test
  should fail for missing retention markers.
- GREEN: add fixture/docs/parity assertions and rerun targeted tests.
- VERIFY: `cargo fmt --check`, `cargo clippy -- -D warnings`, and targeted test suite pass.
