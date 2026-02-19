# Issue #5229 Spec

- Title: Subtask: Typed-DID migration wave B for operator and governance surfaces
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
Operator and governance modules still accept raw DID `String` values across core authorization and workflow boundaries. This permits unchecked DID values to flow into privileged paths and produces inconsistent validation error formats.

## Scope
In:
- `crates/kamn-core/src/operator_binding.rs`
- `crates/kamn-core/src/operator_actions.rs`
- `crates/kamn-core/src/operator_dashboard_api.rs`
- `crates/kamn-core/src/operator_dashboard_ui.rs`
- `crates/kamn-core/src/governance_workflow.rs`
- `crates/kamn-core/src/task_payment.rs`
- Targeted wave-B tests in `crates/kamn-core/tests/**`

Out:
- Wave-A bridge/marketplace modules (`#5228`)
- Wave-C runtime/proof/reputation modules (`#5230`)
- Shell/python/workflow/template changes

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 220
- shell_to_rust_ratio_delta_estimate: -0.0011
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Scoped wave-B module identity boundaries use typed DID conversions (`AgentDid` plus validated human/operator DID wrappers) in core interfaces.
- AC-2: Public/API compatibility is preserved where required by introducing validated conversion wrappers rather than removing existing String-based contracts.
- AC-3: Invalid DID failures emit deterministic error markers (field + reason code + parser detail) and are covered by tests.
- AC-4: Shell LOC remains unchanged.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid DID-bound operator/governance requests | Core logic uses validated typed DID wrappers before authorization/workflow actions |
| C-02 | AC-2 | Integration | Existing operator/governance API inputs | Existing call sites continue to function through conversion boundaries |
| C-03 | AC-3 | Regression | Invalid DID in proposer/voter/operator/payer/payee boundaries | Structured deterministic invalid-DID errors with reason markers |
| C-04 | AC-4 | Conformance | diff-level shell/rust accounting + guardrail | shell delta stays zero |

## Test Mapping
- C-01/C-02/C-03:
  - `cargo test -p kamn-core --test operator_permissioned_actions`
  - `cargo test -p kamn-core --test operator_dashboard_api`
  - `cargo test -p kamn-core --test operator_dashboard_ui`
  - `cargo test -p kamn-core --test governance_workflow`
  - `cargo test -p kamn-core --test task_payment_workflow`
  - targeted module unit tests in `src/*`
- C-04:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`

## Success Metrics
- Wave-B authorization and governance execution paths reject malformed DID inputs before state mutation.
- Invalid DID errors are deterministic and machine-parsable.
- Existing wave-B behavioral tests remain green without shell-surface growth.
