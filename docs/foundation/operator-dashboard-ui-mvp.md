# Operator Dashboard UI MVP Composition (Issues #200 / #201)

This document captures the first implementation slice for the operator dashboard UI presentation model.

## Scope Delivered
- Added `crates/kamn-core/src/operator_dashboard_ui.rs` with:
  - `OperatorDashboardUi` composer that builds deterministic UI sections from `OperatorDashboardSnapshot`.
  - section models for:
    - agent list
    - task timeline
    - message traces
    - escrow status
    - reputation overview
    - operator audit traces
  - summary counters in `DashboardSummary` for blocked tasks, failed messages, disputed escrows, and denied operator actions.
  - typed validation errors via `OperatorDashboardUiError`.
- Added integration and regression tests in `crates/kamn-core/tests/operator_dashboard_ui.rs`.

## UI Composition Rules
- Agent rows require non-empty identity/signing/agreement keys.
- Task timeline rows derive deterministic attention levels from `TaskState`.
- Message traces require at least one recipient and surface rejected/expired items as critical attention.
- Escrow rows surface disputed escrows as critical attention.
- Reputation overview enforces `delivery_rate` and `dispute_rate` in the `0..=1` range.

## Audit Trace Rules
- Audit traces are projected from `OperatorActionAuditRecord`.
- Denied operator actions are marked critical in the UI model.
- Audit trace ordering is deterministic: newest `requested_at_unix` first.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test operator_dashboard_ui --test operator_dashboard_ui_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
