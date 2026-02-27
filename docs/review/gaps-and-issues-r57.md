# KAMN Gaps and Issues Report — Deep Structural Review

**As of:** R57 review, commit `6824f2ca` (2026-02-23)
**Baseline snapshot:** commit `6824f2ca` | **Rust LOC:** 214,075 | **Tests:** 3,967 passed, 0 failed, 12 ignored | **Shell LOC:** 141,965
**Review type:** Structural deep dive (0 commits since R56 — same codebase, different lens)

---

## 1. What This Review Does Differently

R43–R56 measured incremental progress: commit counts, LOC deltas, governance ratios, letter grades. This review examines the **structural reality** of the codebase: what actually runs, what is scaffolding, and where the remaining integration gaps lie — with particular attention to the Kolme blockchain integration that provides the system's security model.

---

## 2. Security Architecture — Dual-Layer Signing Model

KAMN implements a **two-tier signature architecture** — a design that makes sense given Kolme's role as the blockchain/finality layer.

### 2.1 Transport Layer: `baseline-v1` (Service API Authentication)

The Service API uses a deterministic signature profile for request authentication:

```rust
// crates/kamn-sdk/src/tcp.rs:232
format!("sig:ed25519:baseline-v1:{from}:{nonce}:{state_hash}:{}", body.len())
```

This is **not cryptographic** — it's a deterministic string combining sender DID, nonce, state hash, and body length. It provides:
- **Replay protection** via nonce deduplication (BTreeSet in auth middleware)
- **Request origin binding** via sender DID + state hash
- **Rate limiting and anti-spam** in the middleware pipeline

It does **not** provide: body integrity verification (only length, not content), cryptographic authentication, or non-repudiation.

This is an **intentional development placeholder**. The architecture supports pluggable signature profiles — unknown algorithms/profiles are rejected, and a migration path to real crypto (e.g., `secp256k1-v1`) exists in the profile matching infrastructure (`signature_profile.rs`).

**Status:** Medium priority gap. The Service API transport auth should eventually use real cryptographic signatures, but this is not a critical security failure because the actual security model relies on Kolme blockchain anchoring (see 2.2).

### 2.2 Blockchain Layer: Real Secp256k1 ECDSA (Kolme Commits)

The Kolme live runtime uses **production-grade cryptographic signing** via the `k256` crate:

```rust
// crates/kamn-node/src/signer/signer_adapter.rs
use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};

impl KolmeForkSecp256k1SignerAdapter {
    pub(crate) fn sign_message(&self, message: &str) -> Result<(String, u8), ConfigError> {
        let (signature, recovery_id) = self.signing_key
            .sign_recoverable(message.as_bytes())?;  // Real ECDSA signing
        // ...verify immediately after signing...
        Ok((signature_hex, recovery_id))
    }

    pub(crate) fn verify_message(&self, message: &str, signature_hex: &str, recovery_id: u8)
        -> Result<(), ConfigError> {
        let recovered = VerifyingKey::recover_from_msg(
            message.as_bytes(), &signature, recovery
        )?;  // Real ECDSA verification with key recovery
        if recovered != *self.signing_key.verifying_key() {
            return Err(...);
        }
        Ok(())
    }
}
```

Security features:
- **Real secp256k1 ECDSA** via `k256` (production dependency, not dev-only)
- **Key material zeroization** (`zeroize` crate) — private key bytes scrubbed after use
- **Signer backend routing** — key-role segregation (Operator, Admin, Treasury, Auditor) with fail-closed fallback
- **Managed external key source** — AWS KMS emulator integration path
- **Environment variable precedence enforcement** — rejects dual private-key sources

### 2.3 Security Guarantee Distribution

| Guarantee | Provider | Mechanism |
|-----------|----------|-----------|
| Message immutability | **Kolme blockchain** | Consensus + secp256k1 finality |
| Proof of inclusion | **Kolme blockchain** | Transaction in block at height |
| Finality | **Kolme blockchain** | FINAL/PENDING/FAILED status |
| Transport authentication | KAMN Service API | baseline-v1 (nonce + replay guard) |
| Replay prevention | KAMN + Kolme | Nonce tracking + idempotency keys |
| Key management | KAMN signer backend | Role segregation + secure provider routing |

The architecture correctly delegates cryptographic security to Kolme while KAMN handles coordination, transport auth, and orchestration.

---

## 3. kamn-node — A Real Server

### 3.1 What Works

kamn-node is a **fully functional binary** with multiple runtime modes:

