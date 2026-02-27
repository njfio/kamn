# KAMN Gaps and Issues Report — R58 Swarm Review

**As of:** R58 review, commit `4cb4d925` (2026-02-26)
**Baseline snapshot:** commit `4cb4d925` | **Rust LOC:** 231,145 (699 files) | **Shell+Python LOC:** 245,929 (1,509 files) | **Specs:** 2,694 files in 1,120 directories
**Review type:** Full codebase swarm review (6 parallel subagents)
**Test status:** 1 failure (`functional_shell_test_surface_ratio_non_regression_gate`), ~2,491 pass, 8 ignored

---

## 0. Review Methodology

Six parallel review agents examined:
1. **kamn-core** — architecture, state machines, data layer, error handling, duplication
2. **kamn-node** — server, auth middleware, Kolme integration, daemon modes
3. **Client crates** — kamn-sdk, kamn-agent-lib, kamn-cli, kamn-mcp-server
4. **Test/governance/CI** — test quality ratio, spec ecosystem, shell scripts, CI pipeline
5. **Security/crypto** — signing, auth, replay protection, input validation, secrets
6. **Kolme/E2E/fuzz/guards** — blockchain integration, e2e harness, fuzz targets, runtime guards

Each agent read 10-30+ source files and provided specific file:line references. This is a structural review, not an incremental delta from R57.

---

## 1. Consolidated Grade Card

### Per-Crate Grades

| Crate | Architecture | Implementation | Production Readiness |
|-------|-------------|----------------|---------------------|
| kamn-core | C+ | B- | B- |
| kamn-node | B- | B | C |
| kamn-sdk | B+ | B- | C+ |
| kamn-agent-lib | B | B | C |
| kamn-cli | B- | B- | C- |
| kamn-mcp-server | B | B- | C |
| kamn-kolme | B- | B | B |
| kamn-e2e-harness | C+ | C | C+ |
| kamn-runtime-guards | **A-** | **A-** | **A** |

### Cross-Cutting Grades

| Domain | Grade | Notes |
|--------|-------|-------|
| Crypto Implementation | B+ | secp256k1 via k256 is correct; zeroization discipline is strong |
| Auth Architecture | **D** | Single shared key; self-declared DIDs; no per-agent authentication |
| Input Validation | B+ | Body limits, UTF-8 checks, rate limiting, anti-spam all present |
| Secret Management | B | Good zeroization; undermined by Clone derive on signer adapter |
| Test Quality | B- | Behavioral tests are excellent; diluted by 70+ doc-guard test files |
| Governance Value | C+ | AGENTS.md is strong; governance has become self-referential |
| CI/CD Maturity | B+ | Two-tier CI with mutation + coverage gates; no live E2E |
| Spec Quality | B | Individual specs are substantive; volume (787 specs / 140K LOC) is excessive |

### Overall: **C+/B-**

The codebase has genuine engineering strengths — clean crypto, solid state machines, comprehensive middleware, zero-unwrap discipline — but is undermined by fundamental architectural gaps (auth model, persistence, identity) and governance overhead that has outgrown its utility.

---

## 2. Critical Issues (Must Fix)

### C-01: Single Shared Key Authentication
**Files:** `crates/kamn-node/src/service_api_endpoint/auth.rs:60-74`
**Impact:** All clients share one key pair. `x-kamn-sender-did` is self-declared and not cryptographically bound to the signature. Any key holder can impersonate any agent DID.
**Recommendation:** Per-agent key registry with DID-to-public-key binding.

### C-02: TLS Disabled by Default
**Files:** `crates/kamn-node/src/service_api_endpoint/server.rs:134`
**Impact:** Out-of-the-box, all API traffic including auth headers and signed payloads is plaintext. No startup warning.
**Recommendation:** Default to `require` in production modes; emit a loud warning if TLS is disabled.

### C-03: Replay Guard Lost on Restart
**Files:** `crates/kamn-node/src/service_api_endpoint.rs:513-567`
**Impact:** In-memory `BTreeSet` nonce store is ephemeral. Combined with static `state_hash`, all recent requests are replayable after process restart.
**Recommendation:** Persist nonce store to disk, or use monotonically increasing nonces that survive restart.

### C-04: Deterministic Identity Not Marked as Dev-Only
**Files:** `crates/kamn-agent-lib/src/identity.rs:13-23`
**Impact:** `from_agent_name("alice")` derives a secp256k1 private key from the name using custom FNV-based KDF. Anyone who knows the agent name can derive the signing key. No warning in docs, no feature gate.
**Recommendation:** Add `/// # Security Warning` doc comment; gate behind `#[cfg(feature = "dev-identity")]`.

