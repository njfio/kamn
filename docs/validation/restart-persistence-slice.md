# Restart Persistence Slice

This runbook documents one current KAMN service-API slice on `main` that proves restart persistence across several persisted service-API surfaces. It is deliberately narrower than crash-recovery claims.

## Scope
- message state across restart
- task and escrow state across restart
- directory state across restart
- relayed and delivered status continuity across restart or fresh boot

## Proof Anchors
This slice is grounded in current-main tests:
- `integration_service_api_endpoint_persists_message_state_across_restart`
- `integration_service_api_endpoint_persists_task_and_escrow_state_across_restart`
- `integration_service_api_endpoint_persists_channel_creation_state_across_restart`
- `integration_service_api_endpoint_persists_agent_profile_query_state_across_restart`
- `integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract`
- `regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart`

## What This Proves
- restart persistence for message state is present on the current service-api storage path
- restart persistence for task and escrow state is present on the current service-api storage path
- restart persistence for directory state is present for channel creation and agent-profile query state
- relayed and delivered status remain queryable across restart or fresh boot on the current durable state path
- non-recipient restart reads preserve `relayed` and do not incorrectly promote delivery

## What This Does Not Prove
- not crash recovery
- not WAL durability after abrupt kill or corruption
- not consensus, bridge finality, or escrow settlement guarantees
- not production deployment readiness

## Operator Commands
Run the exact restart-persistence proof targets from a clean checkout:

```bash
cargo test -p kamn-node integration_service_api_endpoint_persists_message_state_across_restart -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_persists_task_and_escrow_state_across_restart -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_persists_channel_creation_state_across_restart -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_persists_agent_profile_query_state_across_restart -- --exact --nocapture
cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact --nocapture
cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --exact --nocapture
```

## Expected Evidence
- a created message remains queryable after restart with stable persisted state
- accepted task state and released escrow state remain present after restart
- created channel state and queried agent profile state remain present after restart
- a fresh reader can still observe `delivered` for recipient-visible state
- a non-recipient restart path keeps `relayed` stable

## Notes
- This slice proves restart persistence, not stronger crash-recovery semantics.
- The stronger durable relay and restart-status proof remains documented separately in `docs/validation/durable-cross-node-relay-slice.md`.
