# Operator Dashboard Backend APIs (Issues #202, #203, #591, #610, #640)

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
- Added explicit frontend consumer contract mapping for `packages/kamn-dashboard`:
  - backend snapshot shape maps to deterministic frontend shell state projections.
  - backend absence/failure semantics map to deterministic loading/error/empty frontend states.
  - live backend fetch path requires operator session token + allowed role gates.

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

## Frontend Contract Mapping
- Frontend package consumer: `packages/kamn-dashboard`.
- Backend snapshot response maps to frontend `ready` shell state.
- Backend fetch-in-flight maps to frontend `dashboard-loading` state.
- Backend fetch failures map to frontend `dashboard-error` state.
- Empty backend snapshot sets map to frontend `dashboard-empty` state.
- Live backend session policy:
  - operator session token is required before backend fetch.
  - expired sessions are rejected before backend fetch.
  - allowed roles default to `operator` and `admin`.
  - role mismatches map to deterministic `dashboard-error` output.
  - backend requests carry `Authorization: Bearer <token>` and `X-KAMN-Role` headers.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test operator_dashboard_api --test operator_dashboard_api_docs
npm --prefix packages/kamn-dashboard test
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```

## Regression Guards
- tampered pagination cursor tokens are rejected (`Regression: #203`).
- dashboard shell state mapping remains deterministic for loading/error/empty (`Regression: #591`).
- missing/expired/unauthorized operator sessions are rejected in live backend fetch path (`Regression: #640`).