### C-05: No Cryptographic DID-to-Key Binding
**Files:** `crates/kamn-core/src/did.rs`
**Impact:** DID `kamn:did:agent:alice` is just a name string with no public key hash or on-chain registration. No DID resolution verifies key ownership. DID squatting is trivial.
**Recommendation:** Embed public key hash in DID, or require on-chain registration.

### C-06: Shell LOC Exceeds Hard Ceiling
**Files:** `.ci/shell-loc-hard-ceiling.env`
**Impact:** Current 245,929 shell+Python LOC exceeds the declared hard ceiling of 130,000. Gate is either broken or waived.
**Recommendation:** Enforce the ceiling or formally raise it with justification.

---

## 3. Important Issues (Should Fix)

### I-01: 31+ Duplicated Journal/Snapshot Helpers
**Files:** `channel_models.rs`, `message_lifecycle.rs`, `task_operations.rs`, `durable_guard_store.rs`, `runtime_snapshot_store.rs`
**Impact:** `encode_journal_hex`, `decode_journal_hex`, `decode_journal_nibble`, `map_sqlite_store_error`, journal path construction, journal replay, journal record parsing, and recovery repair are copy-pasted across 5 files.
**Recommendation:** Extract into shared `snapshot_journal` module. Single highest-impact refactoring available.

### I-02: WebSocket Implementation Truncates at 125 Bytes
**Files:** `crates/kamn-sdk/src/service.rs:1010`
**Impact:** Only handles single-byte payload length. RFC 6455 requires 126 (2-byte) and 127 (8-byte) extended lengths. Any server event >125 bytes will silently truncate. Sec-WebSocket-Key is hardcoded.
**Recommendation:** Implement extended length handling; generate random WebSocket key per RFC.

### I-03: MCP Server Has Generic Schemas for All 21 Tools
**Files:** `crates/kamn-mcp-server/src/tools.rs:14-15`
**Impact:** Every tool declares `{"type":"object","additionalProperties":true}` for both input and output. MCP clients cannot discover parameter requirements.
**Recommendation:** Declare per-tool parameter schemas.

### I-04: `KolmeForkSecp256k1SignerAdapter` Derives Clone
**Files:** `crates/kamn-node/src/signer/signer_adapter.rs:8`
**Impact:** `.clone()` creates unzeroized copies of the signing key in memory, defeating the careful zeroization elsewhere.
**Recommendation:** Remove `Clone` derive.

### I-05: Message Lifecycle State Machine Bypass
**Files:** `crates/kamn-core/src/message_lifecycle.rs:990-1001, 211`
**Impact:** `expire_overdue_messages` bypasses the state transition validator. Two code paths with different validation rules. No path to reject messages from `Broadcast`/`Included` states.
**Recommendation:** Document the intentional design, or add explicit `Rejected` transitions from early states.

### I-06: Non-Monotonic Nonces Allow Post-TTL Replay
**Files:** `crates/kamn-node/src/service_api_endpoint.rs:514-567`
**Impact:** Nonce only needs to be unique per (sender_did, 900s window). After TTL, same nonce is accepted again. Combined with C-03 (restart wipe), this widens the replay window.
**Recommendation:** Require monotonically increasing nonces per sender.

### I-07: 19 `expect()` Calls in Bootstrap Path
**Files:** `crates/kamn-core/src/bootstrap.rs`
**Impact:** Production bootstrap path uses `expect()` which panics on failure instead of propagating errors.
**Recommendation:** Convert to `Result` propagation.

### I-08: E2E Harness Never Runs in CI
**Files:** `crates/kamn-e2e-harness/`, `.github/workflows/`
**Impact:** 15 scenarios written with real probe implementations, but `KAMN_E2E_SDK_DIRECT_LIVE` is never set in any CI workflow. Zero integration confidence.
**Recommendation:** Add CI job with live kamn-node + Kolme devnet.

### I-09: Kolme Commit ID Uses Payload Hash LENGTH, Not Value
**Files:** `crates/kamn-kolme/src/runtime_request_identity_policy.rs:18`
**Impact:** `payload_hash.trim().len()` means two different payloads with equal-length hashes produce the same commit ID. Intentional per tests, but creates collision risk.
**Recommendation:** Document the rationale explicitly; consider using actual hash value.

### I-10: 555-Line Monolithic Route Dispatcher
**Files:** `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs:355-909`
**Impact:** Single if/else chain for all routes. Three-way sync burden (handler, auth check, scope check). Unknown routes return 200 with snapshot data.
**Recommendation:** Per-route Axum handlers or a dispatch table.

---

## 4. Suggestions (Nice to Have)