- **API mode:** Axum HTTP server with TLS, 21 routes, 5-layer auth middleware (header extraction, signature verification, replay guard, scope policy, anti-spam), request budgeting, graceful shutdown
- **Daemon mode:** P2P gossip with tick-based consensus scheduling, peer lifecycle management, OS signal handling
- **Kolme-Live mode:** HTTP client to Kolme blockchain — fetches nonces, broadcasts signed runtime commits, polls finality status with exponential backoff
- **Full mode:** Supervisor orchestrating daemon + API together

### 3.2 Integration Gaps

- **Service API state is in-memory.** Messages, tasks, escrows, and channels are generated as response payloads but not persisted. The data layer modules (M0–M11) in kamn-core define postgres integration, but the Service API endpoint doesn't yet connect to them
- **Message relay is not end-to-end.** `POST /v1/messages/send` returns a receipt with a generated message_id, but the message content is not relayed to the recipient through the daemon's gossip layer
- **The daemon and API coupling is thin.** Both run as separate runtime components in Full mode, but the data flow from API-received operations into the consensus loop needs further integration

### 3.3 Assessment

kamn-node is a **functional HTTP server with production-grade middleware** and a **Kolme-integrated blockchain client with real cryptographic signing**. The remaining integration work is connecting the Service API endpoints to persistent storage and the daemon's consensus layer for end-to-end message delivery.

---

## 4. kamn-core — Substantial Production Logic

### 4.1 Source Code (68,800 lines, 147 files)

Approximately 85% real business logic:

| Category | Approx Lines | Examples |
|----------|-------------|---------|
| State machines / lifecycle | ~20,000 | message_lifecycle (1,784), task_operations (1,735), channel_models (1,831) |
| Data layer integrations | ~18,000 | M0–M11 modules (postgres, escrow, vector, realtime delivery) |
| Crypto / signing / proofs | ~3,500 | signer_backend (1,022), zk_message_proofs (1,482), signature_profile |
| P2P / transport | ~6,000 | runtime_peer_coordination (1,095), transport modules |
| Bootstrap / config | ~5,000 | bootstrap (967), configuration modules |

Real algorithms present: task dependency cycle detection, message lifecycle state machines with transition validation, channel membership management, content retention/redaction, peer coordination.

### 4.2 Test Code (71,955 lines, 286 files)

| Category | Files | Lines (approx) | What They Test |
|----------|-------|-------------|---------------|
| Behavioral / functional | ~128 | ~32,000 (45%) | State transitions, API contracts, data layer logic |
| Governance / doc validation | ~155 | ~40,000 (55%) | Document markers, governance taxonomy, review artifacts |

The governance tests use `include_str!()` to embed documentation and assert specific headings/markers are present. These do not validate code behavior — they enforce documentation structure. This is the mechanism by which the governance loop self-sustains: adding a marker requires a test that validates the marker.

---

## 5. Kolme Integration — The Full Picture

### 5.1 The kamn-kolme Crate (5,695 src lines)

Middleware boundary between KAMN and Kolme blockchain:
- Runtime request identity normalization
- API codec contracts for Kolme's `/get-next-nonce` and `/broadcast` endpoints
- Transport policies (TLS, HTTP, retry logic)
- Broadcast payload normalization for signed envelopes
- Service API scope taxonomy (`ServiceApiScope` enum — 10 scope variants)

### 5.2 Runtime Commit Flow

```
kamn-node (kolme-live mode)
  → runtime_kolme_live.rs (orchestration)
    → signer/signer_adapter.rs (secp256k1 ECDSA signing via k256)
    → wire_payload.rs (JSON message with pubkey, nonce, messages array)
    → KolmeRuntimeCommitHttpTransport
      → GET  /get-next-nonce (fetch nonce from Kolme)
      → POST /broadcast (submit secp256k1-signed commit)
      → GET  /runtime-commit/status (poll finality)
```

### 5.3 Proof Verification Flow

```
Client → KamnAgentHandle::verify_proof(message_id, receipt)
  → KolmeClient validates receipt structure
  → Maps finality status: FINAL → verified=true
  → Returns KolmeProofVerification { message_id, block_height, finality, verified }
```

The client trusts Kolme's finality oracle. KAMN provides the coordination and verification interface; Kolme provides the cryptographic guarantees.

### 5.4 ZK Proof Planning (Future)

`zk_message_proofs.rs` (1,482 lines) is a **planning/evaluation framework** for ZK proof adoption, not a production proof system. It evaluates candidates (Groth16, Plonkish, Stark) and models witness commitment/validator consensus. This is Phase 4 future work.

---

## 6. The E2E Harness — Capable But Not Yet Live-Tested

### 6.1 Architecture

15 scenarios across 3 drivers (SDK-Direct, CLI-Scripted, MCP-Agent). Each probe connects to a running kamn-node via `KamnAgentHandle` and makes real HTTP calls.

