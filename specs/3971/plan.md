# Plan — Issue #3971

## Approach

1. Add `scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh`:
   - run key non-Kolme wrapper matrix tests,
   - run dispatcher unknown-wrapper probe,
   - assert fallback taxonomy/codes markers,
   - emit pass marker.
2. Add harness invocation to `scripts/ci/test_ci_tools.sh` (fast mode + full mode paths).
3. Update `docs/ci/strategy.md` with new harness command and fallback marker contract.
4. Update `scripts/ci/test_ci_strategy_contract.sh` marker list for new harness/marker strings.
5. Run targeted harness/strategy tests and fast CI tools regression.

## Affected Paths

- `scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_strategy_contract.sh`
- `docs/ci/strategy.md`
- `specs/3971/spec.md`
- `specs/3971/plan.md`
- `specs/3971/tasks.md`

## Risks / Mitigations

- Risk: duplicate matrix execution increases fast-lane runtime.
  Mitigation: harness reuses existing low-cost matrix scripts and adds only one unknown-wrapper probe.

- Risk: marker-string drift between harness and docs tests.
  Mitigation: update harness/docs/strategy-contract in one change.

## ADR

- Not required (CI harness and contract documentation update).
