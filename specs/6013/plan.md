# Plan: Issue #6013

## Approach
1. Reproduce failing production `expect()` surface gate on latest main-derived branch.
2. Refresh `fixtures/ci/production_expect_surface_baseline.env` with current measured file/count values.
3. Re-run `production_expect_surface_policy` and confirm GREEN.
4. Confirm threshold fixture remains unchanged.

## Affected Modules
- `fixtures/ci/production_expect_surface_baseline.env`

## Risks / Mitigations
- Risk: stale baseline if census is misread.
  Mitigation: use values emitted by failing test output and immediate rerun.
- Risk: accidental threshold weakening.
  Mitigation: do not edit `.ci/production_expect_surface_thresholds.env`.

## Interfaces / Contracts
- No runtime or API changes.
- CI contract fixture refresh only.
