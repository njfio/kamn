# Operator Dashboard UI MVP Composition (Issues #200, #201, #591, #593, #594)

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
- Added frontend MVP package `packages/kamn-dashboard` with:
  - deterministic mock snapshot adapter (`src/mock_api.ts`),
  - UI mapping and rendering logic (`src/dashboard.ts`),
  - exported package surface (`src/index.ts`),
  - frontend unit/functional/integration/regression tests (`tests/dashboard.test.ts`).

## UI Composition Rules
- Agent rows require non-empty identity/signing/agreement keys.
- Task timeline rows derive deterministic attention levels from `TaskState`.
- Message traces require at least one recipient and surface rejected/expired items as critical attention.
- Escrow rows surface disputed escrows as critical attention.
- Reputation overview enforces `delivery_rate` and `dispute_rate` in the `0..=1` range.
- Frontend severity badges are deterministic:
  - `severity-critical` for breached critical thresholds.
  - `severity-warning` for warning-only threshold breaches.
- Frontend stale-state indicators:
  - `stale-data-banner` is rendered when snapshot age exceeds threshold.
  - stale domain rows retain severity badges for operator triage.

## Audit Trace Rules
- Audit traces are projected from `OperatorActionAuditRecord`.
- Denied operator actions are marked critical in the UI model.
- Audit trace ordering is deterministic: newest `requested_at_unix` first.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test operator_dashboard_ui --test operator_dashboard_ui_docs
npm --prefix packages/kamn-dashboard test
bash scripts/frontend/test_dashboard_package.sh
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```

## Regression Guards
- critical severity badge and stale banner render together for degraded snapshots (`Regression: #591`).