| ID | Issue | Impact |
|----|-------|--------|
| S-01 | Move 70 `*_docs.rs` test files out of `cargo test` into a CI lint step | Reduces test LOC by ~11K, faster compile |
| S-02 | Split kamn-core (90+ modules) into 3-4 focused crates | Better layering, faster incremental builds |
| S-03 | Replace custom pipe-delimited snapshot format with serde | ~200 lines of manual serialize/parse per domain type |
| S-04 | Dual `next_state`/`transition_between` in task_lifecycle.rs → single table | Eliminates silent divergence risk |
| S-05 | Add `--help` and argument documentation to kamn-cli | Currently no help text, no usage, no arg names |
| S-06 | Make SDK timeout configurable (currently hardcoded 2s) | Too aggressive for production |
| S-07 | Fix FNV-1a vs FNV-1 ordering in `deterministic_body_tag` | Multiply-then-XOR is FNV-1, not FNV-1a |
| S-08 | Run fuzz targets in CI | 4 well-chosen targets exist but have never been executed |
| S-09 | Parallelize CI fast-gate (currently ~30 lanes sequential) | Reduce PR merge wall time |
| S-10 | Add `EscrowRead` and `AgentsWrite` to ServiceApiScope | Missing scope variants |
| S-11 | Cache log config at startup instead of per-log-event env reads | `logging.rs:100` reads env vars on every log emission |
| S-12 | Fix the `key_file` config parameter in MCP server (accepted but silently discarded) | Trust issue with users |
| S-13 | Remove `Debug` derive from signer adapter or implement manual redacting Debug | Prevent accidental key logging |
| S-14 | Deduplicate kamn-kolme JSON helpers (4 copies of `skip_ascii_whitespace`) | Copy-paste across api_codec, notification_policy, block_scan_policy |
| S-15 | Deduplicate kamn-e2e-harness driver code (~60-70% redundancy across 3 drivers) | Near-identical code in sdk_direct, cli_scripted, mcp_agent |

---

## 5. Structural Observations

### 5.1 The God Crate Problem
kamn-core exports 90+ public modules from a 909-line `lib.rs`. Domain models (channel, message, task, escrow), infrastructure adapters (sqlite, p2p, kolme commit), crypto (signing, ZK proofs), and data layer contracts (M0-M11) all live in one crate. This prevents independent compilation, testing, and versioning.

### 5.2 Governance-Feature Activity Imbalance
With 787 specs for ~140K LOC of production Rust, the project has created one spec per 180 lines. For a pre-mainnet project, a healthy ratio would be closer to 1:1000-2000. The governance infrastructure (documentation-guard tests, shell-to-Rust ratio tracking, spec volume guardrails, CI steps monitoring CI metrics) consumes significant development capacity that could be directed at closing the integration gaps.

### 5.3 The Strongest Crate Is the Smallest
kamn-runtime-guards (1,662 lines) received A-/A- grades — the highest in the entire project. It has zero dependencies, clean APIs, genuine security logic (anti-spam, replay detection, censorship detection, retention enforcement), and well-structured tests. It demonstrates what the rest of the codebase could look like with focused scope and minimal governance overhead.

### 5.4 Real vs. Perceived Completeness
The project presents as feature-complete (21 operations across CLI/MCP/SDK, 15 E2E scenarios, 787 specs) but the integration picture reveals gaps:
- Service API endpoints are now persisted but the daemon mode is a local simulation
- P2P gossip/consensus is contractual, not operational
- E2E scenarios are written but never execute
- Fuzz targets are well-chosen but have never run
- The data layer (M0-M11) has extensive contracts but limited runtime integration

---

## 6. What Works Well

Despite the gaps, the project has genuine engineering strengths that should not be overlooked:

1. **Zero `unwrap()` in production code** — across 231K lines of Rust, all production paths use proper error propagation
2. **secp256k1 signing is correct** — k256 usage, key zeroization, sign-then-verify pattern
3. **kamn-runtime-guards** — production-grade anti-spam, replay protection, and retention enforcement
4. **State machines are well-designed** — message lifecycle, task operations, escrow with property-based testing
5. **HTTP middleware is comprehensive** — 5-layer auth pipeline, rate limiting, concurrency control, anti-spam
6. **Recent SHA-256 migration** — content storage CIDs upgraded from FNV to SHA-256 (commit `e1dc9173`)
7. **Service API now persists state** — recent commits (`3ed1e3d1`, `fcf66ac9`, `43cfc298`, `6bd60507`) added persistence for messages, channels, bridges, agent profiles, and content lifecycle
8. **Clean clippy, clean fmt** — zero warnings, enforced via CI

---

## 7. Priority Roadmap

### Tier 1: Security Architecture (blocks adversarial deployment)
- C-01: Per-agent key authentication
- C-02: TLS on by default for production modes
- C-03: Persistent replay guard
- C-04/C-05: Cryptographic identity binding

