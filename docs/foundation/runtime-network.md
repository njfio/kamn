# Runtime Network Contracts (Issues #315 / #317 / #320 / #324 / #319 / #323 / #321 / #322 / #333 / #336)

This document captures the initial runtime-network foundation slice for peer lifecycle and bounded queue behavior in `kamn-core`.

## Scope Delivered
- Added peer lifecycle state machine primitives in `crates/kamn-core/src/runtime.rs`:
  - `PeerLifecycleState`
  - `PeerLifecycleEvent`
  - `PeerLifecycle`
  - `RuntimeLifecycleError`
- Added daemon phase-coordination primitives in `crates/kamn-core/src/runtime_phase_coordination.rs`:
  - `ConstructLockLease`
  - `ConstructLockGuard`
  - `ConstructLockError`
  - `execute_processor_daemon_tick(...)`
  - `ListenerAttestation`
  - `ListenerQuorumInput`
  - `ListenerQuorumDecision`
  - `ListenerQuorumEvaluator`
  - `ListenerQuorumError`
  - `evaluate_daemon_listener_quorum(...)`
  - `ApproverAttestation`
  - `ApproverQuorumInput`
  - `ApproverQuorumDecision`
  - `ApproverQuorumEvaluator`
  - `ApproverQuorumError`
  - `authorize_daemon_outbound_action(...)`
- Added authenticated peer transport framing primitives in `crates/kamn-core/src/runtime.rs`:
  - `AuthenticatedPeerFrame`
  - `AuthenticatedPeerFrameError`
  - `PeerFrameAuthenticator`
- Added bounded FIFO queue primitives in `crates/kamn-core/src/runtime.rs`:
  - `BoundedRuntimeQueue<T>`
  - `RuntimeQueueError`
- Added deterministic queue backpressure primitives in `crates/kamn-core/src/runtime.rs`:
  - `RuntimeBackpressurePolicy`
  - `RuntimeBackpressureInput`
  - `RuntimeBackpressureDecision`
  - `RuntimeBackpressureAction`
  - `RuntimeBackpressureError`
  - `DeterministicBackpressureController`
- Added deterministic proposal-planner primitives in `crates/kamn-core/src/runtime.rs`:
  - `ProposalCandidate`
  - `DeterministicProposalPlanner`
  - `ProposalPlan`
  - `ProposalPlannerError`
- Added recovery/rejoin guard primitives in `crates/kamn-core/src/runtime.rs`:
  - `RejoinAttempt`
  - `RecoveryRejoinGuard`
  - `RecoveryStatus`
  - `RecoveryGuardError`
- Added runtime snapshot persistence/restore guard primitives in `crates/kamn-core/src/runtime.rs`:
  - `RuntimeSnapshot` (version/hash/cursor)
  - `SnapshotRestoreGuard`
  - `SnapshotRestoreError`
  - `RuntimeSnapshotStore`
  - `InMemoryRuntimeSnapshotStore`
  - `FileRuntimeSnapshotStore`
  - `SnapshotStoreError`
- Added watchdog anomaly and deterministic fault simulation primitives in `crates/kamn-core/src/runtime_transport_coordination.rs`:
  - `WatchdogAnomalyKind`
  - `WatchdogAnomalySeverity`
  - `WatchdogAnomalyWatchInput`
  - `WatchdogAnomalyEvaluator`
  - `WatchdogAnomalyError`
  - `evaluate_daemon_watchdog_anomaly(...)`
  - `NetworkFaultSimulationInput`
  - `NetworkFaultSimulationReport`
  - `NetworkFaultSimulationError`
  - `DeterministicNetworkFaultSimulator`
  - `simulate_daemon_network_fault(...)`
- Kept runtime role wiring behavior unchanged and covered by existing tests.

## Node CLI Recovery-Check Mapping
- `kamn-node --runtime-mode recovery-check` maps directly to `RecoveryRejoinGuard` evaluation flow.
- Command example:
  - `kamn-node --role processor --runtime-mode recovery-check`
  - `kamn-node --role processor --runtime-mode recovery-check --expected-state-version 42 --expected-state-hash state-42 --rejoin-attempt node-a|42|state-42|resume-1`
