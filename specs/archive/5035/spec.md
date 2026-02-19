# Issue #5035 Spec

- Title: Subtask: M6 graph trust-propagation correctness and portability boundary contracts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5022` requires deterministic trust propagation and portability
contracts for M6 graph data. Existing M6 contracts cover trust scoring and
portable projection generation, but portability exports lack an explicit
requester-scoped boundary API, leaving cross-owner export control implicit.

## Acceptance Criteria
- AC-1: M6 exposes a requester-scoped portability export API that denies
  cross-owner access fail-closed with stable owner-scope reason marker.
- AC-2: Authorized scoped export produces deterministic edge projections
  equivalent to owner-scoped portability output.
- AC-3: Trust propagation and cross-owner edge registration reason markers are
  stabilized via exported constants.
- AC-4: Existing trust propagation ranking and portability projection behavior
  remains deterministic and passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add scoped portability export API in `data_layer_m6_graph_integration`.
- Add/align reason-marker constants for owner-scope and cross-owner edge denial.
- Extend conformance tests for authorized scoped export parity and cross-owner
  scoped export denial.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI workflow or shell-script changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Scoped portability export request with requester outside owner scope | Fail-closed `OwnerScopeViolation` with stable owner-scope reason marker |
| C-02 | AC-2 | Functional | Scoped portability export request with requester equal owner | Deterministic projection equals unscoped owner projection |
| C-03 | AC-3 | Regression | Cross-owner edge registration and trust query owner-scope denial paths | Reason markers match exported constants |
| C-04 | AC-4 | Conformance | Existing trust propagation and portability projection cases | Existing deterministic ranking/projection cases remain green |
| C-05 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m6_graph_integration`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5035.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5035.json`

## Success Metrics
- Scoped portability export boundary is explicit, deterministic, and tested.
- M6 reason-marker constants are stable and used by cross-owner denial paths.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
