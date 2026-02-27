# KAMN Gaps and Issues Report -- R59 Swarm Review

**As of:** R59 review, commit `4cb4d925` (2026-02-26)
**Baseline snapshot:** commit `4cb4d925` | **Rust LOC:** 231,145 (699 files) | **Shell+Python LOC:** 245,929 (1,509 files) | **Specs:** 2,694 files in 1,120 directories
**Review type:** Full codebase swarm review (6 parallel subagents)
**Test status:** ~2,491 pass, 1 failure (`functional_shell_test_surface_ratio_non_regression_gate`), 8 ignored

---

## 0. Review Methodology

Six parallel review agents examined the full codebase independently:

| Agent | Scope | Files Read | Key Areas |
|-------|-------|-----------|-----------|
| 1 | kamn-core | 30+ source files | Architecture, state machines, data layer, duplication, error handling |
| 2 | Security & Crypto | 20+ source files | Auth, identity, replay, TLS, signing, input validation, secrets |
| 3 | Runtime & Integration | 25+ source files | Daemon, observability, WebSocket, persistence, message flow, E2E |
| 4 | Tests & Governance | All test dirs + CI | Test quality ratio, spec volume, governance overhead, CI maturity |
| 5 | Client Crates | All of SDK, CLI, MCP, agent-lib | WebSocket, schemas, help, identity, production readiness |
| 6 | Kolme, Guards, Infra | kolme, guards, e2e, fuzz, deploy | Pipeline, anti-spam, scenarios, fuzz targets, Docker/K8s |

Each agent provided file:line references and independent grades. This report consolidates and cross-references their findings.

---

## 1. Consolidated Grade Card

### Per-Crate Grades

| Crate | Architecture | Implementation | Production Readiness |
|-------|-------------|----------------|---------------------|
| kamn-core | **C-** | **B** | **C** |
| kamn-node | B- | B | C |
| kamn-sdk | B | B- | C+ |
| kamn-agent-lib | A- | B | C+ |
| kamn-cli | B | C+ | C |
| kamn-mcp-server | B+ | B- | C |
| kamn-kolme | B+ | B | B |
| kamn-e2e-harness | C+ | C | B |
| kamn-runtime-guards | **A-** | **A-** | **A** |

### Cross-Cutting Grades

| Domain | Grade | Change vs R58 | Notes |
|--------|-------|---------------|-------|
| Crypto Implementation | B+ | = | secp256k1 via k256 correct; sign-then-verify; zeroization discipline strong |
| Auth Architecture | **D** | = | Single shared key; self-declared DIDs; no per-agent auth -- UNCHANGED |
| Input Validation | B+ | = | Body limits, UTF-8 checks, 3-layer rate limiting, anti-spam |
| Secret Management | **B-** | v | Good zeroization; Clone derive on signer + identity still not fixed |
| Runtime Functionality | **C+** | ^^ | Was D in Feb 24; daemon now does real relay; observability real; WebSocket persistent |
| Operational Readiness | **C-** | ^ | Was D; file persistence default-on; non-atomic writes; no WAL |
| Integration Completeness | **C** | ^^ | Was D; message relay pipeline E2E functional; E2E harness runs in CI |
| Test Quality | C+ | = | Behavioral tests excellent; diluted by 93 doc-guard files (17K LOC) |
| Governance Value | **D** | v | Self-referential loop; 14 contract schemas governing governance itself |
| CI/CD Maturity | B- | = | Three-tier pipeline; E2E live; flaky management; 50+ fast-gate steps |
| Spec Quality | B | = | Individual specs substantive; 789 directories for 318 production files |

### Overall: **C+**

The codebase has genuine engineering strengths (zero `unwrap()`, clean crypto, solid state machines, excellent runtime guards) and has made **meaningful progress since Feb 24** on three of the five previously-critical runtime gaps (observability, WebSocket, message delivery). However, the five critical security issues identified in R58 remain **completely unaddressed**, and the governance overhead continues to consume capacity that should target those gaps.

