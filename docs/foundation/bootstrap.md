# Foundation Bootstrap (Issue #16)

This document describes the initial KAMN chain bootstrap scaffold.

## What Exists
- Rust workspace with two crates:
  - `kamn-core`: configuration, state namespaces, runtime wiring, bootstrap planner.
  - `kamn-node`: minimal binary entrypoint for processor/listener/approver bootstrap.
- Baseline state namespaces for key PRD domains:
  - DID registry
  - channels
  - messages
  - tasks
  - reputation
  - escrows
- App-state schema/version primitives:
  - `APP_STATE_VERSION` for the current schema revision.
  - `AppStateSchema` for version + namespace bundle.
  - `canonical_state_key(namespace, entity, id)` for deterministic key serialization and strict validation.
- Migration scaffolding:
  - `MigrationRegistry`, `MigrationStep`, and `MigrationPlan` for deterministic, contiguous state upgrades.
  - `bootstrap_from_state_version(...)` to compute the startup migration plan from persisted state to current schema.

## Local Validation
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Run bootstrap binary example:

```bash
cargo run -p kamn-node -- --role processor --chain-id kamn-devnet --chain-version v0.1.0
```

## Notes
This is intentionally minimal and dependency-light so the bootstrap path is fast and auditable.
Migration execution is not implemented yet; this stage focuses on deterministic planning and validation hooks.
For role interaction baseline coverage, see `docs/foundation/role-smoke.md`.
For transaction invariant guard behavior, see `docs/foundation/transaction-guards.md`.
For canonical invariant IDs and taxonomy mapping, see `docs/foundation/invariants.md`.
For the Rust SDK first implementation slice, see `docs/foundation/rust-sdk-alpha.md`.
For token model and genesis allocation controls, see `docs/foundation/token-model.md`.
For escrow lifecycle state transitions, see `docs/foundation/escrow-lifecycle.md`.
For PaymentOffer and PaymentConfirm integration with task completion workflows, see `docs/foundation/task-payment-workflow.md`.
For multi-AZ topology and failover operations, see `docs/foundation/multi-az-failover-runbook.md`.
For observability SLO evaluation and dashboard rollup controls, see `docs/foundation/observability-slo-dashboards.md`.
For security control ownership and enforcement mapping, see `docs/foundation/threat-control-matrix.md`.
For anti-hallucination instruction validation controls, see `docs/foundation/instruction-verification.md`.
For deterministic key lifecycle and rotation transitions, see `docs/foundation/key-lifecycle.md`.
For tamper-evident key lifecycle audit trail verification controls, see `docs/foundation/key-lifecycle-audit-trails.md`.
For pluggable local/secure signer backend controls, see `docs/foundation/signer-backend-abstraction.md`.
For key compromise containment and recovery workflows, see `docs/foundation/key-recovery.md`.
For Python SDK parity slice, see `docs/foundation/python-sdk-beta.md`.
For TypeScript SDK beta and shared schema package parity slice, see `docs/foundation/typescript-sdk-beta.md`.
For OpenClaw flagship connector reference workflow controls, see `docs/foundation/openclaw-connector-reference-workflow.md`.
For DID method and canonical DID document schema controls, see `docs/foundation/did-method.md`.
For DID register/resolve/update/revoke transaction behavior, see `docs/foundation/did-registry-transactions.md`.
For canonical message envelope schema and validation controls, see `docs/foundation/message-envelope-schema.md`.
For message lifecycle state machine and index query controls, see `docs/foundation/message-lifecycle.md`.
For nonce/TTL/replay enforcement and failed-delivery notice controls, see `docs/foundation/message-delivery-guards.md`.
For direct/group plus specialized broadcast/task/marketplace/governance channel models, see `docs/foundation/channel-models.md`.
For channel permission and retention policy controls, see `docs/foundation/channel-permissions-retention.md`.
For marketplace listing registration, discovery filters, and negotiation hook controls, see `docs/foundation/service-marketplace-discovery.md`.
For agent key hierarchy role bindings and ephemeral session key controls, see `docs/foundation/agent-key-hierarchy.md`.
For direct-message encryption path controls, see `docs/foundation/direct-message-encryption.md`.
For group sender-key distribution and rotation controls, see `docs/foundation/group-sender-key-rotation.md`.
For task state machine and legal transition validation controls, see `docs/foundation/task-state-machine.md`.
For task operation command handling controls, see `docs/foundation/task-operations.md`.
For task artifact integrity references and provenance metadata controls, see `docs/foundation/task-artifacts-provenance.md`.
For bridge inbound/outbound adapter abstraction controls, see `docs/foundation/bridge-adapter-abstraction.md`.
For Telegram bridge listener-validated inbound flow controls, see `docs/foundation/telegram-bridge-listener-validation.md`.
For Discord bridge approver-gated outbound flow controls, see `docs/foundation/discord-bridge-approver-gating.md`.
For Ethereum/Solana cross-chain adapter foundation controls, see `docs/foundation/cross-chain-bridge-adapters.md`.
For CI cache strategy and bounded parallelism controls, see `docs/foundation/ci-caching-parallelism.md`.
For flaky-test quarantine and bounded retry controls, see `docs/foundation/ci-flaky-quarantine.md`.
For release rollback triggers and post-upgrade verification controls, see `docs/foundation/upgrade-rollback-runbook.md`.
For release go/no-go checklist and dry-run evidence controls, see `docs/foundation/release-gonogo-checklist.md`.
For semantic version policy and compatibility matrix controls, see `docs/foundation/versioning-compatibility-matrix.md`.
For A2A and MCP interoperability mapping controls, see `docs/foundation/a2a-mcp-interoperability.md`.
For KAMN to DIDComm v2 compatibility profile controls, see `docs/foundation/didcomm-v2-compatibility-profile.md`.
For `kamn:did` to DID Core 1.1 conformance controls, see `docs/foundation/did-core-conformance-kamn-method.md`.
For redaction and tombstone compliance workflow controls, see `docs/foundation/redaction-tombstones.md`.
For compliance audit export interface controls, see `docs/foundation/audit-export-interfaces.md`.
For configurable retention policy enforcement controls, see `docs/foundation/retention-policy-engine.md`.
For data classification tiers and write-path tagging enforcement controls, see `docs/foundation/data-classification-tagging.md`.
For optional operator binding proof validation and configure/revoke/read-history permission controls, see `docs/foundation/operator-binding-permissions.md`.
For fast/slow/archive sync-mode operational profile controls, see `docs/foundation/sync-mode-profiles.md`.
For PRD 13.2 benchmark target validation evidence controls, see `docs/foundation/performance-target-benchmarking.md`.
