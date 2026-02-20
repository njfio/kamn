# Issue #4043 Plan

- Issue: #4043
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Approach
1. Add `scripts/runtime/api_compatibility_matrix_local_heavy_live_contract.py` with:
   - `run-lane`: deterministic matrix artifact projection over compatibility change classes.
   - `check-policy`: fail-closed report contract validation.
   - `run-contract-lane`: lane + checker + deterministic tamper rejection + ops docs markers.
2. Add matrix fixture metadata file under `fixtures/runtime/` to define supported compatibility rows and expected class-level outcomes.
3. Wire `exec_dispatch` wrappers + registry entries:
   - `validate_api_compatibility_matrix_local_heavy_live.sh`
   - `check_api_compatibility_matrix_local_heavy_live_policy.sh`
   - `validate_api_compatibility_matrix_local_heavy_live_contract_lane.sh`
4. Add Rust contract tests in `crates/kamn-core/tests/api_compatibility_matrix_local_heavy_contract.rs`.
5. Update ops docs + docs-contract assertions:
   - `docs/ops/configuration.md`
   - `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
6. Run targeted `fmt`, `clippy`, and cargo tests.

## Affected Files
- `specs/4043/spec.md`
- `specs/4043/plan.md`
- `specs/4043/tasks.md`
- `fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt`
- `scripts/runtime/api_compatibility_matrix_local_heavy_live_contract.py`
- `scripts/runtime/validate_api_compatibility_matrix_local_heavy_live.sh`
- `scripts/runtime/check_api_compatibility_matrix_local_heavy_live_policy.sh`
- `scripts/runtime/validate_api_compatibility_matrix_local_heavy_live_contract_lane.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/api_compatibility_matrix_local_heavy_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations
- Risk: reason-code and class taxonomy drift between runner/policy/fixture/docs.
  - Mitigation: policy checker validates taxonomy + fixture rows deterministically; docs-contract assertions guard ops markers.
- Risk: shell-surface growth under ratio guardrails.
  - Mitigation: use dispatch symlinks only; avoid new shell script bodies; track mitigation via #5310.
- Risk: flaky local-heavy timing assertions.
  - Mitigation: dry-run performance assertion with conservative threshold.

## Interface Contract
- Lane schema: `kamn.runtime.api-compatibility-matrix-local-heavy-live-report.v1`
- Policy schema: `kamn.runtime.api-compatibility-matrix-local-heavy-live-policy-report.v1`
- Contract-lane schema: `kamn.runtime.api-compatibility-matrix-local-heavy-live-contract-lane-report.v1`
- Fixture schema: `kamn.runtime.api-compatibility-matrix-local-heavy-fixture-matrix.v1`
- Artifact schema: `kamn.runtime.api-compatibility-matrix-local-heavy-artifact-schema.v1`
- Reason taxonomy marker:
  - `api_compatibility_matrix_local_heavy_reason_taxonomy_version=kamn.runtime.api-compatibility-matrix-local-heavy-policy-reason-taxonomy.v1`

## ADR
- Not required (no dependency, protocol, or wire-format changes).
