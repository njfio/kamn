# Plan — Issue #4822

## Approach

- Add shared dispatcher layer:
  - `scripts/lib/exec_dispatch.sh` (shell entrypoint)
  - `scripts/lib/exec_dispatch.py` (registry-backed resolver/executor)
  - `scripts/lib/exec_registry.json` (wrapper metadata)
- Add RED/GREEN guard:
  - `scripts/lib/test_exec_dispatch_registry.sh` validates executable dispatcher + registry + symlink integrity.
- Bulk-migrate eligible tiny wrappers to symlinks resolving to the shared dispatcher.
- Update affected tests that asserted wrapper file text to instead assert:
  - wrapper is symlink
  - symlink resolves to shared dispatcher
  - registry entry exists with expected interpreter/target/args/passthrough.
- Re-run broad regression (`scripts/ci/test_ci_tools.sh`) and fix remaining stale assertions until green.

## Affected Modules

- `scripts/lib/*` dispatcher/registry and dispatcher contract test.
- Wrapper families across `scripts/{ci,runtime,sdk,frontend,compliance,...}` migrated to symlink form.
- CI/runtime/sdk/frontend/compliance contract tests updated for new wrapper contract.
- Fixture/budget logic:
  - `scripts/ci/check_non_kolme_wave_trend_test_loc_soft_budget.py`
  - `fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_baseline.json`

## Risks / Mitigations

- Risk: stale tests expect inline wrapper text and fail after symlink migration.
  Mitigation: convert tests to symlink+registry assertions and run full `test_ci_tools.sh`.
- Risk: LOC budgets inflate if symlink wrappers are counted via resolved target content.
  Mitigation: treat symlink wrapper LOC as `1` in budget checker; update baseline.
- Risk: latent deterministic failures exposed by long CI suite.
  Mitigation: isolate each failing lane test, patch deterministically, rerun targeted + full suites.

## Interfaces / Contracts

- Wrapper invocation compatibility is preserved by registry-targeted dispatch.
- Contract scripts continue emitting deterministic `key=value` status and reason markers.
- No protocol/wire changes introduced.

## ADR

- Not required for this subtask; no dependency/protocol architecture changes were introduced.
