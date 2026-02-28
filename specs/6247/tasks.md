# Issue 6247 Tasks

- T1 (Red/Baseline): Capture existing threshold baseline and measured current coverage for each target.
- T2 (Green/Config): Raise thresholds in `.ci/critical-path-coverage-thresholds.json` to defensible minima.
- T3 (Green/Docs): Add R59 follow-up threshold before/after rationale in `docs/planning/r59-followup.md`.
- T4 (Regression): Run `scripts/ci/test_check_critical_path_coverage.sh` to validate fail-closed behavior remains deterministic.
- T5 (Integration): Run `scripts/ci/run_critical_path_coverage_gate.sh` and confirm updated thresholds pass.
