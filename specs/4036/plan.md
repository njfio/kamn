# Issue #4036 Plan - Deterministic SBOM/Provenance Artifact Generator and Schema Validation

## Approach
1. Add RED Rust contract tests for SBOM/provenance generator behavior (baseline, drift fail-closed, run-mode opt-in boundary, regression, performance).
2. Implement a fixture-driven Python generator contract script that emits deterministic schema/taxonomy markers and JSON output.
3. Add fixture matrix defining profile expectations and schema/taxonomy markers.
4. Update strategy + ops docs with marker/command contracts and add docs-contract assertions.
5. Run targeted tests plus fmt/clippy verification gates.

## Affected Modules
- `scripts/deploy/sbom_provenance_artifact_generator_contract.py` (new)
- `fixtures/ci/sbom_provenance_artifact_fixture_matrix.txt` (new)
- `crates/kamn-core/tests/sbom_provenance_artifact_generator_contract.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `specs/4036/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker drift between fixture/script/docs.
  - Mitigation: encode deterministic constants in script + assert exact markers in Rust docs tests.
- Risk: profile evaluation non-determinism.
  - Mitigation: fixture-driven deterministic rows and fixed profile thresholds.
- Risk: CI budget regressions from new generator lane.
  - Mitigation: bounded max-seconds marker and performance test (<5s smoke budget).

## Interfaces / Contracts
- Run report schema:
  - `kamn.runtime.sbom-provenance-artifact-report.v1`
- Artifact schema:
  - `kamn.runtime.sbom-provenance-artifact-schema.v1`
- Fixture schema:
  - `kamn.ci.sbom-provenance-artifact-fixture-matrix.v1`
- Reason taxonomy:
  - `kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1`
- Reason codes CSV:
  - `sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test sbom_provenance_artifact_generator_contract -- --nocapture`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_sbom_provenance_artifact_generator_contract_markers -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_sbom_provenance_artifact_generator_markers -- --exact`
- GREEN:
  - rerun above after implementation/docs updates
- VERIFY:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
