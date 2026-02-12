# KAMN Core Module Map

This map provides a high-level architecture guide for `crates/kamn-core` so
contributors can locate runtime/domain ownership responsibilities quickly.

## Ownership Matrix

### Identity and Key Management

- Modules:
  - `did`, `did_registry`, `agent_key_hierarchy`, `key_lifecycle`,
    `key_recovery`, `signature_profile`, `operator_binding`
- Ownership boundary:
  - Agent identity roots, DID lifecycle mutation controls, operator/agent
    binding, and key rotation/recovery constraints.
- Runtime/data-flow ownership:
  - Establishes caller identity and key legitimacy before any mutating
    protocol action is admitted.

### Messaging and Channel Control Plane

- Modules:
  - `message_envelope`, `message_lifecycle`, `message_delivery_guards`,
    `channel_models`, `channel_policies`, `group_channel_crypto`,
    `direct_message_crypto`, `anti_spam`, `instruction_verify`
- Ownership boundary:
  - Message schema validity, channel permissions, sender-key distribution, and
    delivery admission/rejection policy.
- Runtime/data-flow ownership:
  - Governs ingress message checks and channel-level policy before state writes
    and delivery fan-out.

### Task, Escrow, and Economic Settlement

- Modules:
  - `task_operations`, `task_lifecycle`, `task_payment`, `task_artifacts`,
    `escrow`, `service_marketplace`, `token`
- Ownership boundary:
  - Task DAG progression, payout/refund transitions, escrow finality, and
    settlement artifacts.
- Runtime/data-flow ownership:
  - Owns task lifecycle mutation and settlement state transitions used by
    audit/reconciliation lanes.

### Reputation and Abuse Response

- Modules:
  - `reputation_state`, `reputation_signals`, `trust_score`, `transaction`,
    `retention_engine`
- Ownership boundary:
  - Reputation signal ingestion, weighted trust updates, and abuse/penalty
    routing.
- Runtime/data-flow ownership:
  - Feeds trust and risk posture into guard/admission decisions for messaging,
    tasks, and governance actions.

### Runtime, State, and Safety

- Modules:
  - `runtime`, `state`, `bootstrap`, `migrations`, `namespaces`, `config`,
    `durable_guard_store`, `invariants`, `performance_targets`, `smoke`,
    `validator_lifecycle`, `watchdog`, `upgrade_orchestration`
- Ownership boundary:
  - Runtime orchestration, state schema/version handling, invariant taxonomy,
    and resilience/failover controls.
- Runtime/data-flow ownership:
  - Coordinates deterministic mutation ordering and durable snapshots, while
    enforcing safety checks and upgrade/cutover posture.

### Storage, Content, and Compliance

- Modules:
  - `content_storage`, `content_retrieval`, `content_lifecycle`,
    `content_replication`, `data_classification`, `redaction_compliance`,
    `audit_exports`
- Ownership boundary:
  - Content persistence/lookup, replication health, retention/redaction policy,
    and audit evidence export contracts.
- Runtime/data-flow ownership:
  - Owns data-plane storage + retrieval behaviors and compliance policy
    enforcement for classified content.

### Governance and Operator Control Plane

- Modules:
  - `governance_workflow`, `operator_actions`, `operator_dashboard_api`,
    `operator_dashboard_ui`, `observability`
- Ownership boundary:
  - Proposal/vote/execution lifecycle, privileged operator action policy, and
    dashboard/reporting surfaces.
- Runtime/data-flow ownership:
  - Controls policy-driven governance transitions and exposes read-only
    operational state for operators.

### External Integration and Bridge Surface

- Modules:
  - `bridge_adapter`, `cross_chain_bridge`, `cross_chain_receipt`,
    `telegram_bridge`, `discord_bridge`, `kolme_runtime_commit`
- Ownership boundary:
  - Platform ingress/egress normalization, cross-chain finality checks, and
    Kolme commit receipt projection.
- Runtime/data-flow ownership:
  - Mediates external transport and settlement confirmations into normalized
    internal state mutations.

### Kolme Extraction Boundary (In Progress)

- Crate/module surface:
  - `crates/kamn-core/src/kolme_runtime_commit.rs` (legacy compatibility shim
    while extraction proceeds)
  - `crates/kamn-kolme/src/codec.rs`, `crates/kamn-kolme/src/transport.rs`,
    `crates/kamn-kolme/src/finality.rs`, `crates/kamn-kolme/src/pipeline.rs`,
    `crates/kamn-kolme/src/api_codec.rs`, `crates/kamn-kolme/src/receipt_finality.rs`,
    `crates/kamn-kolme/src/runtime_lifecycle_policy.rs`,
    `crates/kamn-kolme/src/runtime_request_identity_policy.rs`,
    `crates/kamn-kolme/src/endpoint_policy.rs`, `crates/kamn-kolme/src/block_scan_policy.rs`,
    `crates/kamn-kolme/src/notification_policy.rs`, `crates/kamn-kolme/src/websocket_policy.rs`,
    `crates/kamn-kolme/src/http_response_policy.rs`, `crates/kamn-kolme/src/tls_policy.rs`,
    `crates/kamn-kolme/src/provider_response_policy.rs`, `crates/kamn-kolme/src/flat_json_policy.rs`,
    `crates/kamn-kolme/src/provider_outcome_policy.rs`, `crates/kamn-kolme/src/block_fallback_policy.rs`,
    `crates/kamn-kolme/src/transport_request_policy.rs`, `crates/kamn-kolme/src/broadcast_payload_policy.rs`
