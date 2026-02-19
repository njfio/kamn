
**KAMN DATA LAYER**

PRODUCT REQUIREMENTS DOCUMENT

Privacy-First Agent Storage, Search, and Intelligence

Version 1.0

February 17, 2026

KAMN Project — njfio/kamn

Backend: Kolme Blockchain Framework (fpco/kolme)

**Classification: Confidential**

# **Table of Contents**

## **0. R43 Standalone Decision Contract**

The following markers are authoritative for R43 data-layer design-decision tracking:

- `data_layer_standalone_reason_taxonomy_version`: `kamn.data-layer.standalone-decision.reason-taxonomy.v1`
- `data_layer_m11_operator_readiness_standalone_status`: `standalone_by_design`
- `data_layer_m11_operator_readiness_standalone_reason_code`: `data_layer_m11_operator_readiness_meta_assessment`
- `data_layer_prd_conformance_standalone_status`: `standalone_by_design`
- `data_layer_prd_conformance_standalone_reason_code`: `data_layer_prd_conformance_meta_assessment`
- `typed_did_migration_backlog_issue_ids`: `#5223,#5228,#5229,#5230`
- `typed_did_migration_backlog_scope`: `non_data_layer_string_did_callsites`
- `typed_did_migration_inventory_schema_version`: `kamn.typed-did-migration.inventory.v1`
- `typed_did_migration_inventory_non_data_layer_module_count`: `20`
- `typed_did_migration_inventory_non_data_layer_did_string_callsite_count`: `77`
- `typed_did_migration_inventory_excluded_boundary_modules`: `did.rs,did_registry.rs,service_api_endpoint.rs,kamn-sdk/src/service.rs`
- `typed_did_migration_wave_issue_ids`: `#5228,#5229,#5230`
- `typed_did_migration_wave_a_scope`: `bridge_and_marketplace`
- `typed_did_migration_wave_b_scope`: `operator_and_governance`
- `typed_did_migration_wave_c_scope`: `runtime_proof_and_reputation`

# **1\. Executive Summary**

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable coordination layer for autonomous AI agents. This Product Requirements Document specifies the complete data layer architecture that supports identity, messaging, escrow, settlement, audit trails, semantic intelligence, relationship mapping, operational telemetry, and compliance across the KAMN platform.

The data layer is built entirely on PostgreSQL and its extension ecosystem, providing a unified operational database that serves six distinct functional layers: on-chain trust anchoring via Kolme, envelope-encrypted message storage, vector-based semantic search via pgvector, knowledge graph capabilities via Apache AGE, time-series operational intelligence via TimescaleDB, and a DID-authenticated access gateway. This architecture eliminates the operational complexity of managing multiple database systems while delivering enterprise-grade security, searchability, data-mining capability, scalability, and economic efficiency.

All message content is encrypted at rest using envelope encryption with per-message symmetric keys wrapped for each authorized recipient. The Kolme blockchain serves exclusively as a trust anchor, storing only merkle root hashes, DID registrations, escrow state transitions, and settlement receipts. The full message corpus, audit evidence, vector embeddings, graph relationships, and telemetry reside in PostgreSQL, queryable only by DID-authenticated and authorized parties.

**KEY DESIGN PRINCIPLES**

* **Privacy by Default:** All message bodies are encrypted before storage. The platform operator cannot read message content. Vector embeddings are treated as sensitive data.

* **Verifiable Without Exposure:** On-chain merkle roots prove existence, ordering, and integrity of off-chain data without revealing content or participants.

* **Single Database Engine:** PostgreSQL with extensions (pgvector, Apache AGE, TimescaleDB) provides all storage, search, graph, and time-series capabilities in one operational footprint.

* **Economically Efficient:** No per-transaction or per-block storage fees. Flat infrastructure costs. zstd dictionary compression reduces storage by 8-15x.

* **Compliance-Ready:** Crypto-shredding enables GDPR/CCPA-compliant deletion without modifying append-only storage or blockchain state.

* **Scalable:** PostgreSQL partitioning, TimescaleDB hypertables, and Citus horizontal sharding provide a clear scaling path from prototype to production.

# **2\. System Context and Scope**

## **2.1 KAMN Platform Overview**

KAMN provides trust infrastructure for the agent economy: identity registration, message exchange, escrow management, settlement processing, and auditable records for interactions between autonomous AI agents. Agents are identified by decentralized identifiers (DIDs) and interact through cryptographically signed, encrypted messages. The Kolme blockchain provides the trust anchor, and this data layer provides the storage, search, and intelligence substrate.

## **2.2 Kolme Blockchain Integration**

Kolme is a Rust-powered blockchain framework from FP Complete (fpco/kolme) that provides dedicated blockchains per application with no gas fees, no execution limits, and a triadic security model (listeners, processor, approvers). Kolme uses PostgreSQL via SQLx for its own state storage. KAMN integrates with Kolme as follows:

* **On-Chain (Kolme):** Merkle root anchors, DID registry entries, escrow state machine transitions, settlement finality receipts, key rotation events, and reputation checkpoints.

* **Off-Chain (PostgreSQL Data Layer):** Encrypted message envelopes, plaintext metadata indexes, blind search indexes, vector embeddings, knowledge graph edges, time-series telemetry, audit evidence, and access logs.

This separation ensures that the Kolme chain remains compact (only hashes and state transitions), while the full data corpus lives in the queryable, mineable, scalable PostgreSQL layer.

## **2.3 Actors and Roles**

| Actor | Description | Data Access Scope |
| :---- | :---- | :---- |
| Agent | Autonomous AI entity with a DID, signing key, and encryption key | Own messages (sent/received), own escrow records, own reputation data |
| Owner | Human or organization controlling one or more agents | All messages across all owned agents (via owner master key), fleet analytics, billing data |
| Escrow Auditor | Neutral party or threshold-encrypted key holder | Messages within a specific escrow session (only during dispute resolution) |
| Platform Operator | KAMN infrastructure provider | Encrypted blobs only (cannot read content), aggregate anonymized metrics, system telemetry |
| Validator (Kolme) | Triadic validator set (listener, processor, approver) | On-chain state only (merkle roots, DID entries, escrow transitions) |
| External Auditor | Regulatory or compliance entity | Court-ordered access via threshold key reconstruction, access logs |

## **2.4 Scope Boundaries**

This PRD covers the complete data layer from message ingestion through storage, encryption, indexing, search, analytics, compliance, and access control. It does not cover the Kolme blockchain internals, agent runtime/execution environments, external wallet integrations, or the KAMN SDK wire protocol (covered in separate PRDs). It does specify all interfaces between the data layer and these adjacent systems.

# **3\. Architecture Overview**

## **3.1 Six-Layer Architecture**

The KAMN data layer comprises six interconnected functional layers, all running on PostgreSQL and its extensions:

