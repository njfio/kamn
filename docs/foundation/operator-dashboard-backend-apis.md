# Operator Dashboard Backend APIs (Issues #202 / #203)

This document captures the first implementation slice for backend read APIs that power human-operator dashboard visibility.

## Scope Delivered
- Added `crates/kamn-core/src/operator_dashboard_api.rs` with:
  - `OperatorDashboardApi` read-model registry for agents, tasks, messages, escrows, and reputation.
  - deterministic pagination/filtering via `DashboardPageRequest` and `DashboardPage<T>`.
  - cross-domain upsert adapters:
    - `upsert_agent_from_hierarchy(...)`
    - `upsert_task(...)`
    - `upsert_message_from_store(...)`
    - `upsert_escrow(...)`
    - `upsert_reputation(...)`
  - unified snapshot response via `snapshot(...)`.
  - typed errors via `OperatorDashboardApiError`.
- Added integration and regression tests in `crates/kamn-core/tests/operator_dashboard_api.rs`.

## Pagination and Filter Rules
- Page limit must be positive.
- Cursor tokens must match an existing key in the current filtered result set.
- Optional prefix filter applies before pagination.
- Ordering is deterministic (lexical key ordering from `BTreeMap`).

## Operational Read Models
- Agent view includes DID and key-role references.
- Task view includes requester, assignee, and lifecycle state.
- Message view includes participants and lifecycle status.
- Escrow view includes status and remaining amount.
- Reputation view includes trust/delivery/dispute metrics.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test operator_dashboard_api --test operator_dashboard_api_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
