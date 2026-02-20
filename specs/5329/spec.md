# Issue #5329 Spec

- Title: Audit and disposition long-lived ignored kamn-core tests
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
R45 flagged a long-lived ignored-test population that remained stable across multiple review cycles. The scoped files (`input_mutation_coverage_guided.rs` and `kolme_runtime_commit_http_transport.rs`) already carry anti-`#[ignore]` guards, but the global ignored-test inventory still contains long-lived deep-lane entries that should be reduced or explicitly justified.

## Acceptance Criteria
- AC-1: Reduce ignored-test inventory count by promoting at least two long-lived deep-lane tests from `#[ignore]` to deterministic env-gated execution.
- AC-2: Update ignored-test inventory artifacts so baseline and metadata align with source (`fixtures/ci/ignored_test_inventory_baseline.json`, `fixtures/ci/ignored_test_inventory_metadata.json`).
- AC-3: Preserve deterministic local default behavior (deep-lane tests remain fast/no-op unless opt-in env guard is set).
- AC-4: Ignored-test drift and parser contracts pass after the reduction.

## Scope
In scope:
- Convert selected deep-lane tests from `#[ignore]` to env-gated runtime checks.
- Refresh ignored-test baseline/metadata fixtures to reflect the reduced set.
- Add explicit per-entry disposition notes for remaining ignored tests in metadata.
- Add/refresh issue spec artifacts.

Out of scope:
- Reworking all remaining ignored tests in one change.
- CI lane architecture changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | ignored inventory generator vs source | ignored count decreases by >=2 |
| C-02 | AC-2 | Conformance | baseline + metadata fixtures | both match generated inventory keys |
| C-03 | AC-3 | Unit | promoted deep-lane tests with env unset | test returns quickly without `#[ignore]` |
| C-04 | AC-4 | Integration | ignored-test drift/parser contracts | all checks pass |

## Test Mapping
- `python3 scripts/ci/ignored_test_inventory.py generate --repo-root . --output-json /tmp/current-ignored-inventory.json`
- `bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report.json`
- `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
- `bash scripts/ci/test_ignored_test_inventory_parser_contract.sh`

## Success Metrics
- Ignored inventory count reduced from `12` to `10` (or lower in this change set).
- Drift/parser contract lanes remain pass/green.
