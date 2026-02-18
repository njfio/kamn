# Spec — Issue #4199

- Title: Add red tests for promotion evidence convergence completeness and tamper rejection
- Parent: #4193
- Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence
- Status: Implemented
- Priority: P1

## Problem Statement
Promotion evidence convergence contracts must fail closed when required evidence links are missing or convergence payload acceptance is tampered. Existing go/no-go gate tests cover several paths but do not explicitly lock these two drift classes for local full-runtime convergence evidence.

## Scope
In scope:
- Add deterministic red/green regression tests for missing required convergence evidence links.
- Add deterministic red/green regression tests for tampered convergence marker acceptance.
- Update planning docs to include convergence integrity markers and fail-closed reasons for this contract path.

Out of scope:
- Implementing new convergence checker logic (handled by #4200).
- Artifact-store backend or release orchestration changes.

## Acceptance Criteria
- AC-1: Given a release manifest missing required convergence evidence link(s), when go/no-go lane runs, then it fails closed with deterministic missing-artifact reason codes.
- AC-2: Given tampered convergence marker expectations in the manifest, when go/no-go lane runs, then it fails closed with deterministic required-marker-missing reason codes.
- AC-3: Regression tests lock both missing-link and tamper rejection behaviors.
- AC-4: Planning docs document convergence integrity markers and fail-closed reasons for this evidence path.

## Conformance Cases
- C-01 (Regression): Missing `local_full_runtime_convergence` required artifact produces `release_manifest_missing_required_artifact:local_full_runtime_convergence`. (AC-1)
- C-02 (Regression): Tampered `expected_success_marker` for `local_full_runtime_convergence` produces `release_manifest_success_marker_mismatch:local_full_runtime_convergence`. (AC-2)
- C-03 (Functional): Baseline go/no-go dry-run and run-mode remain green with no new reason-code drift. (AC-3)
- C-04 (Docs Contract): `docs/planning/kolme-devnet-ops.md` includes convergence integrity markers and fail-closed reasons for this path. (AC-4)

## Success Metrics
- Deterministic reason-code assertions for both missing-link and tampered-marker paths.
- No regression in existing go/no-go lane behavior.
- Planning docs and tests remain synchronized.
