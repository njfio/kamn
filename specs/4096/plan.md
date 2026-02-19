# Issue #4096 Plan

## Summary
Implement a fail-closed CI dry-run policy checker for daemon overload stress reports, backed by baseline thresholds and selector command-surface guards.

## Affected Areas
- `scripts/ci/check_daemon_os_signal_stress_policy.sh` (new)
- `scripts/ci/test_check_daemon_os_signal_stress_policy.sh` (new)
- `fixtures/ci/daemon_os_signal_stress_policy_thresholds.env` (new)
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4096/{spec.md,plan.md,tasks.md}`

## Approach
1. Add threshold fixture with schema/runtime/selector guard expectations.
2. Implement checker that validates:
   - report schema + expected final decision,
   - runtime budget + threshold values,
   - reason-code membership,
   - CI fast-mode command-surface guard (test entry required, heavy run entry forbidden).
3. Add shell test script covering pass and deterministic fail-closed cases.
4. Wire shell test into CI tools regression script.
5. Document marker contract in `docs/ci/strategy.md` and enforce via Rust docs-contract test assertions.
6. Execute red/green/regression command set.

## Risks and Mitigations
- Risk: checker overly strict and brittle to benign marker shifts.
  - Mitigation: use fixture-based thresholds and deterministic reason taxonomy with explicit updates.
- Risk: shell-surface growth.
  - Mitigation: keep implementation to one checker + one test script and no extra wrappers.
- Risk: selector guard false positives.
  - Mitigation: check exact command-line entry patterns rather than broad substring bans.

## Interfaces / Contracts
- Checker CLI:
  - `--report-file <path>`
  - `--threshold-file <path>`
  - `--ci-tools-script <path>`
  - `--expected-final-decision <GO|NO-GO>`
  - `--output-json <path>`
- Threshold fixture keys:
  - `REPORT_SCHEMA_VERSION`
  - `MAX_RUNTIME_SECONDS`
  - `ALLOWED_REASON_CODES_CSV`
  - `CI_TOOLS_REQUIRED_ENTRY`
  - `CI_TOOLS_FORBIDDEN_ENTRY`

## ADR
Not required (policy/checker/docs contract scope only).
