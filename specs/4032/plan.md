# Issue #4032 Plan - Local-Heavy Deep Dependency Scan Runner

## Approach
1. Add deterministic fixture matrix for local-heavy deep dependency scan profiles.
2. Add runner contract script under `scripts/runtime` with explicit mode/profile/ci-fast-gate/opt-in boundaries.
3. Register wrapper entry in `scripts/lib/exec_registry.json` and add wrapper symlink.
4. Add Rust contract tests for unit/functional/integration/regression/performance runner behavior.
5. Update `docs/ops/configuration.md` with runner markers and command contracts plus docs test assertions.

## Affected Modules
- `fixtures/ci/dependency_local_heavy_deep_scan_fixture_matrix.txt`
- `scripts/runtime/dependency_local_heavy_deep_scan_lane_contract.py`
- `scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/dependency_local_heavy_deep_scan_lane_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4032/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: profile marker drift between fixture, runner, and docs.
  - Mitigation: docs-contract assertions + fixture-driven runner tests.
- Risk: local-heavy run-mode leakage into ci-fast-gate behavior.
  - Mitigation: fail-closed run-mode guard requiring explicit opt-in and `--ci-fast-gate FAIL`.
- Risk: schema drift across follow-up checker work.
  - Mitigation: stable `schema_version`/`artifact_schema_version`/`reason_taxonomy_version` markers.

## Interfaces / Contracts
- Runner schema version:
  `kamn.runtime.dependency-local-heavy-deep-scan-lane-report.v1`
- Artifact schema version:
  `kamn.runtime.dependency-local-heavy-deep-scan-artifact-schema.v1`
- Fixture schema version:
  `kamn.ci.dependency-local-heavy-deep-scan-fixture-matrix.v1`
- Reason taxonomy version:
  `kamn.runtime.dependency-local-heavy-deep-scan-reason-taxonomy.v1`
- Reason codes:
  `dependency_local_heavy_deep_scan_profile_threshold_exceeded,dependency_local_heavy_deep_scan_runtime_budget_exceeded`

## Validation Strategy
- RED: add runner/docs-contract tests before runner/docs implementation.
- GREEN: implement fixture + runner + docs markers and rerun targeted tests.
- VERIFY: run fmt, targeted tests, and clippy for touched crate tests.
