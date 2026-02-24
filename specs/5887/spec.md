# Spec: Issue #5887 - E2E Driver Env Fallback Hardening + Audit Root Expansion

- Issue: #5887
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Production panic-path audits currently pass for runtime roots but miss e2e-driver env fallback usage. A full `crates/*/src` scan reports 148 unsafe env fallback defaults in e2e harness drivers, leaving an unresolved policy gap.

## Scope
In scope:
- Replace direct `env::var(...).unwrap_or_else(...)` fallback usage in:
  - `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
  - `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
  - `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- Add explicit match-based env helpers and route call sites through them.
- Expand default panic-path checker roots to include `crates/kamn-e2e-harness/src`.

Out of scope:
- Scenario behavioral changes.
- Protocol/API changes.

## Acceptance Criteria
### AC-1 E2E driver unsafe env fallback defaults removed
Given the three e2e driver source files,
When scanning for `env::var(...).unwrap_or(_else)` fallback usage,
Then no direct unsafe env fallback defaults remain.

### AC-2 Panic-path checker default roots include e2e harness
Given `scripts/ci/check_no_production_expect.py`,
When default roots are evaluated,
Then `crates/kamn-e2e-harness/src` is included.

### AC-3 Default panic-path audit passes
Given updated e2e drivers and checker roots,
When `scripts/ci/check_no_production_expect.sh` runs,
Then it exits successfully with zero violations.

### AC-4 All-roots panic-path audit passes for expect/panic/unreachable/fallback taxonomy
Given all crate src roots,
When `python3 scripts/ci/check_no_production_expect.py --root <...all src roots...>` runs,
Then it exits successfully with zero violations.

## Conformance Cases
- C-01 (Functional, AC-1): no direct `env::var(...).unwrap_or(_else)` fallback patterns in e2e driver files.
- C-02 (Functional, AC-2): checker default root list contains `crates/kamn-e2e-harness/src`.
- C-03 (Conformance, AC-3): `scripts/ci/check_no_production_expect.sh` passes with `violation_count=0`.
- C-04 (Integration, AC-4): all-roots checker invocation passes with `violation_count=0`.

## Success Metrics / Observable Signals
- Violation count for all-roots panic-path audit: `0`.
- Default panic-path audit remains green.
- No behavior drift in e2e harness driver compilation/tests.