### 6.2 The Toggle Gap

Live execution requires `KAMN_E2E_SDK_DIRECT_LIVE=true` (disabled by default). When disabled, probes skip and return hardcoded `"pass"`. The 455 e2e-harness tests validate toggle behavior and probe structure — they do not execute against a running service. No CI pipeline runs live scenarios.

**Status:** The probe code is written and the HTTP client is real. The gap is operationalization — setting up a running kamn-node + Kolme node in CI and enabling the toggle.

---

## 7. kamn-sdk and kamn-agent-lib — Real Network Clients

### 7.1 kamn-sdk (2,800 src lines)

Real TCP HTTP/1.1 client (manual construction, no http crate), WebSocket upgrade support, 2-second request timeout. Makes actual `TcpStream::connect()` calls.

### 7.2 kamn-agent-lib (1,028 src lines)

Coordinating facade over kamn-sdk. Manages monotonic nonce counter (thread-safe), builds auth headers, derives agent identity deterministically from names. All 18 operations proxy to real HTTP calls.

### 7.3 Agent Identity Model

Agent identity is derived deterministically from names (`"alice"` → `kamn:did:agent:alice`). This is a development convenience. For production, the managed external key source and signer backend routing provide the path to hardware-backed identity (AWS KMS, HSM).

---

## 8. Quantitative Summary

### 8.1 Code Composition

| Component | LOC | Notes |
|-----------|-----|-------|
| kamn-core src | 68,800 | ~85% real business logic |
| kamn-core tests | 71,955 | ~55% governance contracts, ~45% behavioral |
| kamn-node | 32,128 src | Real runnable server + Kolme client |
| kamn-e2e-harness | 13,815 src | 15 scenario probes (untested live) |
| kamn-kolme | 5,695 src | Kolme integration policies |
| kamn-sdk | 2,800 src | Real TCP/HTTP client |
| kamn-agent-lib | 1,028 src | Coordinating facade |
| kamn-mcp-server | 1,281 src | MCP JSON-RPC stdio |
| kamn-cli | 803 src | 21 CLI subcommands |
| Shell scripts | 141,965 | Unchanged 9 reviews |

### 8.2 Functional Readiness

| Component | Status | What Works | What Remains |
|-----------|--------|------------|-------------|
| HTTP server | **Functional** | TCP/TLS, 21 routes, auth middleware, anti-spam | Persistent state, message relay |
| Kolme signing | **Production** | secp256k1 ECDSA via k256, key zeroization | — |
| Service API auth | **Development** | Replay guard, scope policy, rate limiting | Crypto upgrade from baseline-v1 |
| State machines | **Implemented** | Message lifecycle, task ops, channels | Integration with Service API |
| Data layers | **Designed** | M0–M11 postgres modules in kamn-core | Connection from API to persistence |
| E2E scenarios | **Written** | 15/15 probe code, real HTTP clients | Live execution, CI pipeline |
| Portable agent surface | **Functional** | 21 CLI/MCP/SDK operations | End-to-end state changes |

---

## 9. Priority Gap Analysis (Corrected)

| Priority | Gap | Detail | Impact |
|----------|-----|--------|--------|
| **High** | Service API → persistent storage | Responses are generated, not stored; state lost on restart | API is a validator, not a stateful service |
| **High** | End-to-end message delivery | send_message returns receipt but doesn't relay to recipient | Core use case not complete |
| **High** | Live E2E testing | 15 scenarios written, 0 tested against running service in CI | No integration confidence |
| Medium | baseline-v1 → real crypto for Service API | Transport auth is deterministic strings, not cryptographic | Acceptable short-term (Kolme handles anchoring); should upgrade |
| Medium | Test LOC exceeds source LOC | 72K tests vs 69K source in kamn-core; 55% governance | Inflated metrics, maintenance burden |
| Medium | Shell infrastructure (142K) unchanged 9 reviews | Neither maintained nor removed | Dead code risk |
| Medium | 467 production expect() calls | Growing without audit | Potential panic paths |
| Low | Data layer M0–M11 integration | Postgres modules exist in kamn-core but not connected to API | Persistence architecture ready but unwired |

---

## 10. Critical Assessment: A Well-Engineered Coordination Layer

KAMN is a **well-structured Rust monorepo** that correctly delegates blockchain security to Kolme while implementing the coordination, transport, and orchestration layers:

