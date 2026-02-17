# Plan — Issue #4186

## Approach

- Add `scripts/ci/check_upgrade_compatibility_ci_smoke_convergence.py` that verifies:
  - ci-tools fast-mode contains required low-cost upgrade compatibility smoke commands,
  - heavy replay command (`run_version_compatibility_replay_deep_lane.sh`) is absent from
    ci-tools fast mode and `ci-fast-gate` workflow,
  - strategy markers remain synchronized,
  - smoke runtime budget guard (`--max-seconds <= 120`) is fail-closed.
- Add shell contract tests in
  `scripts/ci/test_check_upgrade_compatibility_ci_smoke_convergence.sh`.
- Wire the new contract test into `scripts/ci/test_ci_tools.sh` fast/full sections.
- Add strategy marker declarations in `docs/ci/strategy.md`.

## Affected Modules

- `scripts/ci/check_upgrade_compatibility_ci_smoke_convergence.py` (new)
- `scripts/ci/test_check_upgrade_compatibility_ci_smoke_convergence.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: marker or command drift causes false positives.
  - Mitigation: deterministic marker list and reason ordering in checker.
- Risk: heavy replay command leaks via future edits.
  - Mitigation: explicit fast-mode and workflow leakage checks with fail-closed reasons.
- Risk: added fast-mode smoke commands increase CI cost.
  - Mitigation: smoke-only command set (evidence + policy tests), enforce 120-second budget marker.

## Interfaces/Contracts

- Checker CLI:
  - `--workflow-file <path>`
  - `--ci-tools-file <path>`
  - `--strategy-doc <path>`
  - `--max-seconds <int>`
  - `--output-json <path>` (optional)
- Output markers:
  - `status=pass|fail`
  - `final_decision=GO|NO-GO`
  - `reason_taxonomy_version=kamn.ci.upgrade-compatibility-ci-smoke-convergence-reason-taxonomy.v1`
  - `reason_codes_csv=<deterministic ordered csv>`
  - `reason_codes_value=none|<csv>`
  - `upgrade_compatibility_ci_smoke_convergence_status=verified|violation`
  - `upgrade_compatibility_ci_smoke_max_seconds=120`
  - `upgrade_compatibility_local_heavy_max_seconds=900`
  - `upgrade_compatibility_ci_smoke_lane_cost_profile=low`
  - `upgrade_compatibility_local_heavy_execution_mode=opt_in`

## ADR

- Not required (bounded checker/test/docs update; no dependency/protocol/schema change).
