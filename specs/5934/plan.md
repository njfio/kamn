# Plan: Issue #5934 - Task: Reduce shell/python surface below policy ceiling and improve governance ratio

- Issue: #5934
- Spec: `specs/5934/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5934/spec.md.
2. Implement: Retire/merge redundant scripts, migrate high-value script logic to Rust lanes, and enforce ratio gates.
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Initial)
- `scripts/ci/generate_combined_shell_surface_trend_report.sh`
- `scripts/ci/check_combined_shell_surface_trend_policy.sh`
- `scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
- `scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- `scripts/ci/ignored_test_and_script_budget_trend_contract_lane_impl.sh`
- `fixtures/ci/ignored_test_inventory_baseline.json`
- `fixtures/ci/ignored_test_inventory_metadata.json`

## Delivery Notes
1. Implemented governance structural-coupling extraction from review markers and policy-file parsing in combined shell-surface trend generation.
2. Added fail-closed governance validations and deterministic WARN/NO-GO reason projection in combined trend policy evaluation.
3. Refreshed ignored-test baseline fixtures and contract-lane fixtures to keep CI tool regression lanes converged with current repository state.

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5934/spec.md`.
- Upstream issue contract: GitHub issue #5934.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
