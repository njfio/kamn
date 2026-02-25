# Plan: Issue #5990

## Approach
1. Extend `ServiceApiPersistedMessageStoreSnapshot` with a durable `contents` map and add content lifecycle CRUD/transition methods in `message_store.rs`.
2. Wire content routes in `middleware_impl.rs` to call message-store methods rather than falling through to synthetic payload rendering.
3. Keep existing error model fail-closed:
   - missing content IDs -> `404` + `service_api_route_not_found`
   - persistence failures -> `500` + `service_api_state_persistence_failed`
4. Add integration tests in `main_tests/service_api_endpoint_tests.rs` for register/expire/tombstone/query across restart.
5. Run relay durability regression test lane to verify baseline cross-node relay flow remains green.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks / Mitigations
- Risk: Snapshot schema extension breaks backward compatibility for existing state files.
  Mitigation: add `#[serde(default)]` for new `contents` field so older snapshots deserialize.
- Risk: Route wiring order changes existing behavior unexpectedly.
  Mitigation: add route-specific integration tests that validate status codes and payload fields end-to-end.
- Risk: Touching middleware may affect relay flow.
  Mitigation: run targeted relay regression test after content flow changes.

## Interfaces / Contracts
- Existing route paths and response types remain unchanged.
- New durable behavior applies to live middleware execution path.
- `render_service_api_endpoint_response` remains available for non-live contract projection paths.
