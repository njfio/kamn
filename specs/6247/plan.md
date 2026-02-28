# Issue 6247 Plan

## Approach
1. Capture baseline threshold values and current measured coverage.
2. Propose stricter minima per target with explicit rationale and headroom.
3. Write/extend tests for weakest targets before applying threshold ratchets.
4. Update threshold JSON and supporting docs.
5. Re-run coverage gate and targeted tests to verify deterministic behavior.

## Affected Modules
- `.ci/critical-path-coverage-thresholds.json`
- `scripts/ci/check_critical_path_coverage.py` (if policy diagnostics need updates only)
- `scripts/ci/run_critical_path_coverage_gate.sh` (if invocation metadata needs updates only)
- Tests in `crates/kamn-node` and `crates/kamn-core` for weak targets
- `docs/planning/r59-followup.md`
- `specs/6247/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: thresholds exceed realistic near-term coverage and cause persistent red CI.
  - Mitigation: use measured headroom and stage increases while still enforcing meaningful jumps.
- Risk: gate logic drifts during threshold work.
  - Mitigation: keep policy script behavior contract tests in regression lane.
- Risk: thresholds rise without real behavioral tests.
  - Mitigation: require new tests tied to previously uncovered branches before ratchet changes merge.

## Interfaces
- CI policy data contract in `.ci/critical-path-coverage-thresholds.json`.
- No runtime API or protocol interface changes.
