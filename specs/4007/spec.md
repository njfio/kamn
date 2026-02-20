# Issue #4007 Spec

- Title: Subtask: add docs-runbook parity contracts for performance-governance thresholds and escalation markers
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement

Capacity governance correctness depends on strategy + ops docs parity for threshold/escalation markers.
Drift in docs/runbook markers must fail closed even when checker logic is unchanged.

## Scope

In scope:
- Docs/runbook parity contracts for overload/capacity governance markers.
- Deterministic remediation/escalation marker coverage for each tracked reason code.

Out of scope:
- Automated paging/on-call integrations.

## Acceptance Criteria

- AC-1: docs/runbook parity checks fail closed when markers drift.
- AC-2: deterministic remediation/escalation marker references remain complete and stable.
- AC-3: integration composition between checker outputs and docs parity contracts remains enforced.
- AC-4: Unit/Functional/Integration/Regression tiers (as applicable to docs contracts) pass.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | drifted strategy/ops marker content | docs-contract failure with deterministic missing-marker signal |
| C-02 | AC-2 | Functional | baseline docs marker set | parity/remediation marker contracts pass |
| C-03 | AC-3 | Integration | checker marker constants + docs parity tests | checker/docs composition remains synchronized |
| C-04 | AC-4 | Conformance | targeted docs-contract suites | all targeted suites pass |

## Test Mapping

- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_overload_docs_parity_and_go_no_go_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_matches_ops_docs_and_runner_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_requires_remediation_marker_for_each_reason_code -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_overload_docs_parity_remediation_controls -- --exact`

## Success Metrics / Observable Signals

- All overload docs parity and remediation markers are asserted by fail-closed docs-contract tests.
- Strategy/ops docs remain synchronized with checker taxonomy/remediation surfaces.
