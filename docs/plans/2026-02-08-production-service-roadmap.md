# KAMN: From Protocol Spec to Production Service

## Current State (honest assessment)

What you have is **88K lines of Rust that define the rules** — domain types, state machines, validation guards, contract tests — plus a working HTTP/WebSocket client for Kolme blockchain integration and a localhost TCP demo. What you don't have is anything that runs as a service.

There is **zero async code** in the entire codebase. No tokio, no `async fn`, no `.await`. All networking is synchronous blocking I/O. There is no database. There is no HTTP server. There is no P2P layer. Every store is `InMemory*` with no persistence.

---

## Execution Status Update (2026-02-14)

- Phase 1.1 delivered: `kamn-node` now runs with a tokio runtime boundary (Story #2890).
- Phase 1.2 delivered: daemon shutdown now handles real OS signals on tokio signal streams (Story #2895).
- Phase 1.3 in progress:
  - Delivered in this slice: `FileContentAdapter` and `FileDidRegistrationChainAdapter` in `kamn-core` (Task #2901).
  - Live validation lane added for persistence adapter restart and fail-closed checks (Task #2903).
  - Remaining: broader persistence backend consolidation and runtime wiring across additional stores.

---

## Phase 1: Make a Node That Stays Running (Foundation)

### 1.1 — Add an async runtime

This is the single biggest blocker. The entire codebase is synchronous. You need tokio.

- Add `tokio` to `kamn-node/Cargo.toml` with `rt-multi-thread`, `macros`, `net`, `time`, `signal` features
- Convert `fn main()` to `#[tokio::main] async fn main()`
- The existing synchronous domain logic in `kamn-core` does NOT need to change — it's pure computation. Wrap it at the node boundary
- Estimated scope: `kamn-node` only. `kamn-core` stays sync

### 1.2 — Real OS signal handling

`daemon_shutdown.rs` currently simulates signals via CLI tick arguments. Replace with actual signal handling:

- Add `tokio::signal::unix::signal(SignalKind::terminate())` and `ctrl_c()`
- Wire into the existing `evaluate_daemon_completion` logic
- This is ~50 lines of real code wrapping what already exists

### 1.3 — Persistent storage

Every store is in-memory. You need at minimum one real backend. Recommended: **redb** (embedded, zero-config, Rust-native) or **SQLite via rusqlite**.

Stores that need real backends (priority order):

| Store | What it holds | Current impl |
|---|---|---|
| `ReputationStore` | Agent trust scores, endorsements, disputes | `BTreeMap` |
| `ChannelSnapshotStore` | Channel metadata, membership | `InMemoryChannelSnapshotStore` |
| `MessageLifecycleSnapshotStore` | Message status, delivery tracking | `InMemoryMessageLifecycleSnapshotStore` |
| `TaskOperationSnapshotStore` | Task state, dependency graph | `InMemoryTaskOperationSnapshotStore` |
| `DurableGuardSnapshotStore` | Delivery guard nonces, replay protection | `InMemoryDurableGuardSnapshotStore` |
| `ContentStorageAdapter` | Content blobs, CIDs | `InMemoryContentAdapter` |
| `DidRegistry` | Agent DID documents | `InMemoryDidRegistrationChainAdapter` |

The trait interfaces already exist (`ContentStorageAdapter`, `ChannelSnapshotStore`, etc.). You need to implement `SqliteChannelSnapshotStore`, `SqliteMessageLifecycleSnapshotStore`, etc. The `File*` variants exist for some stores but they're single-file JSON dumps — not suitable for concurrent access.

Status update: file-backed adapters now also include content storage and DID chain submission persistence for deterministic restart-safe behavior. Database-backed adapters remain planned follow-up work.

Estimated scope: ~1,500–2,500 lines. One generic key-value store adapter would cover most of these since they all follow put/get/list patterns.

---

## Phase 2: Accept Incoming Connections (Server)

### 2.1 — HTTP/JSON-RPC API server

The node needs to listen for requests. Recommended: **axum** (async, tower-based, lightweight).

Minimum endpoints:

```
POST /v1/messages/send          — Submit a signed message envelope
GET  /v1/messages/{id}          — Get message status
POST /v1/channels/create        — Create a channel
GET  /v1/channels/{id}/messages — List messages in a channel
POST /v1/tasks/create           — Create a task
GET  /v1/tasks/{id}             — Get task state
GET  /v1/agents/{did}           — Get agent reputation/profile
GET  /healthz                   — Health check
GET  /metrics                   — Prometheus metrics
```

All the request validation, envelope parsing, state machine transitions, and response construction logic **already exists** in `kamn-core`. The API layer is mostly wiring — deserialize request, call domain function, serialize response.

Estimated scope: ~2,000–3,000 lines for the server skeleton + route handlers.

### 2.2 — Authentication middleware

The cryptographic primitives exist (`k256` ECDSA signing, DID verification). You need:

- Request signature verification middleware (extract signature from header, verify against sender DID)
- The `EnvelopeProof` validation in `message_envelope.rs` already defines the schema
- Nonce/replay checking via `MessageDeliveryGuards` already works

Estimated scope: ~300–500 lines of axum middleware wrapping existing logic.

### 2.3 — WebSocket server for real-time events

Agents need push notifications (message received, task state changed, payment confirmed). Add WebSocket upgrade support to the axum server:

- Use `axum::extract::ws::WebSocket`
- The `KolmeRuntimeCommitNotificationsConsumer` already defines the event model
- Broadcast events when domain state transitions occur

Estimated scope: ~500–800 lines.

---

## Phase 3: Node-to-Node Communication (P2P)

### 3.1 — libp2p gossip layer

The PRD specifies a triadic network (Processor/Listener/Approver) with gossip. The `PeerLifecycle` state machine exists but has no real transport.

- Add `libp2p` with `gossipsub`, `identify`, `kad` (Kademlia DHT for peer discovery), `noise` (encryption), `yamux` (multiplexing)
- Wire `PeerLifecycleEvent` transitions to actual connection events
- Gossip topics: `messages`, `blocks`, `reputation-updates`
- The `RoleSmokeNetwork` (processor/listener/approver) is the skeleton — it needs real networking underneath

This is the largest single piece of work. Estimated scope: ~4,000–6,000 lines.

### 3.2 — Block production and consensus

The `ProducedBlock`, `BaselineTransaction`, and `TransactionGuards` types exist. The processor role needs to:

- Collect transactions from gossip into a real mempool (currently `Vec<BaselineTransaction>` in memory)
- Produce blocks on demand (the `RoleSmokeNetwork.produce_block()` logic exists)
- Broadcast blocks to listeners/approvers
- Validate incoming blocks against `TransactionGuards`
- Persist committed blocks

This connects to Kolme's block production model. Estimated scope: ~2,000–3,000 lines.

---

## Phase 4: Kolme Blockchain Integration (On-Chain)

### 4.1 — Runtime commit pipeline (partially exists)

The `KolmeRuntimeCommitLiveProvider` HTTP client works. What's missing:

- Automatic transaction submission (currently requires manual CLI invocation)
- Finality polling loop (the `KolmeRuntimeCommitFinalityChecker` exists but must be driven by async task)
- WebSocket reconnection (budget tracking exists, reconnect logic doesn't)
- Block fallback reconciliation on reorgs

Estimated scope: ~1,000–1,500 lines to make the existing client code run continuously.

### 4.2 — DID on-chain registration

`DidRegistry` and `DidRegistrationChainAdapter` traits exist with full lifecycle (create, update, deactivate). The `InMemoryDidRegistrationChainAdapter` needs to be replaced with one that submits DID operations as Kolme transactions.

Estimated scope: ~500–800 lines.

### 4.3 — On-chain message anchoring

Messages are currently processed entirely in-memory. For auditability:

- Hash message envelopes and anchor hashes on-chain via Kolme transactions
- The `MessageLifecycleStore` status transitions (Created -> Signed -> Broadcast -> Included) map directly to on-chain states
- Content stays off-chain; only proof/hash goes on-chain

Estimated scope: ~800–1,200 lines.

---

## Phase 5: SDK and Client Libraries

### 5.1 — Rust SDK (partially exists)

`kamn-sdk` is a scaffold with TCP transport examples. Needs:

- HTTP client wrapping the API from Phase 2
- WebSocket subscription client
- High-level `KamnClient` with `send_message()`, `create_task()`, `check_reputation()`
- Connection management, retry, auth

Estimated scope: ~1,500–2,000 lines.

### 5.2 — Python SDK (partially exists)

`tests/python/test_sdk.py` and the `LiveKAMNClient` class exist with a backend adapter pattern. Needs:

- Real HTTP transport (currently uses mock adapter)
- WebSocket support for events
- PyPI-publishable package

Estimated scope: ~1,000–1,500 lines.

---

## Phase 6: Operational Readiness

### 6.1 — Observability endpoints

`daemon_observability.rs` and `kolme_live_observability.rs` synthesize SLO metrics but only write to stdout. Need:

- Prometheus `/metrics` endpoint (add `prometheus` or `metrics` crate)
- Export the existing metrics (latency_p50, throughput_tps, error_rate_bps, availability_bps) as Prometheus gauges/histograms
- Structured JSON logging already exists (`logging.rs`) — wire it to the async runtime

Estimated scope: ~500–800 lines.

### 6.2 — Configuration management

Currently everything is CLI flags (30+ of them in `cli.rs`). For production:

- TOML/YAML config file support (add `toml` or `serde_yaml`)
- Environment variable overrides (partially exists for signer keys)
- Config validation already exists in `NodeConfig.validate()`

Estimated scope: ~300–500 lines.

### 6.3 — Container and deployment

- Dockerfile (multi-stage Rust build)
- Docker Compose for local multi-node setup (processor + listener + approver)
- Kubernetes manifests or Helm chart
- The `upgrade-rollback-runbook.md` exists but needs real deployment tooling

Estimated scope: ~500 lines of infra config.

---

## Priority Order and Effort Estimate

| Phase | What | Depends On | Estimated Lines | Criticality |
|---|---|---|---|---|
| **1.1** | Tokio async runtime | Nothing | ~200 | BLOCKER |
| **1.2** | Real signal handling | 1.1 | ~50 | HIGH |
| **1.3** | Persistent storage | Nothing | ~2,000 | BLOCKER |
| **2.1** | HTTP API server | 1.1, 1.3 | ~2,500 | BLOCKER |
| **2.2** | Auth middleware | 2.1 | ~400 | HIGH |
| **2.3** | WebSocket server | 2.1 | ~600 | MEDIUM |
| **3.1** | libp2p P2P layer | 1.1 | ~5,000 | HIGH |
| **3.2** | Block production | 3.1, 1.3 | ~2,500 | HIGH |
| **4.1** | Kolme commit pipeline | 1.1 | ~1,200 | MEDIUM |
| **4.2** | DID on-chain | 4.1 | ~600 | MEDIUM |
| **4.3** | Message anchoring | 4.1, 1.3 | ~1,000 | MEDIUM |
| **5.1** | Rust SDK | 2.1 | ~1,500 | MEDIUM |
| **5.2** | Python SDK | 2.1 | ~1,200 | LOW |
| **6.1** | Prometheus metrics | 2.1 | ~600 | HIGH |
| **6.2** | Config file support | Nothing | ~400 | MEDIUM |
| **6.3** | Container/deploy | 2.1 | ~500 | MEDIUM |

**Total estimated new code: ~20,000–25,000 lines of Rust** on top of the existing 88K.

---

## What You Can Skip (YAGNI)

Given the current state, these PRD features should be deferred past MVP:

- **ZK message proofs** — `zk_message_proofs.rs` has 1,000+ lines of evaluation logic but zero actual ZK circuits. Park it.
- **Cross-chain bridges** (Ethereum, Solana, Near) — Focus on Kolme only first
- **Telegram/Discord bridges** — Nice-to-have, not core
- **Service marketplace** — Requires a working economy first
- **Group channel encryption** — Start with direct messages
- **Content replication** — Start with single-node storage

---

## Minimum Viable Production Service

If you do only **Phases 1 + 2 + 6.1**, you get:

- A node that starts, stays running, handles signals
- Persistent state that survives restarts
- An HTTP API that agents can call to send messages, create tasks, check reputation
- Prometheus metrics for monitoring
- All backed by the 88K lines of domain logic and 2,000 tests that already exist

That's roughly **5,000–6,000 lines of new code** and would be a real, deployable service — single-node, no P2P, no blockchain anchoring, but functional. The P2P and Kolme integration (Phases 3 + 4) turn it into the decentralized network the PRD describes.
