# Spec: Issue #5947 - Task: Restore service_api_endpoint root line-budget contract by removing redundant delegates

- Issue: #5947
- Status: Implemented
- Type: task
- Priority: P2
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5917

## Problem Statement
`crates/kamn-node/src/service_api_endpoint.rs` regressed above its extraction contract line budget (`935 > 900`) because root-level wrappers were reintroduced for functions already implemented in `auth`, `payload`, and `websocket` submodules.

## Scope
In scope:
- Remove redundant root delegate wrappers from `service_api_endpoint.rs`.
- Update submodule call sites to reference `auth::`, `payload::`, and `websocket::` directly.
- Preserve runtime behavior and response/auth contracts.

Out of scope:
- New route behavior, protocol changes, or auth policy changes.
- Additional module extraction beyond wrapper cleanup.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: `service_api_endpoint.rs` root returns to the enforced extraction budget (`<= 900` lines).
- AC-2: `service_api_endpoint_module_extraction_contract` passes after cleanup.
- AC-3: Existing service API route/auth/websocket behavior remains unchanged.
- AC-4: Touched `kamn-node` code compiles/tests cleanly for targeted conformance paths.

## Conformance Cases
- C-01 (Conformance, AC-1): `conformance_service_api_endpoint_root_stays_within_line_budget` passes with root line count `<= 900`.
- C-02 (Conformance, AC-2): `cargo test -p kamn-node --test service_api_endpoint_module_extraction_contract -- --nocapture` passes.
- C-03 (Functional, AC-3): `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact` remains green.
- C-04 (Regression, AC-3): `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers -- --exact` remains green.
- C-05 (Verify, AC-4): `cargo fmt --check` and strict clippy for `kamn-node` pass.

## Success Metrics / Observable Signals
- Root wrappers removed and all call sites route directly to owning submodules.
- Contract gate no longer fails on line budget.
- Targeted route and websocket regression tests remain green.
- Formatting and lint checks pass for touched crate.

## Required Test Categories
- Unit: N/A (no new pure helper logic introduced)
- Functional: route contract behavior remains stable
- Conformance: module extraction contract passes
- Integration: existing service API route tests pass
- Regression: websocket missing-upgrade-header rejection remains stable

## Dependencies
- #5215
- #5917
