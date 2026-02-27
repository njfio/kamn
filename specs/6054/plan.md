# Plan: Issue #6054

## Approach
1. Add RED tests in `crates/kamn-node/src/service_api_endpoint/tests.rs` for sqlite-backed state paths:
   - assert sqlite database opens via `SqliteStoreBackend`,
   - assert expected namespace/key snapshot row exists after message creation,
   - assert relay projection writes `created -> relayed` status back into sqlite snapshot.
2. Add storage-backend resolution in `state_io.rs`:
   - detect sqlite paths (`.sqlite`, `.sqlite3`, `.db`) vs JSON file paths,
   - centralize state payload load/persist helpers for both backends.
3. Route `ServiceApiMessageStore` load/refresh/persist through new `state_io` helpers.
4. Validate with targeted `kamn-node` tests for sqlite and existing JSON projection contracts.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/state_io.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/tests.rs`
- `specs/6054/spec.md`
- `specs/6054/plan.md`
- `specs/6054/tasks.md`

## Risks / Mitigations
- Risk: sqlite path detection could unintentionally reroute non-sqlite custom paths.
  Mitigation: constrain sqlite detection to explicit extensions (`.sqlite`, `.sqlite3`, `.db`) and preserve existing JSON default path (`.json`).
- Risk: payload encoding mismatch between sqlite bytes and JSON parser.
  Mitigation: treat sqlite payload as UTF-8 JSON bytes; fail closed on decode/parse errors with explicit error messages.
- Risk: regression in existing JSON behavior.
  Mitigation: keep JSON projection tests in targeted verification run.

## Interfaces / Contracts
- State payload contract remains JSON snapshot schema `kamn.runtime.service-api-message-store.v2`.
- Storage backend contract:
  - JSON mode: state payload in file at `state_file`.
  - SQLite mode: payload bytes under namespace `service_api_state` and key `message_store_snapshot`.
- Relay projection contract unchanged: created message IDs can be projected to `relayed`.
