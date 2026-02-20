# Issue #4001 Spec

- Title: Subtask: publish performance baseline artifacts with provenance metadata and drift-threshold seeds
- Status: Implemented (agent-authored; human review requested in PR)
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
Performance baseline fixtures exist, but they do not carry explicit provenance metadata and deterministic drift-threshold seed values needed for reproducible threshold evaluation and auditability.

## Acceptance Criteria
- AC-1: Baseline fixture artifacts include provenance metadata fields (source/lineage + schema/version markers).
- AC-2: Baseline fixture artifacts include deterministic drift-threshold seed values for each workload/lane record.
- AC-3: Report generation and threshold checker ingestion enforce fail-closed behavior when required provenance/seed markers are missing or malformed.
- AC-4: Tests cover metadata/seed generation and ingestion behavior (functional, integration, regression).
- AC-5: `docs/ci/strategy.md` documents baseline refresh policy and required metadata/seed contract markers.

## Scope
In scope:
- Extend performance baseline fixture schema to include provenance metadata and drift-threshold seeds.
- Wire metadata/seed values into generated report output and threshold checker ingestion.
- Add/extend tests for fail-closed marker enforcement.
- Update CI strategy documentation contract markers.

Out of scope:
- Automatic periodic baseline refresh orchestration.
- New benchmarking executables.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | baseline fixture JSON | required provenance fields present and schema-valid |
| C-02 | AC-2 | Functional | workload/lane fixture rows | deterministic drift-threshold seed fields present |
| C-03 | AC-3 | Regression | fixture/report with missing provenance/seed field | generator/checker fails closed with deterministic reason |
| C-04 | AC-3/AC-4 | Integration | generate + threshold-check flow | checker ingests baseline metadata/seed markers successfully |
| C-05 | AC-5 | Docs | `docs/ci/strategy.md` | baseline metadata + seed policy markers documented |

## Test Mapping
- `bash scripts/ci/test_generate_performance_smoke_report.sh`
- `bash scripts/ci/test_check_performance_thresholds.sh`
- `cargo test -p kamn-core --test ci_strategy_docs <new_docs_contract_test>`

## Success Metrics
- Baseline artifacts are reproducible with deterministic provenance/seed markers.
- Missing or malformed baseline provenance/seed fields fail closed in checks.
- CI strategy docs include enforceable marker contract coverage.
