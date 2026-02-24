# Spec: Issue #5924 - Task: Replace kamn-core wipe_bytes loop with compiler-safe zeroization

- Issue: #5924
- Status: Implemented
- Type: task
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Manual byte loops may be optimized away and are not sufficient for secret erasure guarantees.

## Scope
In scope:
- Use zeroize or equivalent guaranteed erasure primitive in signature_profile key material paths.

Out of scope:
- Broad cryptographic API refactor unrelated to zeroization correctness.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: All sensitive buffers in kamn-core signing paths are zeroized using proven primitives.
- AC-2: Failure-path tests verify no secret leaks in error output.
- AC-3: Static checks prevent reintroduction of manual wipe loops in sensitive modules.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Unit, AC-1): `signature_profile::tests::regression_wipe_bytes_zeroizes_secret_material_buffer` verifies `wipe_bytes` erases in-memory secret buffers.
- C-02 (Functional, AC-2): `signature_profile::tests::regression_invalid_private_key_signing_error_does_not_echo_secret_material` verifies private-key input never leaks in error text.
- C-03 (Regression, AC-3): `regression_issue_5924_signature_profile_wipe_bytes_uses_zeroize_trait` verifies `wipe_bytes` keeps `zeroize` usage and blocks manual-loop regressions.
- C-04 (Integration, AC-4): `integration_issue_5924_service_auth_round_trip_remains_valid` verifies service-auth sign/verify flow remains stable after zeroization refactor.
- C-05 (Verify, AC-4): `cargo fmt --check` and strict `kamn-core` clippy pass for touched modules.

## Success Metrics / Observable Signals
- `wipe_bytes` uses `zeroize`-backed erasure in `signature_profile` signing flows.
- Failure-path errors remain secret-safe for invalid private key input.
- Static regression contract and integration round-trip tests pass.
- Scoped `kamn-core` formatting and strict clippy checks pass.


## Required Test Categories
- Unit: zeroization behavior in success/failure paths
- Functional: signing flows under error conditions
- Integration: signer profile end-to-end path
- Regression: manual wipe loop removal coverage
- Performance: N/A (documented)

## Dependencies
- #5916