- CLI argument mapping:
  - `--expected-state-version` -> `RecoveryRejoinGuard::new(expected_state_version, ...)`
  - `--expected-state-hash` -> `RecoveryRejoinGuard::new(..., expected_state_hash)`
  - `--rejoin-attempt <node-id|state-version|state-hash|resume-token>` -> `RejoinAttempt::new(...)`
- Deterministic output mapping:
  - `RecoveryStatus::RejoinAccepted` -> `rejoin-accepted`
  - `RecoveryStatus::CatchUpRequired { from_version, to_version }` -> `catch-up-required:<from_version>-><to_version>`
- Error mapping:
  - malformed rejoin-attempt argument -> `ConfigError::InvalidRejoinAttemptArgument`
  - replay/version/hash mismatch from guard evaluation -> `ConfigError::RuntimeRecovery`

## Node CLI Daemon Lifecycle Mapping
- `kamn-node --runtime-mode daemon` can optionally evaluate peer lifecycle transitions.
- CLI argument mapping:
  - `--daemon-peer-id` -> `PeerLifecycle::new(peer_id)`
  - `--daemon-lifecycle-event <event>` -> `PeerLifecycle::transition(event)` in input order
- Supported daemon lifecycle events:
  - `start-connect`
  - `handshake-succeeded`
  - `heartbeat-missed`
  - `heartbeat-restored`
  - `disconnect`
  - `rejoin`
- Deterministic daemon lifecycle outputs:
  - `daemon_peer_id`
  - `daemon_peer_lifecycle_final_state`
  - `daemon_peer_lifecycle_applied_events`
- Error mapping:
  - invalid lifecycle event argument -> `ConfigError::InvalidDaemonLifecycleEvent`
  - invalid transition from lifecycle state machine -> `ConfigError::RuntimeDaemonLifecycle`

## Bridge Quorum Runtime Mapping
- Listener and approver bridge quorum runtime contracts are documented in:
  - `docs/foundation/bridge-quorum-runtime.md`
- Inbound bridge event handling maps listener attestation normalization and quorum evaluation before acceptance.
- Outbound bridge authorization maps approver attestation threshold validation before action dispatch.
- Bridge runtime guard outcomes are deterministic and include replay, malformed payload, and under-quorum rejection semantics.

## Peer Lifecycle Rules
- `PeerLifecycle` starts in `Disconnected`.
- Valid transitions:
  - `Disconnected` + `StartConnect` -> `Connecting`
  - `Disconnected` + `Rejoin` -> `Connecting`
  - `Connecting` + `HandshakeSucceeded` -> `Active`
  - `Active` + `HeartbeatMissed` -> `Degraded`
  - `Degraded` + `HeartbeatRestored` -> `Active`
  - `Connecting|Active|Degraded` + `Disconnect` -> `Disconnected`
- Invalid transitions return `RuntimeLifecycleError::InvalidTransition`.
- Empty peer IDs are rejected with `RuntimeLifecycleError::InvalidPeerId`.

## Authenticated Peer Transport Framing Rules
- `AuthenticatedPeerFrame` enforces deterministic wire fields:
  - non-empty `frame_id`
  - valid `kamn:did:*` sender and recipient peer DIDs
  - positive nonce
  - non-empty payload and signature
  - scalar fields must not include `|`, newline, or carriage-return delimiters
- Deterministic wire format:
  - `frame|<frame_id>|<sender_peer_did>|<recipient_peer_did>|<nonce>|<payload>|<signature>`
- Signatures are validated against deterministic baseline profile inputs:
  - sender DID
  - nonce
  - recipient DID (bound into signature context)
  - payload
- `PeerFrameAuthenticator` enforces:
  - local recipient DID match
  - sender allowlist membership
  - monotonic sender nonce progression (strictly increasing)
- Regression contract:
  - forged signatures are rejected (`Regression: #618`)
  - unauthorized sender DIDs are rejected (`Regression: #618`)
  - replayed sender nonces are rejected (`Regression: #618`)