**Genuine strengths:**
- **Real cryptographic signing for Kolme** — secp256k1 ECDSA via k256, key zeroization, managed signer routing
- **Production HTTP server** — Axum with TLS, comprehensive auth middleware, anti-spam
- **Clean architectural separation** — Kolme handles finality/immutability, KAMN handles coordination/transport
- **Comprehensive state machine designs** — message lifecycle, task orchestration, channel management
- **Broad crate coverage** — 21 operations across CLI/MCP/SDK, 15 e2e scenarios
- **Zero unwrap(), zero clippy, green main**

**What needs integration work:**
- **Service API to persistence** — connecting the 21 API endpoints to kamn-core's data layer modules (M0–M11) so operations produce durable state
- **Message relay through daemon** — the path from API-received messages into gossip/consensus
- **Live E2E execution** — running the 15 scenarios against a kamn-node + Kolme node in CI
- **Service API crypto upgrade** — replacing baseline-v1 with real signatures (the infrastructure supports this via pluggable profiles)

The codebase is not "missing its core" — it has the correct security architecture (Kolme-anchored cryptographic finality). What it needs is **integration plumbing** between the well-implemented layers: API → persistence → relay → finality.

---

## 11. Trend Table

| Review | Commit | Rust LOC | Tests | Failures | Prod expect() | Shell LOC | Ratio | Modules | Grade |
|--------|--------|----------|-------|----------|----------------|-----------|-------|---------|-------|
| R43 | 4af28701 | 148,022 | 2,888 | 0 | 0 | 122K | 0.82:1 | 82 | A+ |
| R44 | 92c9958c | 151,091 | 2,911 | 0 | 0 | 122K | 0.80:1 | 84 | A+ |
| R45 | ed913983 | 166,297 | 3,080 | 1 | 0 | 122K | 0.73:1 | 90 | A |
| R46 | f95a039e | 166,664 | 3,104 | 0 | 0 | 121K | 0.73:1 | 90 | A+ |
| R47 | 5570c2b8 | 171,635 | 3,188 | 0 | 0 | 121K | 0.70:1 | 90 | A- |
| R48 | f044f9bd | 176,259 | 3,258 | 0 | 0 | 142K | 0.80:1 | 91 | A |
| R49 | cc0f9f00 | 183,111 | 3,379 | 0 | 0 | 142K | 0.78:1 | 92 | A- |
| R50 | 49efe252 | 183,261 | 3,379 | 0 | 0 | 142K | 0.78:1 | 92 | B+ |
| R51 | 0be43ed5 | 193,077 | 3,616 | 0 | 0 | 142K | 0.74:1 | 92 | C |
| R52 | 8e0871cc | 198,094 | 3,160 | 2+compile | 0 | 142K | 0.72:1 | 92 | B- |
| R53 | 982d52df | 199,171 | 3,249 | 0 | 0 | 142K | 0.71:1 | 92 | C- |
| R54 | 8be3cbf9 | 201,189 | 3,283 | 0 | 0 | 142K | 0.71:1 | 92 | C+ |
| R55 | 4d4da4bf | 207,586 | 3,400 | 1 | 418 | 142K | 0.68:1 | 93 | B |
| R56 | 6824f2ca | 214,075 | 3,967 | 0 | 467 | 142K | 0.66:1 | 93 | B+ |
| **R57** | **6824f2ca** | **214,075** | **3,967** | **0** | **467** | **142K** | **0.66:1** | **93** | **B** |

**Grade rationale (B):** Same codebase as R56 (B+), evaluated structurally. The architecture is sounder than initially assessed: Kolme provides real secp256k1 ECDSA for blockchain finality, and the baseline-v1 transport signature is an intentional development placeholder with a clear migration path. The signer backend has production-grade key management (role segregation, zeroization, managed external providers). kamn-core contains ~85% real business logic. The HTTP server, middleware pipeline, and 21-operation portable-agent surface are functional. Deductions from B+: no persistent Service API state, no end-to-end message delivery, zero live E2E test executions in CI, 55% of test code validates documentation rather than behavior, and 142K shell lines unchanged for 9 reviews. The foundation is solid and correctly architected — the remaining work is integration plumbing between well-implemented layers.

---

## 12. Governance-Feature Activity Ratio Markers (R57)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=1
- feature_activity_commit_count=1
- activity_total_commit_count=2
- governance_activity_commit_ratio=0.5
- feature_activity_commit_ratio=0.5

## 13. Spec-Volume Guardrail Markers (R57)

- spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1
- spec_volume_guardrail_baseline_spec_directory_count=697
- spec_volume_guardrail_baseline_module_count=93
- spec_volume_guardrail_baseline_spec_to_module_ratio=7.49
- spec_volume_guardrail_target_spec_to_module_ratio_max=7.7
- spec_volume_guardrail_target_status=within_guardrail
