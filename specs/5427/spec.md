# Issue #5427 Spec — Coherence-Contract Batching Policy Markers

- Status: Reviewed
- Issue: #5427
- Parent: #3812
- Milestone: R27 Program: operational hardening and live validation

## Problem Statement
Topology/coherence hardening previously expanded into one issue per coherence dimension, increasing governance overhead and reducing delivery coherence. We need a deterministic policy in the review docs that enforces future batching expectations.

## Scope
In scope:
- Add a policy section to `docs/review/gaps-and-issues-r45.md` defining coherence batching expectations.
- Add deterministic policy markers with numeric targets and thresholds.
- Add docs contract tests to enforce marker presence and numeric consistency.

Out of scope:
- Rewriting historical closed issues.
- Production runtime/feature behavior changes.

## Acceptance Criteria
- AC-1: R45 review doc contains a dedicated coherence batching policy section with explicit target bundling range.
- AC-2: Deterministic batching markers are present and parseable as numeric/string values.
- AC-3: Docs contract tests validate marker presence and internal numeric consistency.
- AC-4: Lifecycle artifacts and issue process logs capture Specify/Plan/Tasks/Implement/Verify.

## Conformance Cases
- C-01 (Functional, AC-1): section `### 5.4 Coherence Contract Batching Policy` exists and defines bundle-size target guidance.
- C-02 (Conformance, AC-2): markers for schema version, dimension baseline, target bundle min/max, and issue cap are present.
- C-03 (Regression, AC-3): docs contract tests pass for marker presence and numeric consistency.
- C-04 (Conformance, AC-4): `specs/5427/{spec,plan,tasks}.md` exist and issue logs include phase transitions.

## Success Metrics
- Policy section and markers are present in the review doc.
- Docs contract test guards the policy from silent drift.
- Future topology/coherence planning can reference bounded issue-count targets directly.

## AC -> Tests Mapping (initial)
- AC-1: new functional docs contract test for section/marker presence.
- AC-2: marker parseability and numeric-threshold consistency assertions.
- AC-3: targeted docs contract test run.
- AC-4: artifact paths + issue process logs.
