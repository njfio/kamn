# Issue #4961 Spec

- Title: Task: define spec archival policy and archive directory governance contracts
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
The milestone required explicit governance rules for when completed specs leave the active tree, including deterministic policy markers that contract tests can enforce.

## Acceptance Criteria
- AC-1: Archive lifecycle policy defines deterministic move/retention rules for completed issue specs.
- AC-2: Archive location/index conventions are documented and machine-checkable.
- AC-3: Policy exceptions for audit/compliance artifacts are explicit and fail closed when missing markers.
- AC-4: Policy marker contract tests pass.

## Scope
In scope:
- Policy document and required marker taxonomy.
- Policy marker parity checks in archive-policy contract tests.

Out of scope:
- Archive migration tooling implementation details (covered by #4962).
- Initial archive migration wave execution (covered by #4963).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `docs/planning/spec-archive-policy.md` | deterministic lifecycle rules documented |
| C-02 | AC-2 | Functional | milestone/index marker checks | archive policy marker discovered and validated |
| C-03 | AC-3 | Regression | missing-marker mutation in test fixture | fail-closed reason codes emitted |
| C-04 | AC-4 | Unit/Regression | `bash scripts/ci/test_check_spec_archive_policy.sh` | marker contract suite passes |

## Test Mapping
- AC-1/AC-2: `bash scripts/ci/test_check_spec_archive_policy.sh`
- AC-3: `bash scripts/ci/test_check_spec_archive_policy.sh` fail-closed mutation cases
- AC-4: `bash scripts/ci/test_check_spec_archive_policy.sh`

## Success Metrics
- Archive policy markers are documented and contract-enforced.
- No policy-marker regressions in fast CI contract lanes.