## Deterministic Input Mutation Fail-Closed Rules
- Envelope mutation suite must deterministically cover:
  - malformed payload shape/identity cases
  - truncated scalar/list payload cases
  - tampered proof-binding cases
- DID mutation suite must deterministically cover:
  - normalization drift cases
  - encoding/character drift cases
  - method mismatch prefix cases
- All mutation corpus entries must fail closed with explicit typed errors and stable reason strings.
- Fast contract lane command:
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh`
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh --target envelope --output-json /tmp/input-mutation-envelope-smoke-report.json`
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh --target did --output-json /tmp/input-mutation-did-smoke-report.json`
- Coverage-guided parser fuzz command:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json`
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --target envelope --output-json /tmp/input-mutation-coverage-guided-envelope-report.json`
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --target did --output-json /tmp/input-mutation-coverage-guided-did-report.json`
- Processor proof admission guard lane command:
  - `bash scripts/runtime/run_processor_proof_admission_contract_lane.sh`
- Processor proof admission fail-closed rules:
  - message-id mismatch
  - payload commitment mismatch
  - invalid proof format
  - replayed artifact id
- Input mutation replay metadata schema:
  - `kamn.runtime.input-mutation-replay-metadata.v1`
- Input mutation seed corpus keys:
  - `input_mutation_envelope_seed:v1`
  - `input_mutation_did_seed:v1`
- Coverage-guided input mutation replay metadata schema:
  - `kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1`
- Coverage-guided input mutation replay artifact key:
  - `input_mutation_coverage_guided_replay:v1`
- Coverage-guided minimizer marker:
  - `minimal_failing_seed_prefix`
- Deep coverage-guided parser fuzz command remains local-only by default:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh`
- local-only deep routing marker:
  - `runtime_input_mutation_coverage_guided_deep=skipped_local_only`

## Deterministic Concurrency Harness Rules
- Shared-state mutation harness in `crates/kamn-core/tests/concurrency_state_mutation.rs` must enforce:
  - single-winner task accept races under parallel contender workloads
  - deterministic duplicate-submit rejection outcomes under concurrent submit attempts
  - deterministic peer lifecycle phase summaries across replayed concurrent transitions
- Contract/deep lane commands:
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh`
  - `bash scripts/runtime/run_concurrency_state_mutation_deep_lane.sh`
- Regression policy:
  - concurrent accept races must never admit multiple winners (`Regression: #844`).

## Combined Invariant/Fuzz/Concurrency Contract Rules
- Combined lane command:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Evidence policy checker:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`
- Report schema:
  - `kamn.runtime.invariant-fuzz-concurrency-contract-report.v1`
- Required report fields:
  - `status`
  - `property_lane_status`
  - `fuzz_lane_status`
  - `concurrency_lane_status`
  - `elapsed_seconds`
  - `max_seconds`
  - `reason_codes`
- Runtime budget:
  - `KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS=180` (default)
- Regression policy:
  - reason-code policy remains fail-closed with stable `["none"]` success marker (`Regression: #897`).

## Localhost Signed Integration Evidence Key Contract Rules
- Harness command:
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario success --output-json /tmp/localhost-signed-harness-success.json`
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario session-expired --output-json /tmp/localhost-signed-harness-session-expired.json`
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario replay-nonce --output-json /tmp/localhost-signed-harness-replay.json`
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario admission-guards --output-json /tmp/localhost-signed-harness-admission.json`
- Contract lane command:
  - `bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed-integration-contract-report.json`
- Evidence policy checker:
  - `bash scripts/sdk/check_localhost_signed_integration_evidence_policy.sh --report-file /tmp/localhost-signed-integration-contract-report.json`
- Contract report schema:
  - `kamn.sdk.localhost-signed.integration-contract.v1`
- Deterministic fixture corpus:
  - `fixtures/runtime/localhost_signed_integration_cases.json`
  - `kamn.sdk.localhost-signed.integration-fixtures.v1`
