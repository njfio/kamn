# Issue #5241 Plan

## Summary
Reduce shell LOC from #4096 by replacing the long shell regression test body with a Python test implementation while keeping a thin shell wrapper for compatibility.

## Affected Areas
- `scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- `scripts/ci/test_check_daemon_os_signal_stress_policy.py` (new)
- `specs/5241/{spec.md,plan.md,tasks.md}`

## Approach
1. Move overload checker regression test logic into a Python test runner script.
2. Replace shell test script with a thin wrapper that executes the Python test.
3. Preserve command surface so `scripts/ci/test_ci_tools.sh` and command-surface contract tests remain unchanged.
4. Run targeted regression + shell ratio guard checks.

## Risks and Mitigations
- Risk: behavior drift while rewriting test harness.
  - Mitigation: keep identical pass/fail scenarios and assertion messages.
- Risk: contract scripts expecting shell-file internals.
  - Mitigation: preserve script path and executable wrapper behavior.

## Interfaces / Contracts
- Wrapper command remains: `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- Python implementation command: `python3 scripts/ci/test_check_daemon_os_signal_stress_policy.py`

## ADR
Not required (test-harness compaction only).
