# Plan: Issue #5863 - Daemon Relay Drain Lifecycle Projection

- Issue: #5863
- Spec: `specs/5863/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. Add Service API helper to update persisted message statuses to `relayed` by message IDs.
2. Wire daemon relay drain path to call state-projection helper after draining spool entries.
3. Extend message retrieval transition logic to promote `relayed` -> `delivered` for recipient requesters.
4. Add integration/regression tests for drain projection and recipient retrieval continuity.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/full_supervisor_and_shutdown_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks and Mitigations
- Risk: State projection could clobber non-target message records.
  - Mitigation: project only explicit drained `message_id` matches and only `created` records.
- Risk: Recipient delivery transition may regress for existing `created` flow.
  - Mitigation: preserve current `created` behavior and extend transition predicate to include `relayed`.
- Risk: Drift between relay spool and state file paths in daemon mode.
  - Mitigation: reuse existing daemon path resolution helpers and env precedence.

## Interfaces / Contracts
- Internal helper contract:
  - Input: optional state-file path + drained message IDs.
  - Output: count of transitioned records.
  - Behavior: `created -> relayed`, idempotent for other statuses.
- Daemon logging contract:
  - report `service_api_relay_drained_count`
  - report `service_api_relay_projected_state_count`

## ADR Requirement
- Not required (no new dependency or external protocol changes).