- Deterministic keys:
  - `localhost_signed_integration_contract:v1`
  - `localhost_signed_integration:success:v1`
  - `localhost_signed_integration:signature-mismatch:v1`
  - `localhost_signed_integration:malformed-signature:v1`
  - `localhost_signed_integration:timeout:v1`
  - `localhost_signed_integration:session-expired:v1`
  - `localhost_signed_integration:replay-nonce:v1`
  - `localhost_signed_integration:admission-guards:v1`
- Deterministic reason markers:
  - `signature_mismatch_detected`
  - `malformed_signature_detected`
  - `listener_timeout_detected`
  - `session_expired_detected`
  - `replay_nonce_detected`
  - `session_admission_guards_detected`
  - `expiry_guard_status=pass`
  - `admission_reason_codes=["stale_session_detected","unauthorized_sender_detected","malformed_payload_detected"]`
  - `final_decision=GO` for pass reports
- Session nonce/expiry guard rules:
  - payload must include `session_epoch_seconds`
  - listener enforces `--session-max-age-seconds` fail-closed admission checks
  - stale session epochs are rejected before delivery acceptance
- Signature failure taxonomy:
  - malformed signature encoding/profile -> `malformed_signature_detected`
  - deterministic signature mismatch against expected fields -> `signature_mismatch_detected`
- Regression policy:
  - deterministic evidence keys and reason keys remain fail-closed (`Regression: #899`).
  - replay nonce and session admission guards remain fail-closed (`Regression: #1382`).

## Live Transport Replay/Tamper Evidence Contract Rules
- Evidence generator command:
  - `bash scripts/sdk/generate_live_transport_replay_tamper_evidence_bundle.sh --output-file /tmp/live-transport-replay-tamper-bundle.json --transport-lane-id localhost-signed-integration --message-id msg-001 --from-did kamn:did:agent:sender-1 --to-did kamn:did:agent:listener-1 --nonce 41 --signature-status valid --replay-detected false --tamper-detected false --ci-fast-gate PASS`
- Evidence policy checker:
  - `bash scripts/sdk/check_live_transport_replay_tamper_policy.sh --bundle-file /tmp/live-transport-replay-tamper-bundle.json`
- Contract lane command:
  - `bash scripts/sdk/run_live_transport_replay_tamper_contract_lane.sh --output-report /tmp/live-transport-replay-tamper-contract-report.json`
- Fast lane wrapper:
  - `bash scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh --output-report /tmp/live-transport-replay-tamper-fast-report.json`
- Deep lane wrapper:
  - `bash scripts/sdk/run_live_transport_replay_tamper_deep_lane.sh --output-report /tmp/live-transport-replay-tamper-deep-report.json`
- Evidence schema:
  - `kamn.sdk.live-transport-replay-tamper-evidence.v1`
- Deterministic replay/tamper reason codes:
  - `none`
  - `signature_mismatch_detected`
  - `malformed_signature_detected`
  - `replay_nonce_detected`
  - `tamper_payload_detected`
  - `ci_fast_gate_failed`
- Regression policy:
  - tampered final decision, reason-key drift, and replay/tamper marker drift fail closed (`Regression: #1380`).
- Lane mode markers:
  - `lane_mode=fast`
  - `lane_mode=deep`
  - `deep_no_go_status=verified` (deep lane only)

## Live Transport Demo Failure Taxonomy and Troubleshooting
- Primary transport failure taxonomy markers:
  - `signature_mismatch_detected`
  - `malformed_signature_detected`
  - `listener_timeout_detected`
  - `session_expired_detected`
  - `replay_nonce_detected`
  - `session_admission_guards_detected`
  - `tamper_payload_detected`
  - `ci_fast_gate_failed`
- Deterministic troubleshooting command loop:
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario malformed-signature`
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario session-expired`
  - `bash scripts/sdk/run_localhost_signed_integration_harness.sh --scenario replay-nonce`
  - `bash scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh --output-report /tmp/live-transport-replay-tamper-fast-report.json`
  - `bash scripts/sdk/check_live_transport_replay_tamper_policy.sh --bundle-file /tmp/live-transport-replay-tamper-fast-report.json`
  - `bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed-integration-contract-report.json`
  - `bash scripts/sdk/check_localhost_signed_integration_evidence_policy.sh --report-file /tmp/localhost-signed-integration-contract-report.json`
