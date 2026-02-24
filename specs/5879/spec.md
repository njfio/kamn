# Spec: Issue #5879 - Runtime-Wide Production Panic-Path Audit + Env Fallback Remediation

- Issue: #5879
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The production panic-path checker defaults to a single crate root (`crates/kamn-node/src`), which can miss runtime violations in other production crates. When expanded to runtime crates, current code emits `production_unsafe_env_fallback_default` violations in CLI/MCP runtime entry paths due to `std::env::var(...).unwrap_or_else(...)` patterns.

## Scope
In scope:
- Expand default checker roots to runtime crates only:
  - `crates/kamn-core/src`
  - `crates/kamn-node/src`
  - `crates/kamn-cli/src`
  - `crates/kamn-mcp-server/src`
  - `crates/kamn-sdk/src`
  - `crates/kamn-kolme/src`
- Replace runtime env fallback `unwrap_or(_else)` patterns in targeted runtime files with explicit match-based fallback mapping.
- Preserve deterministic reason taxonomy/report fields and wrapper behavior.

Out of scope:
- Refactoring e2e harness crate env-fallback defaults.
- Repo-wide refactor of every env/default callsite.

## Acceptance Criteria
### AC-1 Runtime checker coverage is widened by default
Given `scripts/ci/check_no_production_expect.sh` is executed with no explicit roots,
When checker runs,
Then it scans all runtime crates listed in scope (and does not require caller-provided roots).

### AC-2 Runtime unsafe env fallbacks are remediated in targeted files
Given runtime fallback code in `kamn-cli` and `kamn-mcp-server`,
When checker scans runtime roots,
Then no `production_unsafe_env_fallback_default` violations are emitted from those files.

### AC-3 Checker contract/regression lanes stay green
Given checker wrapper + test harness,
When policy tests run,
Then deterministic outputs and fail-closed behavior remain intact.

## Conformance Cases
- C-01 (Conformance, AC-1): Wrapper baseline scan includes runtime roots and succeeds without caller `--root` flags.
- C-02 (Regression, AC-2): `kamn-cli` and `kamn-mcp-server` no longer emit unsafe fallback reason-code violations.
- C-03 (Functional, AC-3): `scripts/ci/test_check_no_production_expect.sh` passes.

## Success Metrics / Observable Signals
- `scripts/ci/check_no_production_expect.sh` returns `status=ok` on default execution.
- `scripts/ci/test_check_no_production_expect.sh` exits 0.
- Runtime checker output has `reason_codes_value=none`.