---

## 2. Critical Issues (Must Fix)

### C-01: Single Shared Key Authentication -- UNCHANGED
**Files:** `crates/kamn-node/src/service_api_endpoint/auth.rs:60-74`
**Impact:** All clients share one key pair. `x-kamn-sender-did` is self-declared. Any key holder impersonates any DID.
**Status since R58:** No change. Still critical.

### C-02: TLS Disabled by Default -- UNCHANGED
**Files:** `crates/kamn-node/src/service_api_endpoint/server.rs:134`
**Impact:** Default `Disabled`. No startup warning. All traffic plaintext out-of-box.
**Status since R58:** No change.

### C-03: Replay Guard Lost on Restart -- UNCHANGED
**Files:** `crates/kamn-node/src/service_api_endpoint.rs:513-567`
**Impact:** In-memory `BTreeSet` nonce store. Static `state_hash`. All recent requests replayable after restart.
**Status since R58:** No change.

### C-04: Deterministic Identity Not Dev-Gated -- UNCHANGED
**Files:** `crates/kamn-agent-lib/src/identity.rs:13-23`
**Impact:** `from_agent_name("alice")` derives secp256k1 private key from name. No warning, no feature gate. Used in production `connect()` path.
**Status since R58:** No change. No `#[cfg(feature = "dev-identity")]`, no security doc comment.

### C-05: No Cryptographic DID-to-Key Binding -- UNCHANGED
**Files:** `crates/kamn-core/src/did.rs:64-82`
**Impact:** DID is purely syntactic name string. No key hash, no resolver. DID squatting trivial.
**Status since R58:** No change.

### C-06: Non-Atomic State File Writes -- NEW
**Files:** `crates/kamn-node/src/service_api_endpoint/message_store.rs:175-176`
**Impact:** `fs::write()` truncates then writes. Crash mid-write corrupts entire state file. No WAL, no fsync.
**Recommendation:** Write to temp file then `rename()` (atomic on POSIX).

### C-07: WebSocket Truncates at 125 Bytes -- UNCHANGED
**Files:** `crates/kamn-sdk/src/service.rs:1010`
**Impact:** Only reads 7-bit payload length. RFC 6455 requires 126/127 extended lengths. Any frame >125 bytes silently corrupted.
**Recommendation:** Implement 2-byte and 8-byte extended length per RFC 6455.

---

## 3. Important Issues (Should Fix)

### I-01: 31+ Duplicated Journal/Snapshot Helpers -- UNCHANGED
**Files:** `message_lifecycle.rs`, `channel_models.rs`, `task_operations.rs`, `durable_guard_store.rs`, `runtime_snapshot_store.rs`
**Recommendation:** Extract shared `snapshot_codec` module.

### I-02: All 21 MCP Tool Schemas Are Generic -- UNCHANGED
**Files:** `crates/kamn-mcp-server/src/tools.rs:14-15`
**Impact:** `{"type":"object","additionalProperties":true}` for every tool. LLM clients cannot discover parameters.

### I-03: `KolmeForkSecp256k1SignerAdapter` Derives Clone -- UNCHANGED
**Files:** `crates/kamn-node/src/signer/signer_adapter.rs:8`
**Impact:** `.clone()` creates unzeroized key copies, defeating careful zeroization elsewhere.

### I-04: Message Lifecycle State Machine Bypass -- UNCHANGED
**Files:** `crates/kamn-core/src/message_lifecycle.rs:192-208, 990-1001`
**Impact:** `expire_overdue_messages` bypasses transition validator. `Validated` messages immune to time-based expiration. Asymmetry undocumented.

### I-05: Data Layer M0-M5 Have Zero Unit Tests -- UNCHANGED
**Files:** `crates/kamn-core/src/data_layer_m0.rs` through `data_layer_m5_vector_integration.rs`
**Impact:** 8 of 15 data layer modules untested. M0 (hash chain) and M1 (merkle proofs) -- the cryptographic foundations -- have no `#[test]` blocks.

