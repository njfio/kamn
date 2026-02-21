# Issue #4081 Plan — Lifecycle Artifact Integrity Generator and Verification Helpers

## Approach
1. Add runtime wrappers:
   - `scripts/runtime/generate_lifecycle_artifact_integrity_evidence_bundle.sh`
   - `scripts/runtime/check_lifecycle_artifact_integrity_evidence_bundle.sh`
2. Implement contract module `scripts/runtime/lifecycle_artifact_integrity_contract.py` with:
   - `generate` subcommand for deterministic lifecycle artifact payload + hash/provenance markers,
   - `check` subcommand for fail-closed recomputation and marker/tamper validation.
3. Register new wrapper commands in `scripts/lib/exec_registry.json`.
4. Add Rust contract tests in
   `crates/kamn-core/tests/lifecycle_artifact_integrity_contract.rs`.
5. Update `docs/ops/configuration.md` with lifecycle integrity marker schema + validation command markers.
6. Add docs marker assertions in
   `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.

## Affected Modules
- `scripts/runtime/generate_lifecycle_artifact_integrity_evidence_bundle.sh`
- `scripts/runtime/check_lifecycle_artifact_integrity_evidence_bundle.sh`
- `scripts/runtime/lifecycle_artifact_integrity_contract.py`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/lifecycle_artifact_integrity_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4081/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: Non-deterministic hash/provenance fields due unstable serialization.
  - Mitigation: Canonical field ordering + deterministic digest derivation from stable marker tuple.
- Risk: Drift between checker reason taxonomy and docs marker block.
  - Mitigation: Add ops-doc marker assertions in `service_api_ops_configuration_docs`.
- Risk: Runtime checker cost increases CI latency.
  - Mitigation: Keep implementation as file I/O + hash recomputation only; enforce performance test budget.

## Interfaces / Contracts
- Lifecycle artifact schema marker:
  `kamn.runtime.lifecycle-artifact-integrity-evidence.v1`
- Lifecycle artifact reason taxonomy marker:
  `kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v1`
- Deterministic reason codes marker:
  `lifecycle_artifact_required_field_missing,lifecycle_artifact_marker_mismatch,lifecycle_artifact_hash_mismatch,lifecycle_artifact_reason_taxonomy_mismatch,lifecycle_artifact_reason_codes_csv_mismatch,lifecycle_artifact_expected_decision_mismatch`

## Validation Strategy
- RED: add failing Rust contract/docs marker tests for missing lifecycle integrity markers and commands.
- GREEN: implement generator/checker wrappers, registry entries, and ops doc marker block.
- VERIFY: targeted contract tests + formatter/lint gates + CI fast-gate run.
