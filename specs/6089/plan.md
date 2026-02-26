# Plan: Issue #6089

## Approach
1. Keep current `.sh` entrypoints as compatibility wrappers to preserve command surface.
2. Implement Python equivalents for the two large wave-wrapper harness implementations.
3. Wire wrappers to `exec python3 ...` so callers keep the same paths/flags.
4. Run representative wave trend/baseline harness checks plus runner-contract checks.
5. Capture shell LOC delta and closure markers.

## Affected Modules
- `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/test_wave_wrapper_family_budget_trend_impl.py`
- `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh`
- `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.py`
- `specs/6089/spec.md`
- `specs/6089/plan.md`
- `specs/6089/tasks.md`
- `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`

## Risks / Mitigations
- Risk: behavioral drift in contract assertions.
  Mitigation: run representative wave wrappers + runner-contract checks.
- Risk: command-surface drift.
  Mitigation: keep `.sh` entrypoints and only change implementation target.

## Interfaces / Contracts
- No entrypoint path changes for existing `.sh` wave-wrapper harness scripts.
- No protocol/wire/schema changes.
