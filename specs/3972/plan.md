# Plan — Issue #3972

## Approach

1. Add red tests for decline-window warn/fail and threshold staleness/invalid-date paths in `test_check_combined_shell_surface_trend_policy.sh`.
2. Extend `check_combined_shell_surface_trend_policy.sh`:
   - parse new threshold keys,
   - add optional `--today` override for deterministic tests,
   - evaluate non-declining window warn/fail gates,
   - evaluate threshold metadata staleness gate.
3. Update threshold fixture defaults to include new keys.
4. Update CI strategy docs with decline-window policy markers.
5. Run targeted contracts and fast CI tools regression lane.

## Affected Paths

- `scripts/ci/check_combined_shell_surface_trend_policy.sh`
- `scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- `fixtures/ci/combined_shell_surface_trend_thresholds.json`
- `docs/ci/strategy.md`
- `specs/3972/spec.md`
- `specs/3972/plan.md`
- `specs/3972/tasks.md`

## Risks / Mitigations

- Risk: Added window gates could unexpectedly NO-GO current baseline state.
  Mitigation: initialize threshold metadata as fresh and windows reasonable; test warn/fail via `--today` overrides.

- Risk: Contract tests brittle due reason csv exact-match string.
  Mitigation: update deterministic csv string and corresponding test assertions together.

## Interfaces / Contracts

- New reason codes:
  - `combined_shell_surface_decline_window_warn_exceeded`
  - `combined_shell_surface_decline_window_fail_exceeded`
  - `combined_shell_surface_threshold_file_stale`
  - `combined_shell_surface_threshold_date_invalid`
  - `combined_shell_surface_today_override_invalid`

## ADR

- Not required (CI policy evolution only; no protocol/dependency change).