- Artifact checkpoints:
  - `/tmp/live-transport-replay-tamper-fast-report.json`
  - `/tmp/localhost-signed-integration-contract-report.json`
- Failure triage guidance:
  - `signature_mismatch_detected`/`malformed_signature_detected`: verify sender DID, nonce binding, payload canonicalization, and signature envelope encoding.
  - `listener_timeout_detected`: verify listener endpoint reachability and increase harness `--timeout-seconds` only for local diagnostics.
  - `session_expired_detected`/`session_admission_guards_detected`: confirm `session_epoch_seconds`, sender authorization, and payload shape before rerun.
  - `replay_nonce_detected`: reset/reseed nonce progression; ensure monotonic nonce sequence across retries.
  - `tamper_payload_detected`/`ci_fast_gate_failed`: treat as NO-GO until payload integrity and fast-gate contract status are restored.

## Queue Guard Rules
- `BoundedRuntimeQueue<T>` is FIFO and preserves insertion order.
- Capacity must be greater than zero.
- Overflow does not evict existing entries; new enqueue attempts fail with:
  - `RuntimeQueueError::Overflow { capacity, attempted_len }`
- Zero-capacity queues fail with:
  - `RuntimeQueueError::InvalidCapacity { capacity: 0 }`

## Deterministic Backpressure and Stale-Peer Queue Rules
- `RuntimeBackpressurePolicy` thresholds are bounded and ordered:
  - `slow_threshold_per_mille` must be in `1..=1000`
  - `reject_threshold_per_mille` must be in `1..=1000`
  - slow threshold must be strictly lower than reject threshold
- `RuntimeBackpressureInput` requires:
  - valid `kamn:did:*` peer identifier
  - queue capacity greater than zero
  - queue depth less than or equal to queue capacity
- `DeterministicBackpressureController` decisions are deterministic for identical inputs:
  - `Accept`
  - `SlowProducer`
  - `RejectNewEnqueue`
  - `PurgeStalePeerQueue`
- Backpressure action mapping:
  - disconnected peers with pending queue entries can be forced to `PurgeStalePeerQueue` when policy enables stale-peer purge.
  - utilization above reject threshold yields `RejectNewEnqueue`.
  - utilization above slow threshold yields `SlowProducer`.
- Regression contract:
  - queue depth above capacity is rejected (`Regression: #618`)
  - stale disconnected peer queue must purge deterministically (`Regression: #618`)

## Scheduler Determinism Rules
- `ProposalCandidate` requires non-empty:
  - candidate ID
  - sender DID
  - state hash
- Candidate nonce must be positive.
- `DeterministicProposalPlanner` rejects candidate sets when:
  - duplicate candidate IDs are present (`ProposalPlannerError::DuplicateCandidateId`)
  - candidate state hash differs from planner expectation (`ProposalPlannerError::StaleStateHash`)
- Valid candidate sets are ordered deterministically by:
  - nonce ascending
  - sender DID ascending
  - candidate ID ascending

## Recovery and Rejoin Guard Rules
- `RejoinAttempt` requires non-empty:
  - node ID
  - state hash
  - resume token
- Rejoin state version must be positive.
- `RecoveryRejoinGuard` behavior:
  - lagging state versions produce `CatchUpRequired { from_version, to_version }`
  - newer-than-expected state versions are rejected (`StateVersionMismatch`)
  - matching version with mismatched state hash is rejected (`StateHashMismatch`)
  - replayed resume tokens are rejected (`ReplayResumeToken`)
  - matching version/hash with unique resume token is accepted (`RejoinAccepted`)

## Snapshot Persistence and Restore Contract Rules
- Runtime snapshot persistence enforces strict continuity across:
  - state version
  - state hash
  - snapshot cursor/checkpoint
