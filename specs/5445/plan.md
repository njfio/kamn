# Issue #5445 Plan - Rust Harness Migration for SBOM/Provenance Generator

## Approach
1. Update contract tests/docs tests first (RED) to expect Rust harness command surface.
2. Implement Rust binary under `crates/kamn-core/src/bin/` mirroring existing lane semantics.
3. Convert Python lane script into thin delegation shim to the Rust harness for compatibility.
4. Update strategy/ops docs command blocks to Rust harness invocation.
5. Run targeted tests and verification gates (`fmt`, `clippy`).

## Affected Modules
- `crates/kamn-core/src/bin/sbom_provenance_artifact_generator_contract.rs` (new)
- `scripts/deploy/sbom_provenance_artifact_generator_contract.py`
- `crates/kamn-core/tests/sbom_provenance_artifact_generator_contract.rs`
- `crates/kamn-core/tests/sbom_provenance_release_gonogo_checker_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `specs/5445/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: command-surface drift between docs and tests.
  - Mitigation: update docs and assert exact command string markers in docs tests.
- Risk: semantic drift during Python->Rust port.
  - Mitigation: preserve existing contract test assertions and fail-closed outputs.
- Risk: breaking existing script entry path.
  - Mitigation: keep script as compatibility shim delegating to Rust harness.

## Interfaces / Contracts
- Run report schema:
  - `kamn.runtime.sbom-provenance-artifact-report.v1`
- Artifact schema:
  - `kamn.runtime.sbom-provenance-artifact-schema.v1`
- Fixture schema:
  - `kamn.ci.sbom-provenance-artifact-fixture-matrix.v1`
- Reason taxonomy:
  - `kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1`
- Reason codes csv:
  - `sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test sbom_provenance_artifact_generator_contract -- --nocapture`
  - `cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_sbom_provenance_artifact_generator_contract_markers -- --exact`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_sbom_provenance_artifact_generator_markers -- --exact`
- GREEN:
  - rerun above after Rust harness + docs updates
- VERIFY:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
