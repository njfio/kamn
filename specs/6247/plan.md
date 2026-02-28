# Issue 6247 Plan

Status: Reviewed

1. Establish prior baseline thresholds from `.ci/critical-path-coverage-thresholds.json`.
2. Use current measured coverage outputs (`ci-critical-path-coverage-policy.json`) to set defensible ratcheted minima with safety headroom.
3. Update `.ci/critical-path-coverage-thresholds.json` with the ratcheted values.
4. Add/update `docs/planning/r59-followup.md` documenting old/new values and rationale.
5. Verify via:
   - `scripts/ci/check_critical_path_coverage.py` (deterministic policy gate), and
   - `scripts/ci/run_critical_path_coverage_gate.sh` (integration probe), and
   - `scripts/ci/test_check_critical_path_coverage.sh` (regression/fail-closed paths).

## Risks / Mitigations
- Risk: thresholds are raised too aggressively and flake on minor coverage variance.
  - Mitigation: keep margin below measured actuals for each target.
- Risk: doc/config drift breaks policy contracts.
  - Mitigation: add explicit old/new table in follow-up planning doc and run checker regression suite.

## Interfaces / Contracts
- Threshold source of truth: `.ci/critical-path-coverage-thresholds.json`
- Gate runner: `scripts/ci/run_critical_path_coverage_gate.sh`
- Policy checker: `scripts/ci/check_critical_path_coverage.py`
- Planning evidence: `docs/planning/r59-followup.md`
