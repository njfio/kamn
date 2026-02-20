# Issue #5329 Spec

- Title: Audit and disposition long-lived ignored kamn-core tests
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
R45 flagged a long-lived ignored-test population that remained stable across multiple review cycles. The scoped files (`input_mutation_coverage_guided.rs` and `kolme_runtime_commit_http_transport.rs`) already carry anti-`#[ignore]` guards, but the global ignored-test inventory still contains long-lived deep-lane entries that should be reduced or explicitly justified.

## Acceptance Criteria
- AC-1: Provide explicit disposition for every currently ignored test in metadata and either (a) reduce the ignored count, or (b) justify retention with linked follow-up tracking.
- AC-2: Update ignored-test inventory artifacts so baseline and metadata align with source (`fixtures/ci/ignored_test_inventory_baseline.json`, `fixtures/ci/ignored_test_inventory_metadata.json`).
- AC-3: Preserve deterministic local default behavior and avoid introducing CI fast-gate budget regressions.
- AC-4: Ignored-test drift and parser contracts pass after the reduction.

## Scope
In scope:
- Audit currently ignored entries and add explicit per-entry disposition notes in metadata.
- Refresh ignored-test metadata fixture while keeping baseline aligned to source truth.
- Document justified retention path through linked follow-up tracking where reduction is deferred.
- Add/refresh issue spec artifacts.

Out of scope:
- Reworking all remaining ignored tests in one change.
- CI lane architecture changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | metadata fixture entries | each ignored test key has explicit disposition + tracking issue |
| C-02 | AC-2 | Conformance | baseline + metadata fixtures | both match generated inventory keys |
| C-03 | AC-3 | Integration | fast-gate budget evidence from prior run + no new Rust surface | no additional Rust-surface-induced budget risk introduced in this issue delta |
| C-04 | AC-4 | Integration | ignored-test drift/parser contracts | all checks pass |

## Test Mapping
- `python3 scripts/ci/ignored_test_inventory.py generate --repo-root . --output-json /tmp/current-ignored-inventory.json`
- `bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report.json`
- `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
- `bash scripts/ci/test_ignored_test_inventory_parser_contract.sh`

## Success Metrics
- Ignored inventory count remains source-of-truth aligned (`12` in current cycle) with explicit per-entry dispositions.
- Retention decisions are explicitly tracked via linked follow-up issue references.
- Drift/parser contract lanes remain pass/green.
