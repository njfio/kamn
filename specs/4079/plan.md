# Issue #4079 Plan — Local-Heavy Redaction Validation Lane

## Approach
1. Add a runtime wrapper `scripts/runtime/run_local_heavy_redaction_validation_lane.sh` and wire it
   through `scripts/lib/exec_registry.json`.
2. Implement deterministic runner logic in
   `scripts/runtime/local_heavy_redaction_validation_lane_contract.py` with:
   - profiles: `baseline`, `injected-leak`;
   - mode controls: `dry-run|run`, explicit local opt-in required for run mode;
   - deterministic schema/taxonomy/reason markers and structured output JSON.
3. Add Rust contract tests in
   `crates/kamn-core/tests/local_heavy_redaction_validation_lane_contract.rs` for
   unit/functional/integration/regression/performance coverage.
4. Update `docs/ops/configuration.md` with deterministic redaction profile/schema markers and add a
   docs assertion in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
5. Execute RED -> GREEN with targeted tests, then verify via fmt/clippy.

## Affected Modules
- `scripts/runtime/run_local_heavy_redaction_validation_lane.sh`
- `scripts/runtime/local_heavy_redaction_validation_lane_contract.py`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/local_heavy_redaction_validation_lane_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4079/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: taxonomy markers drift between runner and docs.
  - Mitigation: assert schema/taxonomy/reason CSV in runner tests and docs test.
- Risk: run mode executes in CI-fast scope.
  - Mitigation: enforce `--ci-fast-gate FAIL` + explicit opt-in guard for run mode.
- Risk: unstable leak classification causes flaky tests.
  - Mitigation: use deterministic synthetic profile metrics and fixed leak markers.

## Interfaces / Contracts
- Runner schema marker:
  `kamn.runtime.local-heavy-redaction-validation-lane-report.v1`
- Artifact schema marker:
  `kamn.runtime.local-heavy-redaction-validation-artifact-schema.v1`
- Reason taxonomy marker:
  `kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1`
- Reason codes CSV:
  `local_heavy_redaction_sensitive_pattern_detected,local_heavy_redaction_runtime_budget_exceeded`
- Opt-in env guard:
  `KAMN_LOCAL_HEAVY_REDACTION_VALIDATION_OPT_IN=1`

## Validation Strategy
- RED: add unit docs/runner contract tests expecting runner/doc markers before implementation.
- GREEN: implement runner + docs markers and rerun targeted suites.
- VERIFY: run targeted lane/docs tests, `cargo fmt --check`, `cargo clippy -- -D warnings`.
