# Spec: Issue #5918 - Story: SDK and Service Transport Security Hardening

- Issue: #5918
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: story
- Priority: P0
- Area: sdk
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent epic: #5915

## Problem Statement
SDK/service transport has injection risks and incomplete secure transport support.

## Scope
In scope:
- Harden request construction, HTTPS support, probe validation, pooling, and managed signer execution safety.

Out of scope:
- Protocol feature work unrelated to security hardening.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: No header/path injection vectors remain in SDK request path construction.
- AC-2: HTTPS and transport lifecycle behaviors are production-safe and tested.
- AC-3: Managed signer execution surface is command-safe and does not leak secret env state.

## Conformance Cases
- C-01 (Functional, AC-1): Verify No header/path injection vectors remain in SDK request path construction.
- C-02 (Functional, AC-2): Verify HTTPS and transport lifecycle behaviors are production-safe and tested.
- C-03 (Functional, AC-3): Verify Managed signer execution surface is command-safe and does not leak secret env state.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: required for touched modules
- Functional: required for behavior paths
- Integration: required where cross-module runtime paths change
- Regression: required for each remediated finding
- Performance: required for hot paths, else justified N/A

## Dependencies
- #5915

