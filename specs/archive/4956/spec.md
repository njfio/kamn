# Issue #4956 Spec

- Title: Story: implement spec archival lifecycle and completed-issue archive policy enforcement
- Status: Implemented
- Type: story
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Completed issue specs were accumulating in active trees. The story required policy, tooling, and first-wave execution to establish a sustainable archival lifecycle.

## Acceptance Criteria
- AC-1: Archival policy and deterministic placement rules are documented and marker-enforced.
- AC-2: Archival tooling moves completed specs safely with deterministic outputs.
- AC-3: Initial archive migration wave publishes traceable archive index/report artifacts.
- AC-4: Active-vs-archived placement parity contracts remain green.

## Scope
In scope:
- Archive policy marker definition.
- Archive migration tooling and contract tests.
- First archive migration wave and parity hardening.

Out of scope:
- Non-spec archival domains.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | archive policy docs + marker checks | deterministic policy marker coverage |
| C-02 | AC-2 | Functional | archive tool execution/tests | deterministic archive outputs |
| C-03 | AC-3 | Integration | archive index/report publication | first-wave archive map published |
| C-04 | AC-4 | Regression | `bash scripts/ci/test_check_spec_archive_policy.sh` | placement/index parity fail-closed behavior |

## Test Mapping
- AC-1..AC-4: `bash scripts/ci/test_check_spec_archive_policy.sh`
- Child task evidence: `#4961`, `#4962`, `#4963`

## Success Metrics
- Archive lifecycle story completed with policy/tooling/wave evidence and contract enforcement.
