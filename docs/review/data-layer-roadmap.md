# KAMN Data Layer: What Needs to Happen

**As of:** R44 review, commit `92c9958c` (2026-02-19)

---

## Execution Track (R27.45)

- Infrastructure activation epic: `#5247`
- Phase 1 story (PostgreSQL + RLS foundation): `#5248`
- Completed bootstrap task: `#5255`
- Completed bridge task: `#5257`
- Current execution-adapter task: `#5259`
- Activation plan: `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`

Current phase status:
- Phase 1 bootstrap migration scaffolding: Implemented in `#5255` (merged)
- Phase 1 repository bridge contracts and RLS projection: Implemented in `#5257` (merged)
- Phase 1 live sqlx execution adapter + migration runner: Implemented in `#5259` (pending merge)
- Phases 2-6: Planned with dependency order captured in `#5249`..`#5254`

---

## Current State

The data layer consists of 15 modules (9,642 lines) in `kamn-core` implementing **pure domain contracts** — deterministic validators, state machines, and decision logic with zero I/O. Every module has passing integration tests. The contract layer is complete and well-tested.

What does **not** exist is the persistence and infrastructure layer that would make these contracts operational against real databases and services.

### What's Built (Contract Layer)

| Module | Lines | What It Does |
|--------|-------|-------------|
| M0 | 533 | Envelope record derivation, append-only hash-chain verification |
| M1 | 803 | Merkle tree assembly, inclusion-proof generation/verification, Kolme anchoring worker |
| M2 | 902 | DID session issuance, ABAC authorization, RLS policy templates, audit hash-chain |
| M3 | 694 | Blind-index token derivation, exact-match search contracts |
| M4 | 1,081 | 7-state escrow lifecycle, dispute-aware visibility, settlement evidence reconciliation |
| M5 | 1,097 | Embedding registry, cosine-similarity ranking, anomaly scoring, recall drift detection |
| M6 | 611 | Owner-scoped graph nodes/edges, trust propagation ranking, AGE-compatible export |
| M7 | 775 | Telemetry ingest, hourly/daily rollups, billing reconciliation |
| M8 | 607 | 5-class retention model, legal-hold precedence, crypto-shredding eligibility |
| M9 | 992 | Dispatch ACKs, backpressure escalation, scoped presence, anti-spam decisions |
| M10 | 656 | Monthly partition lifecycle, archival eligibility, recovery planning |
| M11 (hardening) | 319 | Security/chaos/performance scenario matrix, operator readiness |
| M11 (closure) | 127 | Release acceptance gate composing hardening + conformance evidence |
| PRD conformance | 294 | Critical-scenario (62-71) pass/fail matrix |
| Shell-neutral policy | 151 | Shell/Rust ratio budget enforcement |

### What's Not Built (Infrastructure Layer)

| Component | PRD Spec | Current State |
|-----------|----------|---------------|
| PostgreSQL persistence | 6 core tables, 15+ indexes, RLS policies | **Nothing** — no PG driver, no schema, no SQL |
| Envelope encryption | AES-256-GCM + X25519 key wrapping | **Nothing** — contracts reference crypto but don't implement it |
| Kolme chain integration | Real `KolmeRuntimeCommitClient` submitting merkle roots | **Trait only** — M1 accepts a pluggable client, no real implementation |
| pgvector semantic search | Embedding storage + cosine similarity queries | **Nothing** — M5 does in-memory ranking only |
| Apache AGE graph queries | Trust propagation, relationship traversal | **Nothing** — M6 exports AGE-compatible format but no DB connection |
| TimescaleDB telemetry | Hypertable partitioning, continuous aggregates | **Nothing** — M7 does in-memory aggregation only |
| WebSocket/SSE delivery | Real-time message push to recipients | **Nothing** — M9 models backpressure but no delivery infrastructure |
| Object storage (S3/GCS) | Partition archival exports | **Nothing** — M10 references `archived_object_uri` but no I/O |
| Scheduled jobs | Merkle batching, crypto-shredding, partition archival | **Nothing** |

The **only** persistent storage in the entire project is `SqliteStoreBackend` — a simple namespace/key/value store used for node state snapshots. No relational schema, no message tables, no query infrastructure.

---

## What Needs to Happen

### Phase 1: PostgreSQL Foundation

**Goal:** Messages can be stored, retrieved, and verified against the contract layer.

#### 1.1 Add PostgreSQL Driver

Add `sqlx` with the `postgres` and `runtime-tokio` features to `kamn-core/Cargo.toml`. This is the same driver Kolme itself uses, keeping the stack consistent.

#### 1.2 Create Schema Migrations

Implement the PRD section 5 schema as SQL migration files. The core tables are:

- **`messages`** — 25-column append-only table with envelope ciphertext, wrapped keys, hash-chain pointer, merkle batch FK, blind indexes, shredding marker, retention class
- **`merkle_batches`** — Batch metadata + Kolme tx reference
- **`did_registry`** — DID lifecycle with signing/encryption keys, capabilities, delegation
- **`escrows`** — 7-state lifecycle with auditor threshold config
- **`key_rotation_log`** — Key lifecycle audit trail
- **`access_log`** — Query audit trail

