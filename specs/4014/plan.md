# Issue #4014 Plan

- Issue: #4014
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Approach
1. Add `scripts/ci/check_sqlite_crash_recovery_ci_dry_run_governance.py` to validate:
   - sqlite crash-recovery dry-run report contract parity (summary/policy/contract-lane),
   - baseline threshold fixture contract,
   - ci-tools fast-mode selector and ci-fast-gate workflow run-mode exclusion,
   - strategy/ops docs marker parity and remediation-marker completeness.
2. Add `fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env` with deterministic schema/reason/budget/selector entries.
3. Add Rust contract tests:
   - `crates/kamn-core/tests/sqlite_crash_recovery_ci_dry_run_governance_contract.rs`
   - generate sqlite crash-recovery dry-run reports, invoke checker, assert pass/fail and reason determinism.
4. Update docs markers:
   - `docs/ci/strategy.md`
   - `docs/ops/configuration.md`
5. Wire checker contract test into CI tools coverage:
   - add `cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture` in fast/full blocks of `scripts/ci/test_ci_tools.sh`.
6. Run targeted verification (`fmt`, `clippy`, focused tests).

## Affected Files
- `specs/4014/spec.md`
- `specs/4014/plan.md`
- `specs/4014/tasks.md`
- `scripts/ci/check_sqlite_crash_recovery_ci_dry_run_governance.py`
- `fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env`
- `crates/kamn-core/tests/sqlite_crash_recovery_ci_dry_run_governance_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `scripts/ci/test_ci_tools.sh`

## Risks and Mitigations
- Risk: checker brittleness to doc prose edits.
  - Mitigation: enforce deterministic marker keys and command strings only.
- Risk: selector false positives from broad matching.
  - Mitigation: enforce exact required/forbidden command entries in fast-mode block and workflow.
- Risk: shell-surface growth from governance additions.
  - Mitigation: isolate behavior to Python checker + Rust tests, minimal shell edits (single ci-tools insertion in fast/full blocks).

## Interface Contract
- Checker schema:
  - `kamn.ci.sqlite-crash-recovery-ci-dry-run-governance-report.v1`
- Checker reason taxonomy:
  - `kamn.ci.sqlite-crash-recovery-ci-dry-run-governance-reason-taxonomy.v1`
- Threshold fixture:
  - `fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env`

## ADR
- Not required (policy/checker/docs parity extension; no dependency/protocol changes).