- `RuntimeSnapshot::with_cursor(...)` requires:
  - positive state version
  - non-empty hash with no metadata delimiter (`|`)
  - positive cursor
- `SnapshotRestoreGuard::with_expected_cursor(...)` enforces deterministic restore identity across version/hash/cursor.
- File-backed snapshot entries serialize as:
  - `<state-version>|<state-hash>|<cursor>`
  - legacy `<state-version>|<state-hash>` lines remain readable and are mapped to cursor=`state_version`.
- Typed persistence guard failures:
  - `SnapshotStoreError::StateVersionRegression`
  - `SnapshotStoreError::CursorRegression`
  - `SnapshotStoreError::StaleStateHash`
- Corrupt or stale metadata restores are repaired deterministically by truncating the invalid suffix.
- Regression contract:
  - stale version/cursor/hash metadata are rejected (`Regression: #617`)
  - cursor mismatch restore attempts are rejected (`Regression: #617`)

## Deterministic Fault Simulation Harness Rules
- `NetworkFaultSimulationInput` requires:
  - non-empty `sample_id`
  - non-empty `peer_id`
  - queue capacity greater than zero
  - watchdog-compatible delivery/peer sample values
- `DeterministicNetworkFaultSimulator` executes deterministic simulation flow:
  - lifecycle bootstrap: `StartConnect` -> `HandshakeSucceeded`
  - degraded lifecycle projection when `healthy_peers < active_peers`
  - bounded queue saturation accounting via `queue_overflow_attempts`
  - watchdog anomaly classification via `WatchdogAnomalyEvaluator`
- `simulate_daemon_network_fault(...)` provides daemon wrapper parity for simulator execution.
- Output contract fields:
  - `final_lifecycle_state`
  - `queue_overflow_attempts`
  - `watchdog_kind`
  - `watchdog_severity`
  - delivery/liveness per-mille ratios

## Kolme Notifications Websocket Consumer Contract Rules
- Notifications endpoint contract:
  - `GET /notifications` websocket stream from Kolme API server.
  - known notification variants are `NewBlock`, `FailedTransaction`, and `LatestBlock`.
- `KolmeRuntimeCommitNotificationsConsumer` in `crates/kamn-core/src/kolme_runtime_commit.rs` enforces deterministic behavior:
  - `NewBlock` events require `txhash` and map to `finality=Final`.
  - `FailedTransaction` events require `txhash` and map to `finality=Failed`.
  - `LatestBlock` events are decoded for watermark tracking and do not emit commit receipts.
- Reconnect policy:
  - dropped or failed websocket connections trigger bounded reconnect attempts.
  - retry exhaustion fails closed with deterministic reason:
    - `notification reconnect attempts exhausted after <retries> retries`
- Validation commands:
  - `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
  - `bash scripts/kolme/run_notifications_consumer_contract_lane.sh`
  - `bash scripts/kolme/test_run_notifications_consumer_contract_lane.sh`
- Runtime budget:
  - `KAMN_KOLME_NOTIFICATIONS_CONSUMER_MAX_SECONDS=60` (default)
- Regression policy:
  - unsupported notification variants and reconnect-budget drift remain fail-closed (`Regression: #1463`).

## Kolme Block Fallback Finality Reconciliation Contract Rules
- Block fallback endpoint contract:
  - `GET /block/{height}` lookup is used when notification windows are missed.
  - deterministic payload fields: `provider`, `block_height` (or `height`), optional `tx_hashes`, optional `failed_tx_hashes`.
- `KolmeRuntimeCommitBlockFallbackReconciler` in `crates/kamn-core/src/kolme_runtime_commit.rs` enforces deterministic behavior:
  - lookup window must be bounded by `max_block_lookups`.
  - provider value must match expected provider.
  - response block height must match requested lookup height.
  - first matching `tx_hashes` entry maps to `finality=Final`.
  - matching `failed_tx_hashes` entry maps to `finality=Failed`.
