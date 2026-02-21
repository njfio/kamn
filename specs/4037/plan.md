# Issue #4037 Plan - SBOM/Provenance Release Go-No-Go Checker and Docs Parity Contracts

## Approach
1. Add RED Rust contract tests for checker baseline pass/fail-closed artifact and docs-parity scenarios.
2. Implement checker script that validates required artifact markers, docs parity markers, and deterministic reason taxonomy outputs.
3. Update strategy/ops docs with release checker marker and command contracts.
4. Add docs contract assertions in Rust test suites.
5. Run targeted tests plus fmt/clippy verification gates.

## Affected Modules
- `scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py` (new)
- `crates/kamn-core/tests/sbom_provenance_release_gonogo_checker_contract.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `specs/4037/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: drift between checker required marker set and docs markers.
  - Mitigation: define one deterministic marker list in checker constants and assert exact markers in docs tests.
- Risk: false-positive GO decisions with incomplete artifact payloads.
  - Mitigation: validate non-empty required marker fields and exact expected schema/taxonomy versions.
- Risk: runtime overhead growth in CI-smoke lane.
  - Mitigation: bounded `max-seconds` contract and performance test coverage.

## Interfaces / Contracts
- Checker report schema:
  - `kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1`
- Checker reason taxonomy:
  - `kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1`
- Checker reason codes CSV:
  - `sbom_provenance_artifact_marker_missing,sbom_provenance_artifact_marker_invalid,sbom_provenance_artifact_decision_not_go,sbom_provenance_docs_parity_marker_missing,sbom_provenance_runtime_budget_exceeded`
- Required artifact schema/value contracts:
  - `schema_version=kamn.runtime.sbom-provenance-artifact-report.v1`
  - `artifact_schema_version=kamn.runtime.sbom-provenance-artifact-schema.v1`
  - `fixture_schema_version=kamn.ci.sbom-provenance-artifact-fixture-matrix.v1`
  - `reason_taxonomy_version=kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1`
  - `release_manifest_required_artifact_id=sbom_provenance`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_sbom_provenance_release_gonogo_checker_contract_markers -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_sbom_provenance_release_gonogo_checker_markers -- --exact`
- GREEN:
  - rerun the above after checker/docs implementation
- VERIFY:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