| Layer | Technology | Purpose | Data Residency |
| :---- | :---- | :---- | :---- |
| Trust Anchor | Kolme Chain (Rust \+ PostgreSQL/SQLx) | Merkle roots, DIDs, escrow state, settlement receipts | On-chain (Kolme's own PostgreSQL) |
| Encrypted Storage | PostgreSQL (append-only tables) | Message envelopes, audit evidence, hash chains | Off-chain (KAMN PostgreSQL) |
| Semantic Search | pgvector extension | Message embeddings, anomaly detection, clustering | Off-chain (KAMN PostgreSQL) |
| Knowledge Graph | Apache AGE extension | Agent relationships, escrow DAGs, reputation networks | Off-chain (KAMN PostgreSQL) |
| Time-Series | TimescaleDB extension | Throughput metrics, latency, health telemetry, billing | Off-chain (KAMN PostgreSQL) |
| Access Gateway | Application layer (Rust service) | DID auth, ABAC policy, query scoping, rate limiting | Stateless (reads PostgreSQL) |

## **3.2 Data Flow Summary**

When Agent A sends a message to Agent B:

1. **Encryption:** Agent A generates a random AES-256-GCM content encryption key (CEK). The message body is encrypted with this CEK. The CEK is wrapped separately for each authorized recipient (Agent B, Agent A's own key, Agent A's owner master key, Agent B's owner master key) using their X25519 public keys.

2. **Submission:** The encrypted envelope (ciphertext \+ wrapped keys \+ plaintext metadata) is submitted to the KAMN gateway via the SDK.

3. **Authentication:** The gateway verifies Agent A's DID signature on the submission request.

4. **Storage:** The envelope is compressed with zstd (dictionary-trained) and inserted into the append-only messages table. A content hash (SHA-256 of the full envelope) is computed. The row includes a hash-chain pointer to the previous row.

5. **Indexing:** Plaintext metadata (sender DID, recipient DID, timestamp, session ID, message type, escrow ID) is indexed for query. Optional blind indexes are computed for searchable encrypted fields.

6. **Delivery:** Agent B is notified via WebSocket/SSE. Agent B retrieves the envelope, decrypts with their X25519 private key.

7. **Anchoring:** Periodically (configurable interval), a batch of content hashes is assembled into a merkle tree. The merkle root is committed to the Kolme chain as a transaction.

8. **Post-Processing:** Owner-side pipelines can decrypt messages, generate vector embeddings, and update the semantic search index. Graph edges are derived from metadata and inserted into Apache AGE. Time-series metrics are recorded in TimescaleDB hypertables.

# **4\. Cryptographic Architecture**

## **4.1 Key Hierarchy**

KAMN uses a three-tier key hierarchy aligned with the DID identity model:

| Key Type | Algorithm | Scope | Storage | Rotation Policy |
| :---- | :---- | :---- | :---- | :---- |
| Agent Signing Key | secp256k1 (ECDSA) | Per-agent, used for message signing and DID authentication | Agent runtime (never leaves agent boundary) | Configurable; rotation event recorded on Kolme chain |
| Agent Encryption Key | X25519 (Curve25519 DH) | Per-agent, used for CEK wrapping (receiving messages) | Agent runtime (private key), DID document (public key) | Rotated alongside or independently of signing key |
| Owner Master Key | X25519 | Per-owner, grants supervisory decryption of all owned agents' messages | Owner's secure key store (HSM, hardware wallet, or KMS) | Low frequency; rotation requires re-wrapping active CEKs |
| Content Encryption Key (CEK) | AES-256-GCM | Per-message, symmetric key for body encryption | Wrapped in each recipient's X25519 key; stored in envelope | Never reused; one CEK per message |
| Escrow Auditor Key | X25519 or threshold (Shamir/FROST) | Per-escrow, grants dispute resolution access to escrow messages | Split across N parties (e.g., 2-of-3 threshold) | Per-escrow lifecycle; destroyed after escrow settlement \+ retention window |
| Blind Index Key (BIK) | HMAC-SHA256 | Per-owner, used to generate searchable blind indexes | Owner's secure key store | Rotation requires re-indexing affected rows |

## **4.2 Envelope Encryption Protocol**

Every message follows the encrypt-then-store pattern:

**ENCRYPTION (SENDER SIDE)**

9. Generate a 256-bit random CEK.

10. Generate a 96-bit random nonce (IV) for AES-256-GCM.

11. Encrypt the message body: ciphertext \= AES-256-GCM(CEK, nonce, plaintext, AAD), where AAD (Additional Authenticated Data) includes sender DID, recipient DID, timestamp, and message type.

12. For each authorized recipient (including self, owner, and optionally escrow auditor), compute: wrapped\_key \= X25519-HKDF-ChaCha20Poly1305(recipient\_public\_key, CEK).

13. Assemble the envelope: { ciphertext, nonce, aad\_hash, wrapped\_keys: \[{did, wrapped\_cek}\], content\_hash: SHA256(envelope) }.

**DECRYPTION (RECIPIENT SIDE)**

14. Locate the wrapped\_key entry matching the recipient's DID.

15. Unwrap: CEK \= X25519-HKDF-ChaCha20Poly1305-Decrypt(private\_key, wrapped\_cek).

16. Decrypt: plaintext \= AES-256-GCM-Decrypt(CEK, nonce, ciphertext, AAD).

17. Verify AAD matches expected metadata (sender, recipient, timestamp, type).

## **4.3 Content Hash and Merkle Anchoring**

The content hash is computed as SHA-256 over the canonical serialization of the complete envelope (ciphertext \+ all wrapped keys \+ nonce \+ AAD hash). This hash serves dual purposes: it is the content-addressed identifier for the message in the append-only store, and it is the leaf value in the merkle tree submitted to Kolme.

Merkle trees are constructed using SHA-256 binary hash trees. Batch size and anchoring interval are configurable per deployment:

| Parameter | Default | Range | Tradeoff |
| :---- | :---- | :---- | :---- |
| Batch size | 1,000 messages | 100 \- 100,000 | Larger batches \= fewer on-chain txns but longer verification lag |
| Anchoring interval | 60 seconds | 10 seconds \- 1 hour | Shorter intervals \= faster tamper detection but more chain load |
| Max batch age | 5 minutes | 1 minute \- 1 hour | Ensures stale batches are anchored even during low traffic |

## **4.4 Hash Chain (Local Tamper Detection)**

In addition to merkle anchoring, each row in the messages table includes a hash\_chain\_prev field containing the SHA-256 hash of the previous row's content\_hash concatenated with its own row\_id. This creates a local hash chain within PostgreSQL that enables detection of deletions, reorderings, or insertions between merkle anchoring intervals. Verification is O(n) over the chain segment between two known-good merkle anchors.

## **4.5 Blind Indexes for Searchable Encryption**

Blind indexes enable server-side search over encrypted content without exposing plaintext. For each searchable field, the owner computes:

blind\_index \= HMAC-SHA256(blind\_index\_key, normalize(field\_value))

Where normalize() applies case-folding, whitespace normalization, and domain-specific canonicalization. The blind index is stored as an additional column alongside the encrypted envelope. The server can perform equality matching on blind indexes without learning the underlying values.

Limitations and mitigations:

* Frequency analysis: identical plaintext values produce identical blind indexes. Mitigation: add per-owner salt to the HMAC key, limiting analysis to within a single owner's dataset.

* No range queries: blind indexes support only equality matching. Mitigation: for range-queryable fields (e.g., amounts), use order-preserving encryption or store ranges as plaintext metadata where acceptable.

* No substring search: blind indexes are whole-value. Mitigation: compute n-gram blind indexes for fields requiring partial matching, at the cost of increased storage.

# **5\. PostgreSQL Schema Design**

## **5.1 Core Tables**

All tables use append-only semantics enforced by database-level triggers that reject UPDATE and DELETE operations on data rows. Administrative metadata (e.g., crypto-shredding markers) is the only mutable state.

### **5.1.1 messages (Primary Message Store)**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| message\_id | UUID (v7, time-ordered) | NOT NULL PK | Unique message identifier, time-sortable |
| content\_hash | BYTEA (32 bytes) | NOT NULL UNIQUE | SHA-256 of canonical envelope serialization |
| hash\_chain\_prev | BYTEA (32 bytes) | NOT NULL | SHA-256(prev.content\_hash || prev.row\_id) for local chain |
| sender\_did | TEXT | NOT NULL | Sender agent's DID (e.g., kamn:did:agent:sender-1) |
| recipient\_did | TEXT | NOT NULL | Primary recipient agent's DID |
| additional\_recipients | TEXT\[\] | NULL | Array of additional recipient DIDs (multicast) |
| owner\_sender\_did | TEXT | NOT NULL | DID of the sender agent's owner |
| owner\_recipient\_did | TEXT | NOT NULL | DID of the recipient agent's owner |
| session\_id | UUID | NULL | Conversation session grouping identifier |
| escrow\_id | UUID | NULL | Associated escrow if message is escrow-scoped |
| message\_type | TEXT | NOT NULL | Protocol message type enum (text, command, response, evidence, etc.) |
| priority | SMALLINT | NOT NULL DEFAULT 0 | Message priority (0=normal, 1=high, 2=urgent) |
| created\_at | TIMESTAMPTZ | NOT NULL | Server-side receipt timestamp (UTC) |
| sender\_timestamp | TIMESTAMPTZ | NULL | Sender-claimed timestamp (may differ from server time) |
| envelope\_ciphertext | BYTEA | NOT NULL | zstd-compressed AES-256-GCM encrypted message body |
| envelope\_nonce | BYTEA (12 bytes) | NOT NULL | GCM nonce/IV |
| envelope\_aad\_hash | BYTEA (32 bytes) | NOT NULL | SHA-256 of AAD used in encryption |
| wrapped\_keys | JSONB | NOT NULL | Array of {did, wrapped\_cek} objects |
| compression\_dict\_id | INTEGER | NULL | zstd dictionary version used for compression |
| merkle\_batch\_id | UUID | NULL | FK to merkle\_batches; set when anchored to Kolme |
| merkle\_leaf\_index | INTEGER | NULL | Position within the merkle batch tree |
| blind\_indexes | JSONB | NULL | Optional blind index values for searchable encryption |
| content\_size\_bytes | INTEGER | NOT NULL | Original uncompressed envelope size |
| compressed\_size\_bytes | INTEGER | NOT NULL | Stored compressed size |
| shredded\_at | TIMESTAMPTZ | NULL | If set, CEK keys have been destroyed (crypto-shredded) |
| jurisdiction | TEXT | NULL | Regulatory jurisdiction tag (e.g., EU, US-CA, APAC) |
| retention\_class | TEXT | NOT NULL DEFAULT 'standard' | Retention policy class (standard, legal\_hold, extended, permanent) |

### **5.1.2 merkle\_batches (Anchoring Records)**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| batch\_id | UUID | NOT NULL PK | Unique batch identifier |
| merkle\_root | BYTEA (32 bytes) | NOT NULL | Root hash of the merkle tree for this batch |
| message\_count | INTEGER | NOT NULL | Number of messages in this batch |
| first\_message\_id | UUID | NOT NULL | First message in batch (for ordering verification) |
| last\_message\_id | UUID | NOT NULL | Last message in batch |
| tree\_height | SMALLINT | NOT NULL | Merkle tree depth |
| tree\_serialization | BYTEA | NOT NULL | Full merkle tree (compressed) for proof generation |
| kolme\_tx\_hash | BYTEA | NULL | Kolme chain transaction hash (set after on-chain confirmation) |
| kolme\_block\_height | BIGINT | NULL | Kolme block number containing the anchor |
| anchored\_at | TIMESTAMPTZ | NULL | Timestamp of on-chain confirmation |
| batch\_created\_at | TIMESTAMPTZ | NOT NULL | When the batch was assembled |
| status | TEXT | NOT NULL DEFAULT 'pending' | pending | submitted | confirmed | failed |

### **5.1.3 did\_registry (Identity Records)**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| did | TEXT | NOT NULL PK | Decentralized identifier (e.g., kamn:did:agent:xyz) |
| did\_type | TEXT | NOT NULL | agent | owner | escrow\_auditor | service |
| owner\_did | TEXT | NULL | FK to owning identity (NULL for owners themselves) |
| signing\_pubkey | BYTEA | NOT NULL | Current secp256k1 public key (compressed, 33 bytes) |
| encryption\_pubkey | BYTEA | NOT NULL | Current X25519 public key (32 bytes) |
| signing\_pubkey\_prev | BYTEA | NULL | Previous signing key (for grace period verification) |
| encryption\_pubkey\_prev | BYTEA | NULL | Previous encryption key (for grace period decryption) |
| capabilities | JSONB | NOT NULL DEFAULT '\[\]' | Agent capability tags (e.g., \["text", "code", "trading"\]) |
| model\_identifier | TEXT | NULL | AI model backing this agent (e.g., claude-4, gpt-4) |
| delegation\_parent | TEXT | NULL | DID of parent agent if this is a delegated sub-agent |
| delegation\_scope | JSONB | NULL | Allowed actions for delegated agents |
| status | TEXT | NOT NULL DEFAULT 'active' | active | suspended | decommissioned | transferred |
| registered\_at | TIMESTAMPTZ | NOT NULL | Initial registration timestamp |
| last\_key\_rotation | TIMESTAMPTZ | NOT NULL | Most recent key rotation timestamp |
| kolme\_registration\_tx | BYTEA | NULL | On-chain registration transaction hash |
| metadata | JSONB | NULL | Additional agent metadata (display name, description, etc.) |

### **5.1.4 escrows (Escrow State Machine)**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| escrow\_id | UUID | NOT NULL PK | Unique escrow identifier |
| initiator\_did | TEXT | NOT NULL | DID of the agent/owner who created the escrow |
| counterparty\_did | TEXT | NOT NULL | DID of the other party |
| auditor\_encryption\_pubkey | BYTEA | NULL | Threshold-encrypted auditor key for dispute access |
| auditor\_threshold | SMALLINT | NULL | Number of shares required to reconstruct auditor key |
| auditor\_share\_holders | JSONB | NULL | DIDs of parties holding auditor key shares |
| amount | NUMERIC(38,18) | NOT NULL | Escrowed amount (supports 18 decimal places for crypto) |
| currency | TEXT | NOT NULL | Currency or token identifier |
| state | TEXT | NOT NULL DEFAULT 'created' | created | funded | active | disputed | released | refunded | expired |
| terms\_hash | BYTEA (32 bytes) | NOT NULL | SHA-256 of the escrow terms document |
| terms\_ciphertext | BYTEA | NULL | Encrypted escrow terms (readable by parties \+ auditor) |
| settlement\_receipt\_hash | BYTEA | NULL | Content hash of settlement evidence bundle |
| kolme\_state\_tx | BYTEA | NULL | Most recent on-chain state transition tx hash |
| created\_at | TIMESTAMPTZ | NOT NULL | Escrow creation timestamp |
| funded\_at | TIMESTAMPTZ | NULL | When funds were confirmed deposited |
| settled\_at | TIMESTAMPTZ | NULL | Settlement timestamp |
| expires\_at | TIMESTAMPTZ | NULL | Expiration deadline |
| dispute\_opened\_at | TIMESTAMPTZ | NULL | When dispute was initiated |
| dispute\_resolved\_at | TIMESTAMPTZ | NULL | When dispute was resolved |

### **5.1.5 key\_rotation\_log (Key Lifecycle Audit)**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| rotation\_id | UUID | NOT NULL PK | Unique rotation event identifier |
| did | TEXT | NOT NULL | DID whose key was rotated |
| key\_type | TEXT | NOT NULL | signing | encryption | owner\_master | blind\_index |
| old\_pubkey | BYTEA | NOT NULL | Previous public key |
| new\_pubkey | BYTEA | NOT NULL | New public key |
| rotated\_at | TIMESTAMPTZ | NOT NULL | Rotation timestamp |
| grace\_period\_until | TIMESTAMPTZ | NOT NULL | Old key remains valid for verification until this time |
| reason | TEXT | NOT NULL | scheduled | compromised | owner\_transfer | decommission |
| kolme\_tx\_hash | BYTEA | NULL | On-chain rotation record transaction hash |

### **5.1.6 access\_log (Query Audit Trail)**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| log\_id | UUID (v7) | NOT NULL PK | Unique log entry identifier |
| requester\_did | TEXT | NOT NULL | DID of the authenticated requester |
| action | TEXT | NOT NULL | query | decrypt\_request | export | admin | search |
| resource\_type | TEXT | NOT NULL | messages | escrows | agents | analytics |
| query\_fingerprint | BYTEA | NOT NULL | SHA-256 hash of the query (for deduplication analysis) |
| result\_count | INTEGER | NOT NULL | Number of records returned |
| source\_ip | INET | NULL | Source IP address |
| user\_agent | TEXT | NULL | Client identifier |
| timestamp | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | Query timestamp |
| latency\_ms | INTEGER | NOT NULL | Query execution time in milliseconds |

## **5.2 Indexes**

Critical indexes for query performance across all access patterns:

| Table | Index | Type | Purpose |
| :---- | :---- | :---- | :---- |
| messages | idx\_messages\_sender\_did\_created | B-tree (sender\_did, created\_at DESC) | Query agent's sent messages chronologically |
| messages | idx\_messages\_recipient\_did\_created | B-tree (recipient\_did, created\_at DESC) | Query agent's received messages chronologically |
| messages | idx\_messages\_session\_id | B-tree (session\_id, created\_at) | Retrieve full conversation sessions |
| messages | idx\_messages\_escrow\_id | B-tree (escrow\_id, created\_at) | All messages within an escrow |
| messages | idx\_messages\_owner\_sender | B-tree (owner\_sender\_did, created\_at DESC) | Owner-level fleet query (all agents' sent) |
| messages | idx\_messages\_owner\_recipient | B-tree (owner\_recipient\_did, created\_at DESC) | Owner-level fleet query (all agents' received) |
| messages | idx\_messages\_content\_hash | B-tree UNIQUE (content\_hash) | Content-addressed lookup and deduplication |
| messages | idx\_messages\_merkle\_batch | B-tree (merkle\_batch\_id, merkle\_leaf\_index) | Merkle proof reconstruction |
| messages | idx\_messages\_type\_created | B-tree (message\_type, created\_at DESC) | Filter by message type |
| messages | idx\_messages\_blind\_gin | GIN (blind\_indexes jsonb\_path\_ops) | Blind index equality search |
| messages | idx\_messages\_not\_shredded | B-tree partial (created\_at) WHERE shredded\_at IS NULL | Active (non-shredded) messages only |
| did\_registry | idx\_did\_owner | B-tree (owner\_did) | List all agents for an owner |
| did\_registry | idx\_did\_capabilities | GIN (capabilities) | Capability-based agent discovery |
| escrows | idx\_escrows\_parties | B-tree (initiator\_did, counterparty\_did) | Escrow lookup by participant |
| escrows | idx\_escrows\_state | B-tree (state, expires\_at) | Active/expiring escrow monitoring |
| access\_log | idx\_access\_log\_requester | B-tree (requester\_did, timestamp DESC) | Per-requester audit trail |

## **5.3 Partitioning Strategy**

The messages table is range-partitioned by created\_at on a monthly basis. This provides:

* Efficient time-range queries (the most common access pattern is recent messages).

* Independent VACUUM and maintenance per partition.

* Simple archival: old partitions can be moved to cheaper storage tiers or detached after retention windows expire.

* Parallel query execution across partitions for analytics workloads.

Partition naming convention: messages\_YYYY\_MM (e.g., messages\_2026\_02). A cron job or pg\_partman extension creates future partitions 3 months ahead and archives partitions older than the active retention window.

## **5.4 TOAST and Compression**

The envelope\_ciphertext column will exceed PostgreSQL's TOAST threshold (\~2KB) for most messages. By default, PostgreSQL compresses TOAST data with LZ4 (or pglz). However, KAMN applies application-level zstd compression before insertion, which substantially outperforms TOAST compression for agent message payloads.

Configuration:

* **TOAST strategy:** SET STORAGE EXTERNAL on envelope\_ciphertext to disable redundant TOAST compression (data is already zstd-compressed).

* **zstd dictionary:** A domain-specific dictionary trained on a corpus of KAMN message envelopes. Dictionaries are versioned (compression\_dict\_id column) and stored in a compression\_dictionaries table.

* **Expected compression ratio:** 8-15x for typical agent messages (natural language text with repetitive protocol structures). Individual small messages benefit most from dictionary training.

* **Dictionary retraining:** Weekly or monthly, using a sample of recent messages. Old dictionaries are retained for decompression of historical data.

# **6\. Vector Embeddings and Semantic Search**

## **6.1 Overview**

Vector embeddings enable semantic search, anomaly detection, behavioral profiling, and conversation clustering over agent messages. Because embeddings can be inverted to reconstruct substantial portions of the original text (research demonstrates 60-80% recovery rates), they are treated as sensitive data with the same privacy controls as message content.

## **6.2 Embedding Pipeline**

Embeddings are generated owner-side, not platform-side. The pipeline:

18. Owner's analytics service retrieves encrypted message envelopes for their agents.

19. Envelopes are decrypted using the owner's master key in a secure environment (e.g., private VPC, TEE, or local machine).

20. Plaintext messages are fed through an embedding model (configurable: OpenAI text-embedding-3-large, Cohere embed-v3, or a self-hosted model like BAAI/bge-large).

21. Embedding vectors are encrypted with the owner's symmetric key (AES-256-GCM, separate from message CEKs) before transmission to the platform.

22. Encrypted embeddings are stored in the message\_embeddings table with a reference to the source message.

## **6.3 Schema: message\_embeddings**

| Column | Type | Nullable | Description |
| :---- | :---- | :---- | :---- |
| embedding\_id | UUID (v7) | NOT NULL PK | Unique embedding identifier |
| message\_id | UUID | NOT NULL FK | Reference to source message |
| owner\_did | TEXT | NOT NULL | DID of the owner who generated this embedding |
| model\_id | TEXT | NOT NULL | Embedding model identifier (e.g., text-embedding-3-large) |
| dimensions | SMALLINT | NOT NULL | Vector dimensionality (768, 1024, 1536, 3072\) |
| vector\_encrypted | BYTEA | NOT NULL | AES-256-GCM encrypted embedding vector |
| vector\_nonce | BYTEA (12 bytes) | NOT NULL | GCM nonce for vector encryption |
| vector\_plaintext | vector(1536) | NULL | Decrypted vector (only populated in owner-scoped secure views) |
| centroid\_distance | FLOAT | NULL | Distance from agent's behavioral centroid (anomaly metric) |
| cluster\_id | INTEGER | NULL | Assigned conversation cluster (from periodic clustering runs) |
| created\_at | TIMESTAMPTZ | NOT NULL | Embedding generation timestamp |

## **6.4 Search Architecture**

Semantic search operates in two modes:

**MODE 1: OWNER-SIDE SEARCH (RECOMMENDED FOR SENSITIVE QUERIES)**

The owner downloads their encrypted embeddings, decrypts locally, builds an in-memory HNSW index (using libraries like faiss, usearch, or hnswlib), and performs similarity search locally. Results are message\_ids that the owner then retrieves and decrypts. This mode provides zero information leakage to the platform.

**MODE 2: SERVER-SIDE SEARCH (HIGHER PERFORMANCE, SOME LEAKAGE)**

For owners willing to accept limited information leakage in exchange for performance, decrypted vectors can be stored in the vector\_plaintext column within an owner-scoped secure schema. pgvector's HNSW index provides fast approximate nearest-neighbor search. The platform learns the vector values (and therefore could theoretically invert them) but not the plaintext messages. This mode is suitable for owners who trust the platform operator or use a secure enclave deployment.

pgvector index configuration:

* Index type: HNSW (preferred over IVFFlat for KAMN's workload — better recall at moderate scale).

* Distance metric: Cosine similarity (normalized embeddings).

* m \= 16, ef\_construction \= 200 (balanced recall/build time).

* ef\_search \= 100 (tunable per query for precision/speed tradeoff).

## **6.5 Anomaly Detection**

Each agent maintains a behavioral centroid: the mean embedding vector computed over its last N messages (configurable, default 500). When a new message's embedding exceeds a configurable distance threshold from the centroid, an anomaly event is generated.

Use cases:

* **Compromised agent detection:** A jailbroken or hijacked agent will produce messages with embedding patterns that diverge from its historical baseline.

* **Behavioral drift monitoring:** Gradual model updates or prompt changes can be tracked as centroid migration over time.

* **Quality assurance:** Owners can set alerts when agent behavior deviates beyond acceptable bounds.

## **6.6 Conversation Clustering**

Periodic batch jobs (nightly or weekly) run clustering algorithms (HDBSCAN or k-means) over an owner's message embeddings to automatically group related conversations. Clusters are assigned semantic labels via a summarization pass (the owner's analytics pipeline decrypts a sample of messages from each cluster and generates a label). This enables navigation and analysis of agent activity without manual tagging.

# **7\. Knowledge Graph**

## **7.1 Overview**

Apache AGE (A Graph Extension) adds openCypher graph query capabilities to PostgreSQL. The KAMN knowledge graph captures relationships between agents, owners, escrows, conversations, and capabilities. All graph data is derived from plaintext metadata — no message content enters the graph layer, preserving privacy.

## **7.2 Graph Schema**

**NODE TYPES**

| Node Label | Key Properties | Source |
| :---- | :---- | :---- |
| Agent | did, type, model, status, capabilities, registered\_at | did\_registry table |
| Owner | did, status, registered\_at | did\_registry table (did\_type \= 'owner') |
| Escrow | escrow\_id, state, amount, currency, created\_at | escrows table |
| Session | session\_id, started\_at, message\_count, last\_activity | Aggregated from messages metadata |
| Capability | name, category, version | Extracted from agent capability tags |
| Cluster | cluster\_id, owner\_did, label, centroid\_summary | From embedding clustering pipeline |

**EDGE TYPES**

| Edge Label | From → To | Key Properties | Meaning |
| :---- | :---- | :---- | :---- |
| OWNS | Owner → Agent | since, delegation\_scope | Ownership relationship |
| MESSAGED | Agent → Agent | count, first\_at, last\_at, sessions | Interaction edge (aggregated) |
| PARTICIPATED\_IN | Agent → Escrow | role (initiator/counterparty), joined\_at | Escrow participation |
| IN\_SESSION | Agent → Session | messages\_sent, messages\_received | Session membership |
| DELEGATED\_TO | Agent → Agent | scope, granted\_at, expires\_at, revocable | Delegation chain |
| HAS\_CAPABILITY | Agent → Capability | proficiency, verified\_at | Agent skill registration |
| TRUSTS | Agent → Agent | score, basis, updated\_at | Trust relationship with score |
| DEPENDS\_ON | Escrow → Escrow | dependency\_type, cascade\_policy | Escrow dependency chain (DAG) |
| BELONGS\_TO\_CLUSTER | Session → Cluster | similarity\_score | Conversation cluster membership |

## **7.3 Graph Queries**

Key query patterns supported by the knowledge graph:

* **Fleet Topology:** MATCH (o:Owner {did: $owner})-\[:OWNS\]-\>(a:Agent) RETURN a — List all agents for an owner.

* **Interaction Network:** MATCH (a:Agent {did: $agent})-\[m:MESSAGED\]-\>(b:Agent) RETURN b, m.count ORDER BY m.count DESC — Most frequent interaction partners.

* **Trust Propagation:** MATCH path \= (a:Agent {did: $agent})-\[:TRUSTS\*1..3\]-\>(target:Agent) RETURN target, reduce(s \= 1.0, r IN relationships(path) | s \* r.score) AS transitive\_trust — Transitive trust computation to 3 hops.

* **Escrow Cascade Analysis:** MATCH path \= (e:Escrow {escrow\_id: $id})-\[:DEPENDS\_ON\*\]-\>(downstream:Escrow) RETURN downstream — All escrows affected if this one fails.

* **Capability Matchmaking:** MATCH (a:Agent)-\[:HAS\_CAPABILITY\]-\>(c:Capability {name: $skill}) WHERE a.status \= 'active' RETURN a — Find agents with a specific skill.

* **Bad Actor Proximity:** MATCH (bad:Agent {status: 'suspended'})\<-\[:MESSAGED\*1..2\]-(nearby:Agent) RETURN nearby — Agents within 2 hops of a known bad actor.

## **7.4 Graph Maintenance**

Graph nodes and edges are maintained by event-driven triggers and periodic batch reconciliation:

* Agent registration/update: Triggers on did\_registry inserts/updates create or update Agent/Owner nodes.

* Message insertion: A lightweight trigger increments the MESSAGED edge counter between sender and recipient. Full edge property updates run in batch (hourly).

* Escrow state changes: Triggers on escrows table create/update Escrow nodes and PARTICIPATED\_IN edges.

* Trust score updates: Computed by a periodic job that combines message volume, escrow completion rates, anomaly scores, and network position (PageRank). Results written to TRUSTS edges and checkpointed to Kolme.

* Cluster updates: After each clustering run, BELONGS\_TO\_CLUSTER edges are refreshed.

# **8\. Time-Series Operational Intelligence**

## **8.1 Overview**

TimescaleDB hypertables provide automatic time-based partitioning, compression, and continuous aggregates for operational metrics. All metrics are derived from plaintext metadata and system telemetry — no message content is used.

## **8.2 Hypertables**

### **8.2.1 agent\_metrics (Per-Agent Operational Data)**

| Column | Type | Description |
| :---- | :---- | :---- |
| time | TIMESTAMPTZ | Metric timestamp (hypertable partition key) |
| agent\_did | TEXT | Agent DID |
| messages\_sent | INTEGER | Messages sent in this interval |
| messages\_received | INTEGER | Messages received in this interval |
| avg\_response\_latency\_ms | FLOAT | Average response time to incoming messages |
| p95\_response\_latency\_ms | FLOAT | 95th percentile response latency |
| active\_sessions | INTEGER | Number of active conversation sessions |
| active\_escrows | INTEGER | Number of active escrows |
| error\_count | INTEGER | Protocol errors or failed deliveries |
| embedding\_anomaly\_count | INTEGER | Anomaly detections in this interval |
| bytes\_sent | BIGINT | Total bytes of encrypted content sent |
| bytes\_received | BIGINT | Total bytes of encrypted content received |

Chunk interval: 1 day. Compression after 7 days. Retention: 90 days at full resolution, 2 years for continuous aggregates.

### **8.2.2 escrow\_metrics (Escrow Performance Tracking)**

| Column | Type | Description |
| :---- | :---- | :---- |
| time | TIMESTAMPTZ | Metric timestamp |
| escrow\_id | UUID | Escrow identifier |
| state | TEXT | Current escrow state |
| time\_in\_state\_ms | BIGINT | Duration in current state |
| messages\_exchanged | INTEGER | Messages exchanged in this escrow |
| dispute\_flag | BOOLEAN | Whether a dispute is active |

### **8.2.3 system\_telemetry (Platform Health)**

| Column | Type | Description |
| :---- | :---- | :---- |
| time | TIMESTAMPTZ | Metric timestamp |
| metric\_name | TEXT | Metric identifier (e.g., gateway\_request\_rate, pg\_connections, merkle\_batch\_lag) |
| metric\_value | DOUBLE PRECISION | Metric value |
| labels | JSONB | Dimensional labels (e.g., {"node": "primary", "region": "us-east"}) |

## **8.3 Continuous Aggregates**

Pre-computed rollups for dashboard and analytics queries:

* **agent\_metrics\_hourly:** Hourly rollups of per-agent metrics. Materialized continuously.

* **agent\_metrics\_daily:** Daily rollups. Used for trend analysis and billing.

* **network\_summary\_hourly:** Platform-wide aggregates (total messages, active agents, escrow volume). Used for operator dashboards.

* **owner\_billing\_daily:** Per-owner daily resource consumption (messages stored, bytes, queries executed, embeddings generated). Used for billing reconciliation.

## **8.4 Alerting Integration**

TimescaleDB continuous aggregates feed alerting rules:

* Agent response latency exceeds SLO threshold.

* Escrow time-in-state exceeds configured deadline.

* Embedding anomaly rate spikes above baseline.

* Merkle batch lag exceeds max\_batch\_age (indicating anchoring pipeline stall).

* Per-owner storage consumption approaching quota.

# **9\. Access Gateway and Authorization**

## **9.1 Authentication**

All data layer access is authenticated via DID challenge-response:

23. Client presents their DID to the gateway.

24. Gateway returns a random 32-byte challenge nonce with a 30-second TTL.

25. Client signs the nonce with their DID's secp256k1 private key.

26. Gateway verifies the signature against the DID's registered public key (from did\_registry or Kolme chain).

27. On success, the gateway issues a short-lived session token (JWT with 15-minute expiry, renewable).

For agent-to-agent communication, agents authenticate once per session and maintain persistent WebSocket connections through the gateway.

## **9.2 Authorization Model (ABAC)**

The gateway enforces Attribute-Based Access Control (ABAC) with the following policy rules:

| Rule | Condition | Scope |
| :---- | :---- | :---- |
| Agent Self-Access | requester\_did \= sender\_did OR requester\_did \= recipient\_did | Read own sent and received messages |
| Owner Fleet Access | requester\_did \= owner\_did AND (owner\_sender\_did \= owner\_did OR owner\_recipient\_did \= owner\_did) | Read all messages across owned agents |
| Escrow Participant Access | requester\_did IN (initiator\_did, counterparty\_did) for the escrow\_id | Read messages within escrows they participate in |
| Escrow Auditor Access | requester presents valid threshold key reconstruction for escrow auditor key | Read messages within a specific disputed escrow |
| Legal Discovery | requester presents court order \+ platform operator threshold key share \+ external auditor share | Read messages identified in discovery order |
| Platform Operator | requester \= platform admin DID | Encrypted blobs only; cannot decrypt. Access to aggregate metrics and system telemetry |

## **9.3 Query Scoping**

Every query passing through the gateway is automatically rewritten to include a WHERE clause restricting results to the requester's authorized scope. This is enforced at the PostgreSQL level via Row-Level Security (RLS) policies, not just at the application level, providing defense-in-depth against gateway bypass.

Example RLS policy for the messages table:

* CREATE POLICY messages\_agent\_access ON messages FOR SELECT USING (sender\_did \= current\_setting('kamn.requester\_did') OR recipient\_did \= current\_setting('kamn.requester\_did'));

* The gateway sets the kamn.requester\_did session variable before each query, and RLS enforces scoping transparently.

## **9.4 Rate Limiting**

Per-DID rate limits prevent enumeration attacks and resource abuse:

| Tier | Queries/Minute | Rows/Query | Embeddings/Day | Graph Queries/Minute |
| :---- | :---- | :---- | :---- | :---- |
| Free Agent | 60 | 100 | 1,000 | 30 |
| Standard Agent | 300 | 1,000 | 10,000 | 150 |
| Premium Agent | 1,200 | 10,000 | 100,000 | 600 |
| Owner Fleet | 3,000 | 50,000 | 500,000 | 1,500 |
| Platform Admin | Unlimited | Unlimited | N/A | Unlimited |

## **9.5 Audit Logging**

Every query executed through the gateway is logged to the access\_log table. The audit log is itself append-only, hash-chained, and periodically merkle-anchored to Kolme. This creates an immutable record of who accessed what data, when, and from where. The audit log is accessible only to the platform operator and (via legal discovery) to authorized external auditors.

# **10\. Real-Time Messaging and Event Streaming**

## **10.1 WebSocket/SSE Message Delivery**

Agents maintain persistent WebSocket connections to the gateway for real-time message delivery. The flow:

28. Agent authenticates and upgrades to WebSocket connection.

29. Gateway subscribes the agent's DID to a PostgreSQL LISTEN/NOTIFY channel (or Redis pub/sub for multi-gateway deployments).

30. When a new message is stored with the agent as recipient, a NOTIFY event fires.

31. Gateway pushes the encrypted envelope to the agent over the WebSocket.

32. Agent acknowledges receipt. If no ACK within configurable timeout, message is re-queued.

For agents that cannot maintain persistent connections (e.g., serverless deployments), Server-Sent Events (SSE) and polling endpoints are available as fallbacks.

## **10.2 Presence System**

The gateway maintains a presence table tracking which agents are currently connected:

| Column | Type | Description |
| :---- | :---- | :---- |
| agent\_did | TEXT PK | Agent DID |
| connected\_since | TIMESTAMPTZ | Connection establishment time |
| last\_heartbeat | TIMESTAMPTZ | Last heartbeat received |
| gateway\_node | TEXT | Which gateway node holds the connection |
| capabilities\_active | JSONB | Currently advertised capabilities |

Agents can query the presence of potential counterparties before initiating conversations. Presence data is scoped: agents can only see presence of agents they have previously interacted with or agents within the same escrow. Presence data is ephemeral and not stored in the append-only log.

## **10.3 Event-Driven Triggers**

State changes in the data layer fire webhook notifications to subscribed endpoints:

* **Escrow State Change:** Fires when an escrow transitions state (created, funded, active, disputed, released, refunded, expired). Subscribers: both parties \+ auditor.

* **Key Rotation:** Fires when a counterparty rotates their encryption key. Subscribers: all agents with active sessions involving the rotating party.

* **Anomaly Detection:** Fires when an agent's embedding anomaly score exceeds threshold. Subscribers: agent's owner.

* **Merkle Anchor Confirmation:** Fires when a merkle batch is confirmed on-chain. Subscribers: configurable per deployment.

* **Agent Status Change:** Fires when an agent is suspended, decommissioned, or transferred. Subscribers: all agents with active sessions.

## **10.4 Backpressure and Flow Control**

When a recipient agent cannot keep up with incoming message rate:

* Gateway buffers up to 1,000 pending messages per agent in a PostgreSQL queue table (message\_delivery\_queue).

* If the buffer fills, new messages are accepted into the messages table but delivery is deferred. Senders receive a 'queued' acknowledgment rather than 'delivered'.

* Persistent backpressure (buffer full for \> 5 minutes) triggers a warning event to the recipient agent's owner.

* Sustained backpressure (\> 1 hour) may trigger automatic escrow timeout extensions for in-flight escrows.

# **11\. Compliance, Retention, and Data Lifecycle**

## **11.1 Crypto-Shredding**

Crypto-shredding is the primary mechanism for data deletion in an append-only system. When a message must be deleted (GDPR right-to-erasure, retention policy expiry, or owner request):

33. All wrapped CEK entries for the target message are destroyed from the wrapped\_keys JSONB column (replaced with a shredding tombstone marker).

34. The shredded\_at timestamp is set on the message row.

35. The ciphertext remains in the append-only store but is permanently unrecoverable.

36. The content\_hash and hash\_chain\_prev values are preserved, maintaining the integrity of the hash chain and merkle anchors.

37. The merkle tree on Kolme continues to prove that a message existed at that position and time, but the content is gone.

Crypto-shredding is irreversible. The platform does not maintain any backup of destroyed CEKs. Legal hold status (retention\_class \= 'legal\_hold') prevents shredding until the hold is released.

## **11.2 Retention Policies**

| Retention Class | Default Duration | Shredding Behavior | Override |
| :---- | :---- | :---- | :---- |
| standard | 90 days | CEKs auto-shredded after duration | Owner can extend or shorten |
| extended | 1 year | CEKs auto-shredded after duration | Used for compliance-sensitive interactions |
| legal\_hold | Indefinite | Shredding blocked until hold released | Set by platform operator or court order |
| permanent | Never | Never shredded | Used for on-chain anchored settlement receipts |
| ephemeral | 24 hours | CEKs shredded after 24 hours | For transient/discovery conversations |

Retention policies are enforced by a background job (kamn-retention-worker) that runs every hour, identifies messages past their retention window, and executes crypto-shredding. Messages in escrows inherit the escrow's retention policy, which extends at minimum to escrow settlement \+ dispute window \+ regulatory hold period.

## **11.3 GDPR and CCPA Compliance**

KAMN's architecture is designed for compliance with major data protection regulations:

* **Right to Erasure (Art. 17 GDPR):** Satisfied via crypto-shredding. Ciphertext remnants are not personal data because they cannot be decrypted.

* **Data Portability (Art. 20 GDPR):** Owners can export all their agents' decrypted messages via the gateway API. Export includes messages, metadata, embeddings, and graph relationships.

* **Right of Access (Art. 15 GDPR):** Owners can query all data associated with their DID. The access\_log provides a record of all access to their data.

* **Data Minimization (Art. 5(1)(c) GDPR):** Plaintext metadata is limited to the minimum required for routing and access control. Message content is encrypted; the platform cannot process it.

* **Jurisdiction Tags:** Each message carries a jurisdiction field enabling per-message regulatory classification. Cross-jurisdiction interactions inherit the stricter retention/compliance regime.

## **11.4 Agent Lifecycle Events**

**AGENT DECOMMISSION**

When an agent is decommissioned:

38. Status set to 'decommissioned' in did\_registry.

39. All active sessions are terminated with a protocol-level 'agent\_decommissioned' message to counterparties.

40. Active escrows are flagged for manual review (cannot auto-settle).

41. Encryption key is retained for the retention window (counterparties may still need to decrypt historical messages).

42. After retention window: crypto-shred all messages, remove graph edges, delete embeddings.

**AGENT TRANSFER (OWNERSHIP CHANGE)**

When an agent is transferred from Owner A to Owner B:

43. New Owner B's master encryption key is added to the agent's DID document.

44. All future messages will include Owner B's wrapped CEK.

45. Historical messages: Owner A retains access to pre-transfer messages (their wrapped CEKs are not modified). Owner B can access pre-transfer messages only if Owner A explicitly re-wraps the historical CEKs for Owner B (optional, consent-based).

46. Graph edges are updated: OWNS edge from A removed, new OWNS edge to B created, with transfer\_from and transferred\_at properties.

47. Reputation/trust scores travel with the agent (they are properties of the agent node, not the owner).

**AGENT FORKING (CAPABILITY CLONE)**

An agent fork creates a new agent with a fresh DID that inherits capability tags but not reputation, message history, or escrow records. The graph records a FORKED\_FROM edge for provenance. The forked agent starts with a zero trust score and must establish its own interaction history.

**DELEGATION**

Agent A can delegate authority to Agent B via a DELEGATED\_TO graph edge with a scoped capability set. Delegated agents can act on behalf of their delegator within the defined scope. Delegation is revocable at any time by the delegating agent or their owner. Delegation chains are limited to a configurable depth (default: 3\) to prevent unbounded transitive delegation.

# **12\. Economic Model and Resource Metering**

## **12.1 Cost Structure**

KAMN's data layer operates on flat infrastructure costs rather than per-transaction fees. However, compute and storage resources must be metered and allocated fairly across tenants.

| Resource | Unit of Measure | Metering Point | Billing Model |
| :---- | :---- | :---- | :---- |
| Message Storage | Compressed bytes stored | Insertion into messages table | Prepaid capacity tiers (GB/month) |
| Message Delivery | Messages delivered via WebSocket/SSE | Gateway delivery events | Included in tier; overage per 1K messages |
| Query Execution | Queries executed | Gateway access log | Included in tier; overage per 1K queries |
| Vector Embedding Storage | Vectors stored | Insertion into message\_embeddings | Prepaid per 100K vectors/month |
| Semantic Search Queries | ANN queries executed | pgvector query events | Per 1K queries |
| Graph Queries | Cypher queries executed | AGE query events | Per 1K queries |
| Merkle Anchoring | Batches anchored to Kolme | merkle\_batches submissions | Flat rate (shared across all tenants) |
| Data Export | Bytes exported | Gateway export events | Per GB exported |

## **12.2 Tier Structure**

| Tier | Agents | Messages/Month | Storage | Vectors | Price Guidance |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Free | 3 | 10,000 | 1 GB | 10,000 | Free |
| Starter | 25 | 500,000 | 50 GB | 500,000 | Usage-based |
| Professional | 250 | 10,000,000 | 500 GB | 5,000,000 | Committed |
| Enterprise | Unlimited | Unlimited | Custom | Custom | Negotiated |

## **12.3 Staking and Reputation**

Optionally, agents or owners can stake value (via Kolme's bridge contracts from Solana, Ethereum, or other supported chains) to back their trust scores. Staked agents receive higher initial trust scores and priority in capability matchmaking. Stake can be slashed for protocol violations (e.g., message tampering detected, escrow fraud, anomaly-confirmed malicious behavior). Staking is optional and does not affect data layer functionality.

## **12.4 Storage Reclamation**

When messages are crypto-shredded, the ciphertext bytes remain in the append-only store but are logically deleted. Physical storage reclamation occurs during partition archival: when an entire monthly partition's messages are all shredded, the partition can be dropped entirely, freeing disk space. Partial reclamation within active partitions occurs via PostgreSQL's VACUUM process on the TOAST tables.

# **13\. Scaling Strategy**

## **13.1 Vertical Scaling (Phase 1\)**

A single PostgreSQL instance with extensions handles the initial deployment:

* Target: up to 50,000 agents, 100M messages/month, 500GB storage.

* Hardware: 32-64 vCPUs, 256GB RAM, NVMe SSD storage.

* Key optimizations: connection pooling (PgBouncer), prepared statements, partitioned tables, TimescaleDB compression.

## **13.2 Read Replicas (Phase 2\)**

Streaming replication to read replicas for query scaling:

* Primary handles all writes (message insertion, graph updates, metric recording).

* Read replicas handle analytics queries, embedding search, graph traversal, and time-series dashboards.

* Gateway routes read-only queries to replicas with configurable replication lag tolerance.

* Target: up to 500,000 agents, 1B messages/month.

## **13.3 Horizontal Sharding (Phase 3\)**

Citus extension for transparent horizontal sharding:

* Shard key: owner\_did. All data for a single owner co-locates on one shard, enabling efficient owner-scoped queries without cross-shard joins.

* Reference tables: did\_registry, compression\_dictionaries (replicated to all shards).

* Distributed tables: messages, message\_embeddings, agent\_metrics.

* Graph sharding: Apache AGE graphs are per-owner, enabling shard-local graph queries. Cross-owner graph queries (e.g., inter-owner trust networks) require scatter-gather.

* Target: millions of agents, 10B+ messages/month.

## **13.4 Cold Storage and Archival**

Monthly partitions older than the active retention window are archived:

* Partitions are exported to compressed Parquet files on object storage (S3/GCS/MinIO).

* Parquet files retain the encrypted ciphertext and metadata for regulatory compliance.

* An archival\_index table tracks which partitions have been archived and their object storage locations.

* Re-attachment: archived partitions can be re-attached to PostgreSQL for ad-hoc historical queries.

# **14\. Compression Strategy**

## **14.1 zstd Dictionary Compression**

KAMN uses zstd with trained dictionaries for application-level compression of message envelopes before insertion into PostgreSQL.

| Parameter | Value | Rationale |
| :---- | :---- | :---- |
| Algorithm | zstd (Zstandard) | Best ratio-to-speed tradeoff; excellent dictionary support |
| Compression level | 3 (default) | Balanced compression ratio and CPU cost; tunable per deployment |
| Dictionary size | 112 KB (default) | Trained on 10,000 sample messages; covers common protocol structures |
| Dictionary retraining | Weekly | Ensures dictionary tracks evolving message patterns |
| Dictionary versioning | Monotonic integer ID | Each message records which dictionary was used for compression |
| Expected ratio (with dict) | 8-15x | Agent messages have highly repetitive protocol structures and vocabulary |
| Expected ratio (without dict) | 3-5x | Fallback if dictionary is unavailable |
| Decompression speed | \~5 GB/s | Near-zero overhead on read path |

## **14.2 Dictionary Training Pipeline**

The dictionary training pipeline:

48. Sample 10,000-50,000 recent message envelopes (after encryption, before compression).

49. Run zstd \--train on the sample corpus to produce a dictionary.

50. Validate the dictionary against a held-out test set; reject if compression ratio degrades vs. previous dictionary.

51. Store the new dictionary in the compression\_dictionaries table with a new version ID.

52. Configure the message ingestion pipeline to use the new dictionary for new messages.

53. Retain all historical dictionaries indefinitely (required for decompression of older messages).

## **14.3 TimescaleDB Native Compression**

TimescaleDB hypertables use native columnar compression for chunks older than 7 days:

* Compression algorithm: gorilla (for float metrics) \+ delta-of-delta (for timestamps) \+ dictionary (for text labels).

* Expected compression: 10-20x for time-series metrics.

* Compressed chunks are read-only; decompression is transparent on query.

## **14.4 Parquet Archival Compression**

Archived partitions exported to Parquet use:

* Column-level compression: zstd for BYTEA columns (ciphertext), dictionary encoding for TEXT columns (DIDs, types).

* Expected total compression: 15-25x vs. uncompressed PostgreSQL heap.

* Parquet files are queryable via DuckDB or Apache DataFusion for ad-hoc historical analysis without re-importing to PostgreSQL.

# **15\. Security Model**

## **15.1 Threat Model**

| Threat | Mitigation | Residual Risk |
| :---- | :---- | :---- |
| Database compromise (attacker gains full PostgreSQL access) | All message bodies encrypted; attacker gets ciphertext only. Metadata (DIDs, timestamps) exposed. | Metadata leakage reveals interaction patterns. Mitigation: metadata minimization, DID pseudonymity. |
| Platform operator reads messages | Envelope encryption; operator never holds CEKs or owner master keys. | None (operator cannot decrypt by design). |
| Embedding inversion attack | Embeddings encrypted per-owner; server-side mode requires explicit owner opt-in. | Owner-opted server-side vectors are vulnerable to platform compromise. Mitigation: secure enclave deployment. |
| Blind index frequency analysis | Per-owner salted HMAC keys; analysis limited to single-owner datasets. | Within-owner frequency patterns detectable. Acceptable for owner-scoped search. |
| Key compromise (agent signing/encryption key stolen) | Key rotation mechanism \+ grace period. On-chain rotation record. Forward secrecy via session-level ratcheting (optional). | Messages encrypted with compromised key remain at risk until key rotation detected. |
| Merkle anchor tampering | Kolme chain with triadic validator consensus. Anchors immutable once confirmed. | Kolme chain consensus compromise (requires collusion of 2+ validator groups). |
| Denial of service (flood messages) | Per-DID rate limiting, backpressure system, connection limits. | Sustained DDoS may degrade service. Mitigation: scaling, CDN, admission control. |
| SQL injection via gateway | Parameterized queries only. RLS enforces scoping even if query injection succeeds. | Defense-in-depth via RLS makes injection ineffective for privilege escalation. |
| Insider threat (rogue platform employee) | No CEK access. Database credentials managed via IAM/RBAC. Audit log is append-only and anchored. | Metadata visible. Mitigation: background checks, access reviews, break-glass procedures. |

## **15.2 Encryption at Rest and in Transit**

* **At rest:** Application-level envelope encryption (AES-256-GCM) for message bodies. PostgreSQL TDE (Transparent Data Encryption) for the database volume. Object storage encryption (AES-256) for archived Parquet files.

* **In transit:** TLS 1.3 for all gateway connections (WebSocket, HTTPS, SSE). mTLS between gateway and PostgreSQL. mTLS between KAMN services and Kolme nodes.

* **Key management:** Agent and owner keys managed by their respective environments. Platform operational keys (database credentials, TLS certificates) managed via HashiCorp Vault or AWS KMS/GCP Cloud KMS.

## **15.3 Privacy Invariants**

The following invariants must hold under all conditions, including system compromise, operator malfeasance, and legal compulsion:

54. Plaintext message bodies never exist outside the sender/recipient's local environment.

55. The platform operator can never read message content.

56. Vector embeddings are computed owner-side and stored encrypted.

57. Graph edges are derived from plaintext metadata only — no content leakage.

58. On-chain merkle roots prove existence and ordering without revealing content or participants.

59. Crypto-shredding satisfies deletion requests without modifying append-only storage or chain state.

60. All data access is logged, and the access log itself is append-only and anchored.

61. Legal discovery requires threshold key reconstruction — no single party can unilaterally access disputed messages.

# **16\. Observability and Monitoring**

## **16.1 Metrics**

The following metrics are collected via the system\_telemetry hypertable and exported to Prometheus/Grafana:

* Gateway: request rate, latency percentiles, active WebSocket connections, authentication failures, rate limit hits.

* Message Pipeline: insertion rate, compression ratio (actual vs. expected), batch assembly time, anchoring lag.

* PostgreSQL: connection pool utilization, query latency, replication lag, table/index sizes, VACUUM progress.

* pgvector: ANN query latency, recall (sampled), index build time, memory usage.

* Apache AGE: query latency by type, graph size (nodes/edges), cache hit ratio.

* TimescaleDB: chunk compression ratio, continuous aggregate refresh latency, retention policy execution.

* Kolme Integration: transaction submission latency, confirmation time, block height, validator health.

## **16.2 Logging**

Structured JSON logging for all components:

* Gateway: request/response logs with correlation IDs (no message content logged).

* Encryption pipeline: envelope creation/verification events (no plaintext logged).

* Merkle anchoring: batch assembly, submission, confirmation events.

* Retention worker: crypto-shredding events, retention policy evaluations.

* All logs shipped to centralized log aggregation (ELK, Loki, or equivalent).

## **16.3 Alerting Rules**

| Alert | Condition | Severity | Action |
| :---- | :---- | :---- | :---- |
| Anchoring Lag | merkle batch unconfirmed \> 10 minutes | Critical | Investigate Kolme chain / submission pipeline |
| Hash Chain Break | hash\_chain\_prev verification failure | Critical | Halt ingestion, investigate data integrity |
| High Error Rate | Gateway 5xx rate \> 1% for 5 minutes | High | Auto-scale, investigate root cause |
| Replication Lag | Streaming replication \> 30 seconds | High | Investigate replica health |
| Storage Quota | Owner storage \> 90% of tier allocation | Medium | Notify owner, suggest tier upgrade |
| Anomaly Spike | Agent anomaly rate \> 3x baseline | Medium | Notify owner, flag agent for review |
| Certificate Expiry | TLS cert expires within 14 days | Medium | Auto-renew or manual rotation |

# **17\. Migration, Deployment, and Operations**

## **17.1 Database Migrations**

Schema migrations are managed via a versioned migration framework (sqlx-migrate or Flyway). All migrations are:

* Forward-only (no rollback scripts for append-only tables — destructive rollbacks are incompatible with the data model).

* Tested in a staging environment with synthetic data before production deployment.

* Applied in maintenance windows for DDL changes that acquire AccessExclusiveLock.

* Online for index creation (CREATE INDEX CONCURRENTLY).

## **17.2 Extension Management**

| Extension | Version | Installation | Notes |
| :---- | :---- | :---- | :---- |
| pgvector | \>= 0.7.0 | CREATE EXTENSION vector; | HNSW index support required |
| Apache AGE | \>= 1.5.0 | CREATE EXTENSION age; SET search\_path \= ag\_catalog; | openCypher query support |
| TimescaleDB | \>= 2.14 | CREATE EXTENSION timescaledb; | Community edition sufficient; licensed for compression |
| pg\_partman | \>= 5.0 | CREATE EXTENSION pg\_partman; | Automated partition management |
| pgcrypto | Built-in | CREATE EXTENSION pgcrypto; | gen\_random\_uuid(), digest() functions |

## **17.3 Deployment Architecture**

Production deployment topology:

* **Primary PostgreSQL:** Handles all writes. Extensions: pgvector, AGE, TimescaleDB, pg\_partman, pgcrypto.

* **Read Replicas (2+):** Streaming replication. Handle analytics, search, graph queries.

* **Gateway Cluster (3+):** Stateless Rust services behind a load balancer. WebSocket affinity via consistent hashing.

* **Merkle Anchoring Worker:** Single-leader worker that assembles batches and submits to Kolme. Leader election via pg\_advisory\_lock.

* **Retention Worker:** Periodic job for crypto-shredding. Runs with restricted database permissions (can UPDATE shredded\_at and wrapped\_keys, nothing else).

* **Connection Pooler (PgBouncer):** Transaction-mode pooling. 500-1000 connection pool size depending on tier.

* **Object Storage (S3/MinIO):** Archived Parquet partitions, zstd dictionaries, backup snapshots.

## **17.4 Backup and Disaster Recovery**

* **Continuous Archiving:** WAL archiving to object storage (pg\_basebackup \+ pg\_receivewal or pgBackRest).

* **Point-in-Time Recovery:** Recovery to any point within the WAL retention window (default: 7 days).

* **RPO:** \< 1 second (synchronous replication to at least one replica).

* **RTO:** \< 15 minutes (automated failover via Patroni or pg\_auto\_failover).

* **Cross-Region Replication:** Asynchronous replica in a secondary region for disaster recovery.

* **Backup Encryption:** All backups encrypted at rest with a separate backup encryption key (not the same as data CEKs).

# **18\. Testing Strategy**

## **18.1 Test Categories**

| Category | Scope | Frequency | Tooling |
| :---- | :---- | :---- | :---- |
| Unit Tests | Individual functions: encryption, hashing, compression, blind index computation | Every commit | Rust \#\[test\], Python pytest |
| Contract Tests | Schema validation, migration correctness, RLS policy enforcement | Every commit | sqlx-test, pg\_prove |
| Integration Tests | End-to-end message flow: encrypt → store → query → decrypt → anchor | Every PR | Custom test harness (Rust) |
| Crypto Verification | Envelope round-trip, merkle proof verification, hash chain integrity | Every PR | Property-based testing (proptest) |
| Performance Tests | Query latency under load, insertion throughput, ANN search recall/speed | Weekly | pgbench, k6, custom benchmarks |
| Chaos Tests | Network partitions, replica lag, worker crashes, Kolme chain delays | Monthly | Toxiproxy, custom fault injection |
| Security Tests | RLS bypass attempts, injection vectors, key rotation edge cases | Monthly | Custom security test suite |
| Compliance Tests | Crypto-shredding verification, retention policy enforcement, export completeness | Monthly | Custom compliance test suite |

## **18.2 Critical Test Scenarios**

The following scenarios must pass before any production deployment:

62. A message encrypted by Agent A can be decrypted by Agent B, Agent A's owner, Agent B's owner, and no one else.

63. After crypto-shredding, the message ciphertext remains in the database but cannot be decrypted by any party.

64. The merkle root anchored to Kolme verifies correctly against the stored merkle tree and individual message hashes.

65. The hash chain detects a single deleted or reordered row within a batch.

66. RLS policies prevent Agent A from querying Agent C's messages (where C is not a counterparty).

67. An owner can query all messages across their agent fleet but not messages between agents they don't own.

68. Blind index search returns the correct results for exact-match queries.

69. Vector similarity search returns semantically relevant results with recall \> 95% at k=10.

70. Graph queries return correct transitive trust scores to 3 hops.

71. Under sustained 10,000 messages/second load, p99 insertion latency remains below 50ms.

# **19\. Dependencies, Assumptions, and Risks**

## **19.1 Dependencies**

| Dependency | Type | Version | Risk Level | Mitigation |
| :---- | :---- | :---- | :---- | :---- |
| PostgreSQL | Infrastructure | \>= 16 | Low | Mature, widely supported, multiple managed service options |
| pgvector | Extension | \>= 0.7.0 | Low | Active development, widely adopted, Supabase/Neon support |
| Apache AGE | Extension | \>= 1.5.0 | Medium | Younger project; evaluate Neo4j as fallback if AGE stalls |
| TimescaleDB | Extension | \>= 2.14 | Low | Mature, well-funded, enterprise support available |
| Kolme | Blockchain | fpco/kolme main | Medium | Proprietary framework; dependency on FP Complete roadmap |
| zstd | Library | \>= 1.5 | Low | Industry standard, used by Linux kernel, Facebook, etc. |
| Rust Ecosystem | Language/Runtime | Stable | Low | Well-supported, active ecosystem |

## **19.2 Assumptions**

* PostgreSQL extensions (pgvector, AGE, TimescaleDB) can coexist in a single PostgreSQL instance without incompatibility.

* Kolme chain provides reliable sub-minute block finality for merkle anchor transactions.

* Agent runtime environments can securely generate and manage cryptographic keys (secp256k1, X25519).

* Owner-side embedding generation infrastructure (GPU access or embedding API) is available.

* Network latency between gateway and PostgreSQL is consistently \< 5ms.

* zstd dictionary compression achieves at least 5x ratio on KAMN message payloads (to be validated in prototype).

## **19.3 Risks**

| Risk | Likelihood | Impact | Mitigation |
| :---- | :---- | :---- | :---- |
| Apache AGE maturity limits graph query capabilities | Medium | Medium | Design graph schema to be portable to Neo4j; abstract via query interface layer |
| pgvector \+ AGE \+ TimescaleDB extension conflicts | Low | High | Test all extensions together in integration environment before production. Fallback: separate PostgreSQL instances per extension. |
| zstd dictionary training corpus insufficient at launch | Medium | Low | Use generic zstd compression initially; train dictionary after accumulating sufficient message corpus |
| Kolme chain availability/latency impacts anchoring | Low | Medium | Batch buffering \+ retry logic. Messages remain valid in PostgreSQL regardless of chain status. |
| Embedding inversion attacks improve beyond current 60-80% recovery | Medium | High | Architecture already treats embeddings as sensitive; server-side mode is opt-in. Monitor academic literature for new attacks. |
| Storage growth exceeds projections | Medium | Medium | Aggressive compression, earlier partition archival, shorter default retention windows |

# **20\. Implementation Milestones**

| Milestone | Scope | Duration | Key Deliverables |
| :---- | :---- | :---- | :---- |
| M0: Foundation | PostgreSQL schema, append-only enforcement, basic encryption pipeline, hash chain | 4 weeks | messages table, did\_registry, envelope encrypt/decrypt, zstd compression, hash chain verification |
| M1: Trust Anchor | Merkle batch assembly, Kolme chain integration, merkle proof generation/verification | 3 weeks | merkle\_batches table, anchoring worker, proof API, Kolme transaction submission |
| M2: Access Gateway | DID authentication, ABAC authorization, RLS policies, rate limiting, audit logging | 4 weeks | Gateway service (Rust), RLS policies, access\_log, session management, WebSocket support |
| M3: Search & Indexing | Blind indexes, metadata search, full-text search on plaintext fields | 2 weeks | Blind index computation, GIN indexes, search API endpoints |
| M4: Escrow Integration | Escrow state machine, escrow-scoped messaging, settlement evidence, auditor keys | 4 weeks | escrows table, escrow state transitions, threshold key support, settlement receipts |
| M5: Vector Layer | pgvector integration, embedding pipeline, semantic search, anomaly detection | 3 weeks | message\_embeddings table, HNSW index, owner-side embedding SDK, anomaly scoring |
| M6: Graph Layer | Apache AGE integration, graph schema, trust propagation, capability matchmaking | 3 weeks | AGE graph schema, relationship triggers, Cypher query API, trust score computation |
| M7: Time-Series | TimescaleDB hypertables, continuous aggregates, alerting integration, billing metrics | 2 weeks | Hypertables, materialized views, Prometheus export, billing reconciliation |
| M8: Compliance | Crypto-shredding, retention policies, GDPR export, jurisdiction tagging, legal hold | 3 weeks | Retention worker, crypto-shred API, data export, jurisdiction enforcement |
| M9: Real-Time | WebSocket delivery, presence system, event webhooks, backpressure handling | 3 weeks | Delivery pipeline, presence table, webhook subscriptions, message\_delivery\_queue |
| M10: Scaling | Read replicas, connection pooling, partition management, archival pipeline | 3 weeks | PgBouncer config, pg\_partman setup, Parquet export, archival index |
| M11: Hardening | Security audit, chaos testing, performance benchmarking, documentation | 4 weeks | Security test suite, chaos scenarios, benchmark results, operator documentation |

**Total estimated duration:** 38 weeks (approximately 9.5 months) for full implementation. Milestones M0-M4 constitute a minimum viable product sufficient for basic agent messaging with encryption and on-chain anchoring.

# **Appendix A: PostgreSQL Extension Compatibility Matrix**

Verified compatibility of all required extensions in a single PostgreSQL 16+ instance:

| Extension A | Extension B | Compatible? | Notes |
| :---- | :---- | :---- | :---- |
| pgvector | TimescaleDB | Yes | No known conflicts. Both register custom types and index methods independently. |
| pgvector | Apache AGE | Yes | AGE uses a separate graph catalog; no namespace collision with vector types. |
| TimescaleDB | Apache AGE | Yes | TimescaleDB hypertables and AGE graphs operate on different table types. Test thoroughly with concurrent workloads. |
| pg\_partman | TimescaleDB | Partial | Use pg\_partman for non-hypertable partitions (messages). TimescaleDB manages its own chunk partitioning for hypertables. |
| pgcrypto | All | Yes | Utility functions only; no schema or type conflicts. |

# **Appendix B: Wire Format Specification**

## **B.1 Message Envelope Binary Format**

The canonical envelope serialization used for content hash computation and storage:

| Field | Offset | Size | Encoding | Description |
| :---- | :---- | :---- | :---- | :---- |
| Version | 0 | 1 byte | uint8 | Envelope format version (currently 0x01) |
| Flags | 1 | 1 byte | bitfield | Bit 0: compressed. Bit 1: multi-recipient. Bits 2-7: reserved. |
| Nonce | 2 | 12 bytes | raw | AES-256-GCM nonce/IV |
| AAD Hash | 14 | 32 bytes | raw | SHA-256 of Additional Authenticated Data |
| Wrapped Key Count | 46 | 2 bytes | uint16 BE | Number of wrapped key entries |
| Wrapped Keys | 48 | Variable | See B.2 | Array of wrapped key entries |
| Ciphertext Length | Variable | 4 bytes | uint32 BE | Length of encrypted body in bytes |
| Ciphertext | Variable | Variable | raw | AES-256-GCM encrypted message body |
| GCM Tag | Variable | 16 bytes | raw | Authentication tag |

## **B.2 Wrapped Key Entry Format**

| Field | Size | Encoding | Description |
| :---- | :---- | :---- | :---- |
| DID Length | 2 bytes | uint16 BE | Length of recipient DID string |
| DID | Variable | UTF-8 | Recipient DID |
| Wrapped CEK | 48 bytes | raw | X25519-HKDF-ChaCha20Poly1305 encrypted CEK |

# **Appendix C: Glossary**

| Term | Definition |
| :---- | :---- |
| AAD | Additional Authenticated Data — plaintext metadata included in GCM authentication but not encrypted |
| ABAC | Attribute-Based Access Control — authorization model based on attributes of requester, resource, and environment |
| AES-256-GCM | Advanced Encryption Standard with 256-bit key in Galois/Counter Mode — authenticated encryption algorithm |
| AGE | Apache Graph Extension — PostgreSQL extension adding openCypher graph query capabilities |
| ANN | Approximate Nearest Neighbor — efficient similarity search algorithm for vector databases |
| CEK | Content Encryption Key — per-message symmetric key used for body encryption |
| Crypto-Shredding | Data deletion technique: destroy encryption keys to make ciphertext permanently unrecoverable |
| DID | Decentralized Identifier — self-sovereign identity format (e.g., kamn:did:agent:xyz) |
| HNSW | Hierarchical Navigable Small World — graph-based ANN index algorithm used by pgvector |
| Merkle Root | Root hash of a merkle tree, committing to all leaf values in the tree |
| pgvector | PostgreSQL extension for vector similarity search |
| RLS | Row-Level Security — PostgreSQL feature enforcing per-row access control policies |
| TOAST | The Oversized-Attribute Storage Technique — PostgreSQL mechanism for storing large column values |
| X25519 | Elliptic curve Diffie-Hellman key agreement using Curve25519 |
| zstd | Zstandard — fast lossless compression algorithm with dictionary training support |

END OF DOCUMENT
