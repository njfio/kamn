# Spec: Issue #5928 - Task: Bound replay guard memory with TTL/capacity eviction

- Issue: #5928
- Status: Implemented
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5917

## Problem Statement
Replay guard currently grows monotonically with process lifetime.

## Scope
In scope:
- Implement bounded replay cache with TTL and capacity eviction policy.

Out of scope:
- Changes to external auth semantics outside replay-window policy contract.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Replay structure remains memory-bounded under sustained traffic.
- AC-2: Replay attacks within configured window are still rejected.
- AC-3: Load tests verify no unbounded growth and acceptable eviction behavior.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): `service_api_endpoint::auth::tests::regression_replay_guard_capacity_eviction_bounds_memory_and_releases_oldest_nonce` verifies capacity-bounded eviction.
- C-02 (Unit, AC-2): `service_api_endpoint::auth::tests::regression_replay_guard_ttl_eviction_rejects_only_within_active_window` verifies replay rejection within TTL and acceptance after eviction.
- C-03 (Functional, AC-2): `main_tests::service_api_endpoint_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender` verifies replay requests fail closed.
- C-04 (Integration, AC-2): `main_tests::service_api_endpoint_tests::integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement` verifies bounded replay guard coexists with anti-spam middleware.
- C-05 (Regression, AC-2): `main_tests::service_api_endpoint_tests::regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable` verifies reason-code ordering remains deterministic.
- C-06 (Verify, AC-4): `cargo fmt --check` and strict node clippy pass for touched modules.

## Success Metrics / Observable Signals
- Replay nonce tracking is bounded by configured defaults (`DEFAULT_SERVICE_API_REPLAY_GUARD_MAX_ENTRIES`, `DEFAULT_SERVICE_API_REPLAY_GUARD_TTL_SECS`).
- Replay rejection behavior remains fail-closed for active window duplicates.
- Scoped unit, functional, integration, regression, formatting, and clippy checks pass.


## Required Test Categories
- Unit: eviction policy and keying behavior
- Functional: replay acceptance/rejection within window
- Integration: service API auth chain with bounded cache
- Regression: unbounded BTreeSet path removed
- Performance: memory and latency under sustained load

## Dependencies
- #5917
