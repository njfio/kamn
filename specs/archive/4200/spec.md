# Spec — Issue #4200

- Title: Implement convergence verifier and deterministic promotion decision reason mapping
- Parent: #4193
- Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence
- Status: Implemented
- Priority: P1

## Problem Statement
Go/no-go gate output currently exposes raw policy outcome and reason codes but does not publish explicit convergence-verifier markers and stable promotion decision reason-mapping markers for downstream governance checks.

## Scope
In scope:
- Implement convergence-verifier marker projection in go/no-go gate report/output.
- Implement deterministic promotion-decision reason mapping markers from observed fail-closed reasons.
- Extend integration tests to validate convergence-to-decision mapping behavior.
- Update release go/no-go checklist docs and docs-contract tests for new markers.

Out of scope:
- External orchestration platform changes.
- New artifact storage backends.

## Acceptance Criteria
- AC-1: Invalid/missing promotion evidence is classified by convergence verifier markers with deterministic reason output.
- AC-2: Promotion decision reason mapping markers are deterministic and stable for equivalent failure classes.
- AC-3: Integration tests validate baseline and tamper paths for convergence-to-decision mapping.
- AC-4: Release-governance docs include convergence reason mapping references.

## Conformance Cases
- C-01 (Functional): Baseline go/no-go dry-run and run-mode emit convergence verifier and promotion reason mapping markers with `..._reason_code=none`. (AC-2, AC-3)
- C-02 (Regression): Missing `local_full_runtime_convergence` manifest link emits `promotion_evidence_reason_code=promotion_evidence_link_missing` and deterministic promotion decision mapping markers. (AC-1, AC-3)
- C-03 (Regression): Tampered `local_full_runtime_convergence` success marker emits `promotion_evidence_reason_code=promotion_evidence_payload_tamper_detected` and deterministic promotion decision mapping markers. (AC-1, AC-3)
- C-04 (Docs Contract): Release go/no-go checklist includes convergence reason taxonomy + mapping markers. (AC-4)

## Success Metrics
- Deterministic convergence reason projection for missing-link and tampered-marker failure classes.
- Stable mapped reason-code projection in report/output across repeated runs.
- Updated docs + docs tests remain synchronized.