### Tier 2: Integration Plumbing (blocks end-to-end flows)
- I-08: Live E2E testing in CI
- I-10: Route dispatcher refactoring
- Daemon mode → real P2P networking
- API → gossip/consensus relay path

### Tier 3: Code Quality (reduces maintenance burden)
- I-01: Extract shared snapshot journal module
- S-01: Separate governance tests from behavioral tests
- S-02: Split kamn-core into focused crates
- C-06: Address shell LOC ceiling breach

### Tier 4: Production Hardening
- I-02: WebSocket extended length support
- I-06: Monotonic nonces
- S-06: Configurable timeouts
- S-08: Run fuzz targets in CI

---

## 8. Trend Table

| Review | Commit | Rust LOC | Tests | Failures | Prod expect() | Shell+Py LOC | Ratio | Modules | Grade |
|--------|--------|----------|-------|----------|----------------|-------------|-------|---------|-------|
| R43 | 4af28701 | 148,022 | 2,888 | 0 | 0 | 122K | 0.82:1 | 82 | A+ |
| R44 | 92c9958c | 151,091 | 2,911 | 0 | 0 | 122K | 0.80:1 | 84 | A+ |
| R45 | ed913983 | 166,297 | 3,080 | 1 | 0 | 122K | 0.73:1 | 90 | A |
| R46 | f95a039e | 166,664 | 3,104 | 0 | 0 | 121K | 0.73:1 | 90 | A+ |
| R47 | 5570c2b8 | 171,635 | 3,188 | 0 | 0 | 121K | 0.70:1 | 90 | A- |
| R48 | f044f9bd | 176,259 | 3,258 | 0 | 0 | 142K | 0.80:1 | 91 | A |
| R49 | cc0f9f00 | 183,111 | 3,379 | 0 | 0 | 142K | 0.78:1 | 92 | A- |
| R50 | 49efe252 | 183,261 | 3,379 | 0 | 0 | 142K | 0.78:1 | 92 | B+ |
| R51 | 0be43ed5 | 193,077 | 3,616 | 0 | 0 | 142K | 0.74:1 | 92 | C |
| R52 | 8e0871cc | 198,094 | 3,160 | 2+ | 0 | 142K | 0.72:1 | 92 | B- |
| R53 | 982d52df | 199,171 | 3,249 | 0 | 0 | 142K | 0.71:1 | 92 | C- |
| R54 | 8be3cbf9 | 201,189 | 3,283 | 0 | 0 | 142K | 0.71:1 | 92 | C+ |
| R55 | 4d4da4bf | 207,586 | 3,400 | 1 | 418 | 142K | 0.68:1 | 93 | B |
| R56 | 6824f2ca | 214,075 | 3,967 | 0 | 467 | 142K | 0.66:1 | 93 | B+ |
| R57 | 6824f2ca | 214,075 | 3,967 | 0 | 467 | 142K | 0.66:1 | 93 | B |
| **R58** | **4cb4d925** | **231,145** | **~2,491** | **1** | **467** | **246K** | **1.06:1** | **93** | **C+** |

**Grade rationale (C+):** The codebase has grown 17K Rust LOC since R57 with meaningful persistence improvements (4 recent commits adding state persistence across restarts). The state machines, crypto, and runtime guards are genuinely well-engineered. However, the swarm review exposed fundamental architectural gaps that were previously underweighted: single-shared-key auth (D grade), no cryptographic DID binding, TLS off by default, ephemeral replay guard, and an identity system where knowing a name gives you the private key. The shell+Python LOC (246K) now exceeds Rust LOC (231K) — a ratio of 1.06:1, breaching the declared hard ceiling. The governance overhead (787 specs for 140K production LOC, 70 doc-guard test files, CI steps monitoring CI metrics) consumes capacity that should be directed at the Tier 1 security gaps. kamn-runtime-guards (A-/A-) proves the team can write excellent focused code; the challenge is extending that quality to the rest of the system while closing the integration gaps.

---

## 9. Governance-Feature Activity Ratio Markers (R58)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=2
- feature_activity_commit_count=8
- activity_total_commit_count=10
- governance_activity_commit_ratio=0.2
- feature_activity_commit_ratio=0.8

## 10. Spec-Volume Guardrail Markers (R58)

- spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1
- spec_volume_guardrail_baseline_spec_directory_count=1120
- spec_volume_guardrail_baseline_module_count=93
- spec_volume_guardrail_baseline_spec_to_module_ratio=12.04
- spec_volume_guardrail_target_spec_to_module_ratio_max=7.7
- spec_volume_guardrail_target_status=exceeded_guardrail