- Ownership boundary:
  - `kamn-kolme` is the dedicated home for runtime-commit transport/codec/finality
    contracts (including direct signed payload validation, receipt-to-commit
    finality mapping and parse helpers, lifecycle/finality, deterministic
    request identity, and JSON escape serialization policy). `kamn-core`
    retains temporary compatibility exports until full migration is complete.
- Runtime/data-flow ownership:
  - New runtime-commit submissions should target `kamn-kolme` contracts first,
    then map back to `kamn-core` compatibility paths only where migration is
    still in progress.

### Cryptographic Signer and ZK Surface

- Modules:
  - `signer_backend`, `zk_message_proofs`
- Ownership boundary:
  - Deterministic signing provider integration and zero-knowledge proof
    representation/verification surfaces.
- Runtime/data-flow ownership:
  - Provides cryptographic assurance artifacts consumed by message and
    settlement safety lanes.

## Runtime Flow (Condensed)

1. Identity and key modules (`did`, `agent_key_hierarchy`, `key_lifecycle`)
   establish actor legitimacy.
2. Messaging/channel control modules validate envelopes, permissions, and abuse
   posture before admitting actions.
3. Task/escrow/economic modules execute mutation flows and settlement outcomes.
4. Runtime/state/safety modules commit and persist validated state transitions.
5. Governance/operator surfaces publish execution state and operator-facing APIs.
6. Bridge/adapters and signer/ZK surfaces emit external receipts and proof
   artifacts for reconciliation.

## Missing-Docs Graduation Status

- Graduated modules currently enforced outside the allow-list:
  - `agent_key_hierarchy`
  - `anti_spam`
  - `audit_exports`
  - `bootstrap`
  - `config`
  - `content_lifecycle`
  - `content_replication`
  - `content_retrieval`
  - `data_classification`
  - `did`
  - `did_registry`
  - `content_storage`
  - `cross_chain_bridge`
  - `cross_chain_receipt`
  - `direct_message_crypto`
  - `discord_bridge`
  - `durable_guard_store`
  - `group_channel_crypto`
  - `key_lifecycle`
  - `key_recovery`
  - `kolme_runtime_commit`
  - `migrations`
  - `namespaces`
  - `observability`
  - `operator_actions`
  - `operator_dashboard_api`
  - `operator_dashboard_ui`
  - `operator_binding`
  - `performance_targets`
  - `redaction_compliance`
  - `retention_engine`
  - `reputation_signals`
  - `service_marketplace`
  - `smoke`
  - `signature_profile`
  - `state`
  - `token`
  - `task_artifacts`
  - `task_payment`
  - `telegram_bridge`
  - `task_lifecycle`
  - `transaction`
  - `validator_lifecycle`
- Regression marker:
  - `Regression: #1828`
  - `Regression: #1981`
  - `Regression: #1983`
  - `Regression: #1985`
  - `Regression: #1987`
  - `Regression: #1989`
  - `Regression: #1991`
  - `Regression: #1993`
  - `Regression: #1995`
  - `Regression: #1997`
  - `Regression: #1999`
  - `Regression: #2001`
  - `Regression: #2003`
  - `Regression: #2005`
  - `Regression: #2007`
  - `Regression: #2009`
  - `Regression: #2011`
  - `Regression: #2013`
  - `Regression: #2015`
  - `Regression: #2017`
  - `Regression: #2019`
  - `Regression: #2021`
  - `Regression: #2023`
  - `Regression: #2025`
  - `Regression: #2027`
  - `Regression: #2029`
  - `Regression: #2031`
  - `Regression: #2033`
  - `Regression: #2035`
  - `Regression: #2037`
  - `Regression: #2039`
  - `Regression: #2041`
  - `Regression: #2043`
  - `Regression: #2045`
  - `Regression: #2047`
  - `Regression: #2049`

## Contributor Entrypoint Matrix

| Contributor need | Entrypoint | Why it exists |
| --- | --- | --- |
| See ownership boundaries across core modules | `docs/architecture/kamn-core-module-map.md#ownership-matrix` | Canonical map for domain ownership and runtime/data-flow responsibilities. |
| Understand high-level runtime path | `docs/architecture/kamn-core-module-map.md#runtime-flow-condensed` | Condensed sequence from identity checks to external receipts. |
| Find exported public API surface | `crates/kamn-core/src/lib.rs` | Canonical `pub mod` and `pub use` inventory for `kamn-core`. |
| Find extracted Kolme scaffold contracts | `crates/kamn-kolme/src/lib.rs` | Canonical crate boundary for runtime-commit codec, transport, finality, and pipeline scaffolding. |
| Run missing-doc drift policy | `scripts/ci/check_kamn_core_missing_docs_policy.sh` | Fail-closed lint allowlist checker for docs hardening. |
| Generate bounded rustdoc artifact evidence | `scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh` | Deterministic rustdoc report/artifact lane used in CI and local checks. |
| Enforce rustdoc artifact policy schema | `scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh` | Validates report schema, digest, artifact path, and runtime budget. |
| Review hardening command surface | `docs/planning/engineering-hardening-wave.md#commands` | Planning baseline for docs hardening and CI contract commands. |
| Review rustdoc publication policy contract | `docs/developer/rustdoc-publishing.md#contract-enforcement` | Contributor-facing rustdoc publication + policy checker contract. |
