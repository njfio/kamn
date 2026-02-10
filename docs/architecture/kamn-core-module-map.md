# KAMN Core Module Map

This map provides a high-level architecture guide for `crates/kamn-core` so
contributors can locate runtime/domain responsibilities quickly.

## Domain Clusters

### Identity and Keying

- `did`, `did_registry`, `agent_key_hierarchy`, `key_lifecycle`, `key_recovery`
- Purpose:
  - DID registration, lifecycle mutation policy, and key hierarchy/recovery
    controls.

### Messaging and Channels

- `message_envelope`, `message_lifecycle`, `message_delivery_guards`,
  `channel_models`, `channel_policies`, `group_channel_crypto`,
  `direct_message_crypto`
- Purpose:
  - Envelope validation, lifecycle tracking, delivery guard contracts, and
    channel/membership policy enforcement.

### Task and Escrow Lifecycle

- `task_operations`, `task_lifecycle`, `task_payment`, `escrow`,
  `service_marketplace`, `reputation_state`, `reputation_signals`
- Purpose:
  - Task DAG and state transitions, payment/escrow settlement flow, and
    reputation signal routing.

### Runtime and Observability

- `runtime`, `state`, `migrations`, `bootstrap`, `observability`, `smoke`,
  `performance_targets`, `invariants`, `transaction`
- Purpose:
  - Runtime wiring, state versioning/migrations, health/SLO surfaces, and
    fail-closed invariant enforcement.

### External Adapters and Bridges

- `bridge_adapter`, `cross_chain_bridge`, `cross_chain_receipt`,
  `telegram_bridge`, `discord_bridge`, `kolme_runtime_commit`
- Purpose:
  - Platform adapters, cross-chain receipt normalization/finality, and Kolme
    commit client abstractions.

## Runtime Flow (Condensed)

1. Identity verification and key checks gate actor eligibility.
2. Message/channel/task actions pass guard + invariant checks.
3. State mutations are committed through runtime/store surfaces.
4. Observability/performance lanes evaluate SLO and contract outcomes.
5. Adapter/bridge layers emit normalized external receipts/events.

## Contributor Entry Points

- Public export surface:
  - `crates/kamn-core/src/lib.rs`
- Missing-doc policy and graduation checks:
  - `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- Verification lanes:
  - `scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh`