- Fail-closed policy:
  - stale lookup windows above max budget fail closed.
  - malformed block payloads and mismatched response heights fail closed.
  - unresolved txhash across bounded lookup window fails closed.
- Validation commands:
  - `cargo test -p kamn-core --test kolme_runtime_commit_block_fallback`
  - `bash scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh`
  - `bash scripts/kolme/test_run_block_fallback_reconciliation_contract_lane.sh`
- Runtime budget:
  - `KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS=75` (default)
- Regression policy:
  - stale-window and block-height mismatch drift remain fail-closed (`Regression: #1464`).

## Test Coverage Mapping
- Unit:
  - invalid transition checks
  - empty peer ID and zero-capacity queue rejection
  - invalid peer-frame wire payload and delimiter rejection
  - deterministic signature mismatch rejection
  - deterministic malformed/truncated/tampered envelope mutation rejection
  - deterministic DID normalization/encoding/method mismatch mutation rejection
  - deterministic concurrency replay fixture validity checks
  - invalid backpressure threshold ordering and queue-depth validation rejection
  - empty proposal candidate ID rejection
  - empty resume-token rejoin attempt rejection
  - snapshot cursor/hash validation and continuity regression checks
  - websocket notification variant decode and txhash extraction invariants
  - block fallback lookup-window and response-height guard validation
- Functional:
  - peer lifecycle connect/degrade/recover/disconnect flow
  - authenticated peer-frame wire roundtrip and signature verification
  - envelope mutation corpus fail-closed classification by malformed/truncated/tampered classes
  - DID mutation corpus fail-closed classification by normalization/encoding/method mismatch classes
  - deterministic multi-thread task accept replay fixture invariant checks
  - deterministic queue saturation backpressure classification
  - planner deterministic ordering contract
  - rejoin acceptance with matching snapshot
  - snapshot recovery truncation with stale metadata suffix
  - websocket reconnect-after-close receipt continuation behavior
  - missed-notification block fallback reconciliation to final/failed receipts
- Integration:
  - bounded FIFO queue behavior under capacity
  - inbound peer-frame authenticator monotonic nonce acceptance flow
  - stale disconnected peer queue purge decision mapping
  - queue-drain to planner ordering preservation
  - lagging-node catch-up guidance output
  - daemon network-fault simulation with queue-overflow/degradation reporting
  - deterministic runtime mutation contract lane command coverage
  - deterministic concurrency replay summaries across peer lifecycle phase transitions
  - combined invariant/fuzz/concurrency lane + policy checker contract coverage
  - runtime snapshot contract lane wiring and docs mapping checks
  - notifications consumer websocket handshake + text-frame decode with mock endpoint
  - block fallback reconciler with mock block API and HTTP transport wiring
- Regression:
  - rejoin without disconnect is rejected (`Regression: #324`)
  - queue overflow rejects new event (`Regression: #324`)
  - duplicate candidate ID is rejected (`Regression: #323`)
  - stale state hash is rejected (`Regression: #323`)
  - rejoin replay token is rejected (`Regression: #322`)
  - rejoin state hash mismatch is rejected (`Regression: #322`)
  - CLI recovery-check replay/version/hash mismatch rejection (`Regression: #336`)
  - daemon lifecycle invalid transition rejection (`Regression: #349`)
  - listener attestation replay rejection (`Regression: #371`)
  - outbound under-quorum rejection (`Regression: #372`)
  - network fault simulation censorship critical-boundary guard (`Regression: #618`)
  - forged or unauthorized peer frame rejection (`Regression: #618`)
  - replayed peer-frame nonce rejection (`Regression: #618`)
  - queue depth above capacity is rejected (`Regression: #618`)
  - stale disconnected peer queue purge decision remains deterministic (`Regression: #618`)
  - snapshot stale metadata rejection (`Regression: #617`)
  - snapshot restore cursor mismatch rejection (`Regression: #617`)
  - concurrent accept races never allow multiple winners (`Regression: #844`)