### I-06: Non-Monotonic Nonces Allow Post-TTL Replay -- UNCHANGED
**Files:** `crates/kamn-node/src/service_api_endpoint.rs:514-567`

### I-07: MCP Server `key_file` Accepted but Discarded -- UNCHANGED
**Files:** `crates/kamn-mcp-server/src/main.rs:43`
**Impact:** `let _ = config.key_file.as_str();` -- Required argument parsed then thrown away.

### I-08: CLI Has No `--help` Flag
**Files:** `crates/kamn-cli/src/lib.rs:141-199`
**Impact:** `kamn-cli --help` returns "unsupported command: --help". No usage text, no command listing, no argument documentation.

### I-09: MCP Server No Max Content-Length
**Files:** `crates/kamn-mcp-server/src/main.rs:104`
**Impact:** `vec![0_u8; content_length]` with no upper bound. Malformed header could attempt 4GB allocation.

### I-10: Kolme JSON Helper Duplication (~150 lines)
**Files:** `api_codec.rs`, `notification_policy.rs`, `block_scan_policy.rs`, `flat_json_policy.rs`
**Impact:** `skip_ascii_whitespace` x3, `split_unquoted` x3, `parse_json_value` x2 -- maintenance divergence risk.

### I-11: E2E Driver Duplication (~5,000 lines)
**Files:** `sdk_direct.rs` (2,719), `cli_scripted.rs` (3,601), `mcp_agent.rs` (4,507)
**Impact:** Near-identical constants, helpers, probe fields across all three drivers.