Plus 15+ indexes (B-tree for DID lookups, GIN for blind indexes and capabilities, partial indexes for non-shredded messages).

Use versioned `.sql` files under a `migrations/` directory with deterministic runtime execution in the PostgreSQL adapter.

#### 1.3 Implement Repository Layer

Create a `data_layer_persistence` module (or set of modules) that bridges the contract layer to PostgreSQL:

```
Insert flow:  Input → M0 contract validation → M2 authorization check → SQL INSERT
Query flow:   Request → M2 session/ABAC check → SQL SELECT → M0 hash-chain verification
Search flow:  Query → M3 blind-index derivation → SQL GIN lookup → M2 scope filtering
```

Each contract module's types become the validated inputs/outputs for database operations. The repository layer handles connection pooling, transactions, and error mapping.

#### 1.4 Implement RLS Policies

M2 already generates PostgreSQL RLS template SQL. The persistence layer needs to:
- Enable RLS on `messages`, `escrows`, and related tables
- Apply M2's generated policies
- Set the `kamn.requester_did` session variable before each query

---

### Phase 2: Cryptographic Pipeline

**Goal:** Messages are encrypted at rest, CEK wrapping works end-to-end.

#### 2.1 Envelope Encryption

Implement the PRD section 4.2 protocol:
- AES-256-GCM content encryption with random CEK per message
- X25519-HKDF key wrapping for each recipient (agent, owner, optionally escrow auditor)
- zstd compression with dictionary training
- SHA-256 content hashing for the append-only store

This likely belongs in a `crypto_pipeline` module that calls into existing `direct_message_crypto` and `group_channel_crypto` modules.

#### 2.2 Blind Index Generation