- deterministic envelope/DID mutation fail-closed reasons remain stable (`Regression: #843`)
- deep coverage-guided parser fuzz remains local-only and excluded from `ci-fast-gate` (`Regression: #2693`)
  - processor proof admission message/commitment/replay/format guards remain fail-closed (`Regression: #995`)
  - task/escrow/peer lifecycle generated invariant lanes remain deterministic (`Regression: #842`)
  - invariant/fuzz/concurrency combined lane reason-code policy remains fail-closed (`Regression: #897`)
  - notifications websocket variant/reconnect fail-closed contract remains stable (`Regression: #1463`)
  - block fallback stale-window/height-mismatch fail-closed contract remains stable (`Regression: #1464`)
- Performance:
  - bounded PR-lane runtime backpressure evaluation budget check
  - bounded PR-lane authenticated peer-frame validation budget check
  - bounded PR-lane deterministic fault simulation budget check
  - bounded PR-lane envelope mutation validation budget check
  - bounded PR-lane DID mutation validation budget check
  - bounded PR-lane concurrency mutation validation budget check
  - bounded PR-lane combined invariant/fuzz/concurrency validation budget check
  - bounded PR-lane snapshot recovery budget check
  - bounded PR-lane block fallback reconciliation budget check
  - scheduled chaos lane stress hook (`--ignored`)
  - scheduled concurrency stress deep lane hook (`--ignored`)
  - scheduled snapshot recovery deep lane stress hook (`--ignored`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core runtime::tests::
cargo test -p kamn-core runtime::tests::functional_runtime_backpressure_classifies_queue_saturation
cargo test -p kamn-core runtime::tests::regression_runtime_backpressure_rejects_capacity_overflow_sample
cargo test -p kamn-core runtime::tests::functional_authenticated_peer_frame_roundtrips_wire_and_signature
cargo test -p kamn-core runtime::tests::regression_forged_or_unauthorized_peer_frame_is_rejected
cargo test -p kamn-core network_fault_simulation
cargo test -p kamn-core snapshot_store
cargo test -p kamn-core --test runtime_network_docs
cargo test -p kamn-core --test bridge_quorum_runtime_docs
cargo test -p kamn-core --test kolme_runtime_commit_notifications
cargo test -p kamn-core --test kolme_runtime_commit_block_fallback
cargo test -p kamn-core --test message_envelope_fuzz_smoke functional_envelope_mutation_suite_covers_malformed_truncated_and_tampered_classes -- --exact
cargo test -p kamn-core --test did_fuzz_smoke functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes -- --exact
bash scripts/runtime/run_input_mutation_contract_lane.sh
bash scripts/runtime/run_processor_proof_admission_contract_lane.sh
cargo test -p kamn-core --test concurrency_state_mutation functional_task_accept_concurrency_replay_fixture_preserves_invariants -- --exact
cargo test -p kamn-core --test concurrency_state_mutation integration_peer_lifecycle_concurrency_replay_is_deterministic_across_rounds -- --exact
bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh
cargo test -p kamn-core --test task_state_machine task_lifecycle_property_generated_sequences_preserve_transition_contracts -- --exact
cargo test -p kamn-core --test escrow_lifecycle escrow_property_generated_action_sequences_preserve_amount_and_status_invariants -- --exact
cargo test -p kamn-core --test runtime_peer_lifecycle peer_lifecycle_property_generated_event_sequences_match_transition_contract -- --exact
bash scripts/runtime/run_lifecycle_property_contract_lane.sh
bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json
bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json
cargo test -p kamn-node --test node_runtime_cli_docs
cargo test -p kamn-node regression_runtime_daemon_rejects_invalid_lifecycle_transition
bash scripts/runtime/run_runtime_snapshot_contract_lane.sh
bash scripts/kolme/run_notifications_consumer_contract_lane.sh
bash scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh
```

Scheduled deep-lane command:

```bash
cargo test -p kamn-core performance_network_fault_simulation_chaos_lane_stress -- --ignored
bash scripts/runtime/run_concurrency_state_mutation_deep_lane.sh
bash scripts/runtime/run_runtime_snapshot_deep_lane.sh
```

Then run strict lint/format gates:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```
