# Spec: Issue #5877 - Production Expect Checker Test-Only Path Hardening

- Issue: #5877
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The deterministic production panic-path checker currently flags `crates/kamn-node/src/service_api_endpoint/tests.rs` as production code and fails with `production_expect_reachable` violations. The file is test-only by intent, so current scan classification is too broad and produces false-positive `expect()` audit findings.

## Scope
In scope:
- Harden test-only path classification in `scripts/ci/check_no_production_expect.py` so `src/**/tests.rs` files are excluded from production scans.
- Add checker regression coverage proving `src/**/tests.rs` files are ignored while true production `.expect(` usage still fails.
- Validate deterministic checker output remains fail-closed for real production violations.

Out of scope:
- Repository-wide conversion of all `.expect(` callsites.
- Runtime API/protocol behavior changes.
- Governance/report-document content modifications.

## Acceptance Criteria
### AC-1 Test-only source files are excluded from production scan
Given Rust sources under `crates/kamn-node/src`,
When a file path matches `src/**/tests.rs`,
Then `check_no_production_expect` must not report violations from that file.

### AC-2 Real production expect violations still fail closed
Given a non-test production source file containing `.expect(`,
When checker tests run,
Then the checker returns `status=fail` with deterministic reason code `production_expect_reachable`.

### AC-3 Existing checker contract lane remains green
Given the repo checker lane,
When `scripts/ci/test_check_no_production_expect.sh` runs,
Then all existing assertions pass with new path-classification behavior.

## Conformance Cases
- C-01 (Conformance, AC-1): fixture rooted at `src/**/tests.rs` with `.expect(` yields no violations.
- C-02 (Regression, AC-2): fixture rooted at non-test source path with `.expect(` yields fail with `reason_codes_value=production_expect_reachable`.
- C-03 (Functional, AC-3): full script `scripts/ci/test_check_no_production_expect.sh` passes.

## Success Metrics / Observable Signals
- `scripts/ci/check_no_production_expect.sh` reports `status=ok` on workspace.
- `scripts/ci/test_check_no_production_expect.sh` exits 0.
- No false-positive violations reported for `crates/kamn-node/src/service_api_endpoint/tests.rs`.
