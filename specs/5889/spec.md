# Spec: Issue #5889 - sdk_direct Unsafe Env Fallback Regression Remediation

- Issue: #5889
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`main` regressed after #5888 follow-up formatting: `scripts/ci/check_no_production_expect.sh` now reports 13 `production_unsafe_env_fallback_default` violations in `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`.

## Scope
In scope:
- Replace remaining `env::var(...).unwrap_or_else(...)` fallback-default callsites in `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` with explicit helper-based fallback resolution.
- Preserve existing env var names and default values.
- Re-verify panic-path audit and e2e harness tests.

Out of scope:
- Behavior changes to S-01..S-15 scenarios.
- Protocol/API changes.
- Additional checker taxonomy changes.

## Acceptance Criteria
### AC-1 Remaining sdk_direct unsafe env fallback defaults removed
Given `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`,
When scanning for direct `env::var(...).unwrap_or(_else)` fallback-default usage,
Then the 13 failing fallback-default paths are replaced by helper-based fallback resolution.

### AC-2 Default panic-path checker is green again
Given default checker roots,
When `scripts/ci/check_no_production_expect.sh` runs,
Then it exits successfully with `violation_count=0`.

### AC-3 e2e-harness scoped checker is green
Given `crates/kamn-e2e-harness/src`,
When `python3 scripts/ci/check_no_production_expect.py --root crates/kamn-e2e-harness/src` runs,
Then it exits successfully with `violation_count=0`.

### AC-4 e2e harness regression safety remains green
Given the sdk_direct fallback remediation,
When `cargo test -p kamn-e2e-harness` runs,
Then the test suite remains green.

## Conformance Cases
- C-01 (Functional, AC-1): targeted regex scan shows no remaining direct fallback-default patterns at the 13 prior sdk_direct callsites.
- C-02 (Conformance, AC-2): `scripts/ci/check_no_production_expect.sh` passes.
- C-03 (Conformance, AC-3): `python3 scripts/ci/check_no_production_expect.py --root crates/kamn-e2e-harness/src` passes.
- C-04 (Integration/Regression, AC-4): `cargo test -p kamn-e2e-harness` passes.

## Success Metrics / Observable Signals
- `runtime_panic_replacement_evidence_violation_count=0` on default checker invocation.
- `runtime_panic_replacement_evidence_violation_count=0` on e2e-harness-scoped checker invocation.
- No e2e harness test regressions.
