# Spec: Issue #5922 - Task: Replace fake SHA-256 labels with real sha2::Sha256 in data layer M0-M5

- Issue: #5922
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Current functions label outputs as sha256 while using custom FNV-like mixing.

## Scope
In scope:
- Introduce shared digest utility backed by sha2::Sha256; migrate M0-M5 call sites.

Out of scope:
- Data-model redesign unrelated to hash correctness.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: All M0-M5 digest outputs are computed with real SHA-256.
- AC-2: Duplicate deterministic_digest_256_hex implementations are removed.
- AC-3: Compatibility/regression tests verify expected digest format and deterministic outputs for fixed vectors.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify All M0-M5 digest outputs are computed with real SHA-256.
- C-02 (Functional, AC-2): Verify Duplicate deterministic_digest_256_hex implementations are removed.
- C-03 (Functional, AC-3): Verify Compatibility/regression tests verify expected digest format and deterministic outputs for fixed vectors.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: digest utility vectors
- Functional: M0-M5 module hash outputs
- Integration: append-only ledger integrity path
- Regression: old fake hash path absent
- Performance: hash throughput non-regression check

## Dependencies
- #5916

