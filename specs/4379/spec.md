# Issue #4379 Spec

Status: Implemented
Priority: P1
Parent: #4373

## Problem Statement

Simulated signature acceptance regressions must be caught by deterministic RED tests before implementation changes.

## Scope

In scope:
- Signed-to-Kolme policy/contract-lane RED test updates that fail when simulated profiles are accepted.
- RED assertions for missing native signer taxonomy output markers.

Out of scope:
- Runtime signer implementation changes.

## Acceptance Criteria

AC-1:
- Given run summary evidence
- When runtime command includes simulated signing profile marker
- Then tests fail with deterministic simulated-signature rejection expectation.

AC-2:
- Given policy report output
- When native signer taxonomy fields are absent
- Then tests fail deterministically.

AC-3:
- Existing signed-to-Kolme regression checks remain intact.

## Conformance Cases

C-01 (AC-1, Functional): mutate run summary/check command to simulated profile and assert `runtime_commit_simulated_signing_profile_detected`.

C-02 (AC-1, Functional): remove native profile marker and assert `runtime_commit_native_signing_profile_marker_missing`.

C-03 (AC-2, Conformance): assert policy JSON includes deterministic native signer taxonomy version/csv/value keys.

C-04 (AC-3, Regression): prior signed message/runtime commit mismatch checks still execute.
