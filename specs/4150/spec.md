# Issue #4150 Spec

- Title: Subtask: add red tests for deployment preflight marker completeness and schema drift rejection
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md

## Problem Statement
Missing or drifted deployment-preflight markers can silently bypass promotion safeguards in release go/no-go workflows.

## Acceptance Criteria
- AC-1: Docs-contract tests fail closed when required deployment preflight marker fields are missing from the release checklist contract section.
- AC-2: Docs-contract tests fail closed when deployment preflight schema/taxonomy parity markers drift.
- AC-3: Release checklist documentation includes deterministic marker contract text and explicit regression references for future drift protection.

## Scope
In scope:
- `release_gonogo_checklist_docs` test additions for preflight marker completeness and schema parity drift checks.
- `docs/foundation/release-gonogo-checklist.md` contract section updates for deployment preflight marker governance.
- Lifecycle artifacts for issue `#4150`.

Out of scope:
- Deployment lane runtime behavior changes.
- New dependencies or wire/protocol changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run `release_gonogo_checklist_docs` against checklist with required markers | Test fails when required marker set is absent; passes with complete marker set |
| C-02 | AC-2 | Regression | Validate schema/taxonomy marker assertions in docs-contract tests | Drift/mismatch is rejected deterministically |
| C-03 | AC-3 | Conformance | Inspect checklist section + regression anchors | Marker contract and regression guards are documented and test-enforced |

## Test Mapping
- `cargo test -p kamn-core --test release_gonogo_checklist_docs`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | `checklist_contains_deployment_preflight_marker_completeness_schema_drift_gate` |
| AC-2 | ✅ | `checklist_contains_deployment_preflight_marker_completeness_schema_drift_gate` |
| AC-3 | ✅ | checklist contract section + `Regression: #4146` and `Regression: #4150` markers |

## Success Metrics
- Issue `#4150` closes with deterministic preflight marker completeness + schema drift tests green.
- Checklist contract includes required marker taxonomy/version/reason-code fields and regression references.
