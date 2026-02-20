# Issue #5328 Plan

## Approach
1. Keep `data_layer_postgres_execution_adapter.rs` as the public root module path.
2. Extract helper concerns into child modules:
   - `error.rs` for adapter error taxonomy and trait impls
   - `migration.rs` for migration discovery/statement splitting
   - `codec.rs` for row decode + blind-index JSON encode helpers
   - `validation.rs` for merkle payload validation helpers
3. Re-export required public symbols from root (`DataLayerPgExecutionAdapterError`, `data_layer_pg_collect_migration_files`).
4. Validate with existing adapter-related tests and strict clippy.

## Affected Modules
- `crates/kamn-core/src/data_layer_postgres_execution_adapter.rs`
- `crates/kamn-core/src/data_layer_postgres_execution_adapter/error.rs`
- `crates/kamn-core/src/data_layer_postgres_execution_adapter/migration.rs`
- `crates/kamn-core/src/data_layer_postgres_execution_adapter/codec.rs`
- `crates/kamn-core/src/data_layer_postgres_execution_adapter/validation.rs`
- `specs/5328/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: missing visibility wiring across new modules.
  - Mitigation: keep helper names stable; enforce through compile + adapter suites.
- Risk: API drift for public error/function exports.
  - Mitigation: root re-exports + public API test validation.

## Interfaces and Contracts
- Public module path remains `kamn_core::data_layer_postgres_execution_adapter`.
- Reason-code constants preserved.
- Existing test selector names preserved.