### I-12: Kolme Commit ID Uses Payload Hash LENGTH Not Value
**Files:** `crates/kamn-kolme/src/runtime_request_identity_policy.rs:18,78`
**Impact:** `payload_hash.trim().len()` -- equal-length hashes produce same commit ID. Intentional per test (#1777) but creates collision risk. Parameter named `payload_hash` is misleading.

---

## 4. Significant Improvements Since Feb 24 Critical Analysis

Three of five critical runtime gaps have been resolved:

### Resolved: Daemon Tick Loop Now Processes Real Messages
**Before:** Incremented synthetic counters, ran hardcoded test scenarios.
**Now:** `execute_daemon_service_api_relay_tick_loop` drains real relay spool entries from NDJSON, forwards via TCP with signed auth headers, re-queues failures, projects statuses.
**Files:** `daemon_phase.rs:807-918`, `daemon_phase.rs:994`

### Resolved: Observability Now Computes Real Metrics
**Before:** Hardcoded zeros for latency, throughput, error rate.
**Now:** Two-layer system: Service API records every request with duration/status, computes real p50/p99/throughput/error-rate from bounded 1,024-sample window. Daemon observability derives from actual tick processing.
**Files:** `runtime_observability.rs:38-87`, `daemon_observability.rs:46-123`

### Resolved: WebSocket Now Maintains Persistent Connections
**Before:** Sent one message then closed.
**Now:** Persistent `tokio::select!` event loop with `tokio::broadcast` channel (256 buffer). Handles Ping/Pong, graceful close, lagged subscriber recovery. Publishes `message.created` events on `POST /v1/messages/send`.
**Files:** `websocket.rs:209-250`, `middleware_impl.rs:543`

### Resolved: End-to-End Message Delivery Pipeline
**Before:** API accepted messages but didn't deliver.
**Now:** Full relay path: API ingress -> local persist -> relay spool append -> daemon tick drain -> TCP forward to recipient -> recipient ingress as "relayed" -> delivery on recipient read.
**Files:** `middleware_impl.rs:498-562`, `daemon_phase.rs:832-916`, `middleware_impl.rs:462-497`, `message_store.rs:384-418`

### Improved: Storage Persistence Default-On
**Before:** Ephemeral in-memory, optional file.
**Now:** Auto-derived state file path from bind address, persistence on by default. Recent commits added persistence for bridges, channels, agent profiles, content lifecycle.
**Remaining gap:** Non-atomic `fs::write` (C-06 above).

### Improved: Lane Supervision
**Before:** No monitoring at all.
**Now:** Startup HTTP probes to `/healthz`, orderly shutdown with error propagation, stop contract validation.
**Remaining gap:** No inter-tick liveness checking during daemon execution.

### Confirmed: E2E Harness Runs in CI
**R58 claimed** `KAMN_E2E_SDK_DIRECT_LIVE` never set in CI -- **this was incorrect**.
**Verified:** `.github/workflows/e2e-live.yml:52` sets `KAMN_E2E_SDK_DIRECT_LIVE: "1"`. Runs on push to main + weekly. 3 KAMN nodes + Kolme, 15 scenarios, evidence collection.

---

## 5. Structural Observations

### 5.1 The God Crate Problem Persists
kamn-core exports 90+ modules from a 909-line `lib.rs`. Six natural extraction boundaries identified: data layer (M0-M11), P2P/transport, storage/persistence, crypto, bridges, operator/dashboard. No intermediate crate boundaries. A change to `discord_bridge.rs` recompiles crypto modules.

### 5.2 Governance Has Become Self-Referential (D Grade)
The `docs/review/README.md` (439 lines) defines 14 contract schemas with 22 `schema_version` markers. Several govern the governance process itself:
- Doc-Contract Test-File Non-Regression Ratchet: caps the number of governance test files
- Governance-Feature Activity Ratio: measures governance-vs-feature commit ratio
- Post-Publication Governance-Feature Target Reconciliation: reconciles past governance assessments

The governance remediation has itself become governance overhead. 93 doc-guard test files (17K LOC, 19% of test code) assert `doc.contains("marker string")` -- contributing zero defect-detection value. 5 governance policy files exist for 10 production crates.

### 5.3 Security Architecture Is the Blocking Gap
None of the five critical security issues (C-01 through C-05) have been addressed despite being identified in both the Feb 24 analysis and R58. The codebase grew 17K Rust LOC since R57 with meaningful persistence/runtime improvements, but the security architecture remains: single shared key, self-declared DIDs, name-derived private keys, TLS off, ephemeral replay guard. These block adversarial deployment.

### 5.4 The Strongest Crate Is Still the Smallest
kamn-runtime-guards (1,662 lines, A-/A-/A) remains the best-engineered crate: zero dependencies, clean focused APIs, real security logic. It demonstrates what focused engineering without governance overhead produces.

### 5.5 Real Progress on Runtime Gap
The project has moved from "specification validator" (Feb 24 D grades) to "functional single-node relay" (C+/C- grades). Three of five critical runtime findings resolved. The remaining gap is transitioning from single-hop TCP relay to distributed multi-node coordination using the P2P transport layer (which exists architecturally but isn't wired to message delivery).

---

## 6. What Works Well

Despite the gaps, genuine engineering strengths that should not be overlooked:

1. **Zero `unwrap()` in production code** -- across 231K lines, all production paths use proper error propagation
2. **secp256k1 signing is correct** -- k256 usage, key zeroization, sign-then-verify, length-prefixed canonical format
3. **kamn-runtime-guards** -- production-grade anti-spam with deposit/rate/suspension/dedup, censorship detection, replay protection, retention enforcement with resurfaced-record protection
4. **State machines** -- message lifecycle, task operations, escrow with property-based testing
5. **HTTP middleware** -- 5-layer auth pipeline with concurrency/global-rate/per-sender anti-spam
6. **Real relay pipeline** -- messages flow end-to-end from sender API through daemon tick to recipient node
7. **Real observability** -- computed from actual request processing with percentiles and health classification
8. **Persistent WebSocket** -- broadcast fanout with Ping/Pong, lagged subscriber recovery
9. **E2E in CI** -- live 3-node topology with Kolme, 15 scenarios, evidence artifacts
10. **Clean clippy, clean fmt** -- zero warnings, enforced via CI

---

## 7. Priority Roadmap

### Tier 1: Security Architecture (blocks adversarial deployment)
- C-01: Per-agent key authentication with DID-to-key binding
- C-02: TLS on by default for production modes with startup warning
- C-03: Persistent replay guard (write nonces to sqlite)
- C-04/C-05: Feature-gate `from_agent_name`; embed key hash in DID
- C-06: Atomic state file writes (write-to-temp-then-rename)

### Tier 2: Code Quality (reduces duplication burden)
- I-01: Extract shared snapshot journal module (highest-impact refactoring)
- I-10: Consolidate Kolme JSON helpers
- I-11: Extract shared E2E driver constants/helpers
- S-02 (below): Begin kamn-core split

### Tier 3: Client Crate Production Readiness
- C-07: WebSocket extended length support in SDK
- I-02: Per-tool MCP schemas
- I-07: Wire `key_file` to `from_did_and_signing_key` or remove
- I-08: Add `--help` to CLI
- I-09: Max Content-Length on MCP server

### Tier 4: Production Hardening
- I-05: Unit tests for M0-M5 data layer modules
- I-06: Monotonic nonces
- Run fuzz targets in CI
- K8s Service/Ingress definitions and probes
- Lane liveness monitoring during daemon execution

---

## 8. Suggestions (Nice to Have)

| ID | Issue | Impact |
|----|-------|--------|
| S-01 | Move 93 `*_docs.rs` test files to CI lint step | Reduce test LOC by ~17K, faster compile, cleaner test metrics |
| S-02 | Split kamn-core into 4-6 focused crates | Better layering, faster incremental builds |
| S-03 | Replace custom pipe-delimited snapshot format with serde | ~200 lines of manual serialize/parse per domain type |
| S-04 | Dual `next_state`/`transition_between` in task_lifecycle.rs -> single table | Eliminates silent divergence risk |
| S-05 | Add `AgentIdentity` zeroization (currently derives Clone on hex key) | Prevent uncontrolled key copies in memory |
| S-06 | Make SDK timeout configurable (currently hardcoded 2s) | Too aggressive for production |
| S-07 | Fix FNV-1a vs FNV-1 ordering in `derive_name_seed_bytes` | XOR-then-multiply is FNV-1, not FNV-1a |
| S-08 | Add bounded eviction to `seen_message_ids` in anti-spam | Unbounded HashSet grows forever |
| S-09 | Replace `f64` watchdog censorship ratio with integer math | Determinism for blockchain context |
| S-10 | Cache log config at startup instead of per-event env reads | `logging.rs:100` reads env on every emission |
| S-11 | CLI: unknown flags silently consumed as positional args | `kamn-cli send --endpont` silently swallowed |
| S-12 | MCP: success detection via `contains("\"ok\":true")` string match | Fragile; should parse JSON |
| S-13 | Document Kolme `payload_hash.len()` design decision | Currently misleading parameter name |
| S-14 | Wire channel/task/bridge events into WebSocket fanout | Only `message.created` published currently |
| S-15 | Add fuzz target for `split_unquoted` / `parse_flat_json_value_fields` | Hand-rolled parsers most benefit from fuzzing |

---

## 9. Trend Table

| Review | Commit | Rust LOC | Tests | Failures | Shell+Py LOC | Ratio | Modules | Grade |
|--------|--------|----------|-------|----------|-------------|-------|---------|-------|
| R43 | 4af28701 | 148,022 | 2,888 | 0 | 122K | 0.82:1 | 82 | A+ |
| R44 | 92c9958c | 151,091 | 2,911 | 0 | 122K | 0.80:1 | 84 | A+ |
| R45 | ed913983 | 166,297 | 3,080 | 1 | 122K | 0.73:1 | 90 | A |
| R46 | f95a039e | 166,664 | 3,104 | 0 | 121K | 0.73:1 | 90 | A+ |
| R47 | 5570c2b8 | 171,635 | 3,188 | 0 | 121K | 0.70:1 | 90 | A- |
| R48 | f044f9bd | 176,259 | 3,258 | 0 | 142K | 0.80:1 | 91 | A |
| R49 | cc0f9f00 | 183,111 | 3,379 | 0 | 142K | 0.78:1 | 92 | A- |
| R50 | 49efe252 | 183,261 | 3,379 | 0 | 142K | 0.78:1 | 92 | B+ |
| R51 | 0be43ed5 | 193,077 | 3,616 | 0 | 142K | 0.74:1 | 92 | C |
| R52 | 8e0871cc | 198,094 | 3,160 | 2+ | 142K | 0.72:1 | 92 | B- |
| R53 | 982d52df | 199,171 | 3,249 | 0 | 142K | 0.71:1 | 92 | C- |
| R54 | 8be3cbf9 | 201,189 | 3,283 | 0 | 142K | 0.71:1 | 92 | C+ |
| R55 | 4d4da4bf | 207,586 | 3,400 | 1 | 142K | 0.68:1 | 93 | B |
| R56 | 6824f2ca | 214,075 | 3,967 | 0 | 142K | 0.66:1 | 93 | B+ |
| R57 | 6824f2ca | 214,075 | 3,967 | 0 | 142K | 0.66:1 | 93 | B |
| R58 | 4cb4d925 | 231,145 | ~2,491 | 1 | 246K | 1.06:1 | 93 | C+ |
| **R59** | **4cb4d925** | **231,145** | **~2,491** | **1** | **246K** | **1.06:1** | **93** | **C+** |

**Grade rationale (C+):** Same commit baseline as R58. The swarm review confirmed R58's findings with deeper code-level evidence and identified meaningful runtime improvements since the Feb 24 critical analysis (3 of 5 critical runtime gaps resolved). However, all 5 critical security issues remain unaddressed, governance overhead continues to grow self-referentially, and the god-crate problem persists. The grade reflects the tension between genuine engineering quality (zero unwraps, correct crypto, excellent runtime guards) and fundamental architectural gaps (auth model, identity, duplication) that block production deployment.

---

## 10. Delta vs R58: What Changed, What Didn't

### Confirmed Fixed (3 items)
| R58/Feb24 Finding | Status |
|-------------------|--------|
| Daemon tick loop inert | **Fixed** -- real relay processing with TCP forwarding |
| Observability hardcoded to zero | **Fixed** -- two-layer real metrics from actual requests |
| WebSocket fire-and-forget | **Fixed** -- persistent event loop with broadcast fanout |

### Confirmed Unchanged (12 items)
| R58 Finding | Status |
|-------------|--------|
| C-01: Single shared key auth | Unchanged |
| C-02: TLS disabled by default | Unchanged |
| C-03: Replay guard ephemeral | Unchanged |
| C-04: Deterministic identity not dev-gated | Unchanged |
| C-05: No DID-to-key binding | Unchanged |
| I-01: Journal helper duplication | Unchanged |
| I-02: WebSocket 125-byte truncation | Unchanged |
| I-03: Generic MCP schemas | Unchanged |
| I-04: Signer Clone derive | Unchanged |
| I-05: Message lifecycle bypass | Unchanged |
| I-10: 555-line route dispatcher | Unchanged (20 routes, same pattern) |
| God-crate problem | Unchanged (90+ modules, 909-line lib.rs) |

### R58 Finding Corrected (1 item)
| R58 Finding | Correction |
|-------------|------------|
| I-08: "E2E harness never runs in CI" | **Incorrect** -- `e2e-live.yml:52` sets `KAMN_E2E_SDK_DIRECT_LIVE=1`. Runs on push to main + weekly. |

### New Findings (5 items)
| ID | Finding |
|----|---------|
| C-06 | Non-atomic `fs::write` for state persistence |
| I-08 | CLI has no `--help` flag |
| I-09 | MCP server no max Content-Length |
| I-11 | E2E driver duplication (~5,000 lines) |
| I-12 | Kolme commit ID uses `payload_hash.len()` not value |