Implement the PRD section 4.5 blind index protocol:
- HMAC-SHA256 with per-owner blind index key
- Normalization (M3's `normalize_blind_index_value()` already exists)
- Storage as JSONB in the `blind_indexes` column

---

### Phase 3: Kolme Chain Integration

**Goal:** Merkle roots are periodically anchored to Kolme; inclusion proofs are verifiable.

#### 3.1 Real KolmeRuntimeCommitClient

M1 already defines the `KolmeRuntimeCommitClient` trait. Implement a production version that:
- Connects to a Kolme node
- Submits merkle root transactions
- Tracks confirmation status (pending → submitted → confirmed)
- Records `kolme_tx_hash` and `kolme_block_height` in `merkle_batches`

#### 3.2 Batch Scheduler

Implement the periodic batching job (PRD defaults: 1,000 messages or 60 seconds, whichever comes first):
- Collect unbatched messages
- Call M1 to assemble the merkle tree
- Submit root to Kolme via the real client
- Update `merkle_batch_id` and `merkle_leaf_index` on each message row

---

### Phase 4: PostgreSQL Extensions

**Goal:** Semantic search, graph queries, and time-series analytics are operational.

#### 4.1 pgvector (Semantic Search)

- Enable the `pgvector` extension
- Add an `embeddings` table with `vector(N)` column
- Implement embedding pipeline: owner decrypts message → generates embedding → stores via M5 registry contract → inserts into pgvector
- Implement cosine-similarity search using M5's query contracts as the API layer
- Honor M5's privacy modes (OwnerSideEncrypted vs ServerSidePlaintextOptIn)

#### 4.2 Apache AGE (Graph)

- Enable the `age` extension
- Create a graph schema for M6's 5 node kinds and 7 edge relationships
- Implement graph write path: metadata events → M6 contract validation → AGE `CREATE` via Cypher
- Implement trust propagation queries using M6's ranking contracts
- M6 already exports AGE-compatible portable format — use this for bulk import

#### 4.3 TimescaleDB (Telemetry)

- Enable the `timescaledb` extension
- Create hypertables for telemetry points (M7's hourly/daily bucket boundaries: 3600s, 86400s)
- Implement continuous aggregates for agent/network rollups
- Wire M7's billing reconciliation to compare computed vs stored aggregates

---

### Phase 5: Real-Time Delivery

**Goal:** Recipients receive messages in real time.

#### 5.1 WebSocket/SSE Gateway

- Implement WebSocket connections for authenticated agents
- M9's dispatch contracts provide the decision layer (Delivered vs Queued vs QueueFull)
- M9's backpressure thresholds (warn at 300s, escrow extension at 3600s) drive escalation
- M9's anti-spam decisions gate message acceptance

#### 5.2 Presence Service

- Implement M9's scoped presence tracking
- Owner-scoped visibility (agents only visible within their owner's scope)
- Connection state management (online/offline/away)

---

### Phase 6: Compliance Automation

**Goal:** Retention policies are enforced, crypto-shredding runs automatically.

#### 6.1 Retention Scheduler

- Scheduled job evaluating M8's retention classes (Ephemeral: 24h, Standard: 90d, Extended: 1y)
- Legal-hold precedence enforcement (M8 contracts already handle this)
- Crypto-shredding execution: destroy CEK wrapped keys, set `shredded_at` marker
- M8's crypto-shredding is irreversible by design — envelope remains but content becomes unrecoverable

#### 6.2 Partition Archival

- Implement M10's monthly partition lifecycle (Active → Archived → Reattached)
- Export eligible partitions to Parquet + zstd (M10 specifies the format)
- Upload to S3/GCS (M10 tracks `archived_object_uri`)
- Verify M8 shred-completeness before archival (M10 already checks this)

---

## Dependency Order

```
Phase 1 (PostgreSQL)
  ├── 1.1 Add sqlx driver
  ├── 1.2 Schema migrations
  ├── 1.3 Repository layer
  └── 1.4 RLS policies
        │
Phase 2 (Crypto) ──── can start in parallel with Phase 1
  ├── 2.1 Envelope encryption
  └── 2.2 Blind index generation
        │
Phase 3 (Kolme) ──── requires Phase 1
  ├── 3.1 Real commit client
  └── 3.2 Batch scheduler
        │
Phase 4 (Extensions) ──── requires Phase 1
  ├── 4.1 pgvector (requires Phase 2 for decryption pipeline)
  ├── 4.2 Apache AGE
  └── 4.3 TimescaleDB
        │
Phase 5 (Real-Time) ──── requires Phase 1 + Phase 2
  ├── 5.1 WebSocket gateway
  └── 5.2 Presence service
        │
Phase 6 (Compliance) ──── requires Phase 1 + Phase 2
  ├── 6.1 Retention scheduler
  └── 6.2 Partition archival
```

Phases 1 and 2 can proceed in parallel. Phases 3-6 all depend on Phase 1. Phase 4.1 (pgvector) additionally requires Phase 2 for the decryption pipeline that generates embeddings.

---

## Sizing Estimate

| Phase | New Modules | Estimated LOC | External Dependencies |
|-------|-------------|---------------|----------------------|
| 1. PostgreSQL | 2-3 | 2,000-3,000 | sqlx, tokio |
| 2. Crypto pipeline | 1-2 | 1,000-1,500 | aes-gcm, x25519-dalek, zstd |
| 3. Kolme integration | 1 | 500-800 | kolme client crate |
| 4. PG extensions | 3 | 1,500-2,500 | pgvector, age (PG-side only) |
| 5. Real-time delivery | 1-2 | 1,000-1,500 | tokio-tungstenite or axum WS |
| 6. Compliance automation | 1-2 | 800-1,200 | tokio (scheduler), object-store |
| **Total** | **~12** | **~7,000-10,500** | |

This roughly doubles the data layer from 9,642 lines (contracts) to ~20,000 lines (contracts + infrastructure).

---

## What's Working in the Contract Layer's Favor

The contract-first approach means the infrastructure layer has clear, tested interfaces to implement against:

- **M0** tells you exactly what fields to validate before INSERT and how to verify the hash chain after SELECT
- **M1** gives you a complete merkle tree implementation — you just need to schedule it and store the results
- **M2** generates the actual PostgreSQL RLS SQL — you just need to execute it
- **M3** normalizes and hashes blind index values — you just need to store them in the GIN-indexed JSONB column
- **M4** validates every escrow state transition — the repository layer just persists the validated result
- **M5** ranks embeddings by cosine similarity — pgvector handles the actual vector math at query time
- **M6** exports AGE-compatible Cypher — you just need to pipe it to the graph extension
- **M7** computes rollups — TimescaleDB continuous aggregates can also do this server-side
- **M8** determines shred eligibility — the job just needs to execute the CEK destruction
- **M9** makes dispatch decisions — the WebSocket layer just needs to honor them
- **M10** plans partitions — PostgreSQL declarative partitioning handles the DDL

Every infrastructure component has a contract layer that defines its inputs, outputs, and invariants. The implementation work is wiring, not design.

---

## Risk: Contract-Infrastructure Divergence

The contract layer has been stable for 4 review cycles (R41-R44) with no changes. The longer it sits without a real infrastructure layer exercising it, the higher the risk that:

1. **Schema assumptions drift** — the PRD schema and the contract types may diverge as either evolves independently
2. **Performance surprises** — in-memory contracts don't reveal query patterns that are expensive at scale (e.g., M6 trust propagation over large graphs)
3. **Concurrency gaps** — the contracts are single-threaded; real PostgreSQL operations involve connection pools, transaction isolation, and concurrent access patterns not tested today
4. **Missing error paths** — the contracts model happy-path and explicit-rejection paths but may not cover database-level failures (connection loss, constraint violations, deadlocks)

The recommended mitigation is to start Phase 1 (PostgreSQL foundation) soon, even if only with the `messages` and `did_registry` tables, to validate the contract-to-persistence bridge before the full 6-phase build.
