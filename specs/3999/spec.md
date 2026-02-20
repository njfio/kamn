# Issue #3999 Spec

- Title: Task: add ci dry-run capacity governance checker and release go-no-go parity contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement

Capacity/load governance needs deterministic, low-cost CI dry-run checks that fail closed on marker,
threshold, and docs/runbook drift while preserving fast-gate runtime constraints.

## Scope

In scope:
- Dry-run governance checker and threshold contract enforcement for capacity/go-no-go markers.
- Docs/runbook parity and remediation marker completeness contracts.
- CI fast-mode selector/workflow exclusion checks to keep heavy execution off fast-gate.

Out of scope:
- Heavy load/local-heavy run-mode execution in default PR workflows.

## Shell-Surface Estimates

- shell_loc_delta_estimate: 8
- rust_loc_delta_estimate: 430
- shell_to_rust_ratio_delta_estimate: -0.0008
- shell_surface_mitigation_issue: None

## Acceptance Criteria

- AC-1: CI checker fails closed on marker/threshold/runbook drift.
- AC-2: release go/no-go marker parity remains deterministic with stable reason taxonomy.
- AC-3: CI cost controls are preserved (dry-run governance coverage without heavy-run leakage).
- AC-4: Unit, Functional, Integration, and Regression evidence exists and passes.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | baseline performance/go-no-go dry-run reports + thresholds | checker status pass/GO with deterministic markers |
| C-02 | AC-1 | Regression | tampered report/docs/taxonomy/threshold fixtures | checker fails closed with deterministic reason code(s) |
| C-03 | AC-2 | Integration | strategy/ops docs markers + checker reason constants | docs/checker parity remains synchronized |
| C-04 | AC-3 | Integration | CI tools fast-mode + ci-fast-gate workflow | dry-run checks required; heavy run-mode entries excluded |
| C-05 | AC-4 | Conformance | targeted governance suites | all mapped conformance tests pass |

## Test Mapping

- `cargo test -p kamn-core --test capacity_ci_dry_run_governance_contract -- --nocapture`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_overload_docs_parity_and_go_no_go_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_matches_ops_docs_and_runner_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_requires_remediation_marker_for_each_reason_code -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_overload_docs_parity_remediation_controls -- --exact`

## Success Metrics / Observable Signals

- Capacity dry-run checker emits deterministic pass/fail outcomes with stable reason taxonomy.
- Docs/runbook parity drift is fail-closed and remediation markers remain complete.
- CI fast-gate preserves dry-run governance coverage without heavy-load execution.
