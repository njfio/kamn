# KAMN Gaps and Issues Report

**As of:** R46 review follow-up, issue `#5336` (2026-02-20)
**Rust LOC:** 166,664 | **Tests:** 3,104 passed, 0 failed, 12 ignored | **Shell LOC:** 120,958

---

## R45 Gap Resolution Summary

Six gaps were identified in R45. All six are now resolved after the R46 follow-up closure tranche:

| R45 Gap | Resolution | Commit/PR |
|---------|-----------|-----------|
| Flaky signer env guard test | Fully stabilized by lock-domain unification across managed-backend and signer tests | #5322, #5323, #5336 |
| data_layer_m10 monolith (2,001 lines) | Decomposed into root (560) + 5 submodules | #5325 |
| data_layer_postgres_execution_adapter (1,114 lines) | Decomposed into root (704) + 4 submodules | #5328 |
| Doc-contract consolidation (122 files) | Wave 4 tranche A+B: harness migration reduced include_str suite-file count to 69 | #5327, #5336 |
| Branch re-accumulation (134) | Manual cleanup wave executed; remote branch count now 54 | #5183, #5336 |
| Ignored tests (12) | Formal audit with explicit dispositions | #5329 |

---

## 1. Test Infrastructure

### 1.1 Signer Env Guard Flake — RESOLVED (R46 follow-up)

`regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch` at `signer_tests.rs:1521` now runs against a fully unified signer env lock domain across `main_tests`, `signer` module tests, and `signer/managed_backend` tests (#5336).

Verification commands with `--test-threads=16` passed for the original flaky regression and adjacent signer env suites.

**Resolution status:** closed.

### 1.2 Ignored Tests (12 total, audited)

Stable since R39 (7 consecutive reviews). All 12 ignored tests received formal disposition review in #5329:
- Tests either promoted to env-gated deep lanes with deferred promotion
- Explicit rationale documented per test

The inventory is now deliberately managed. Periodic re-evaluation recommended every 5 reviews.

### 1.3 Doc-Contract Test Files (69, -53 from R45)

Down from 122 to 69 through wave 3 + wave 4 tranche A+B harness consolidation (#5327, #5336). Wave 4 migrated 21 low-coupling suites into `docs_contract_wave4_harness.rs` with compatibility placeholders kept for stable test-target names.

Distribution:
- ~51 in `crates/kamn-core/tests/`
- ~16 in `crates/kamn-node/tests/`
- ~2 in `crates/kamn-sdk/tests/`

**Status:** `<70` target achieved in R46 follow-up.

### 1.4 Public API Surface Ratchet Gate

Baseline refreshed in R45 for 6 new module exports. Gate functioning as designed — requiring explicit baseline updates for each surface expansion. Stable in R46.

---

## 2. Data Layer Integration — COMPLETE

### 2.1 Infrastructure Activation (Phases 1–6) — RESOLVED R45

All 6 phases of the data layer PRD roadmap landed in R45. See `data-layer-roadmap.md` for full execution tracking.

### 2.2 Structural Improvements (R46)

Both data layer monoliths from R45 were decomposed:

**data_layer_m10_partition_archival.rs** — Decomposed (#5325):
```
data_layer_m10_partition_archival.rs      (560 lines, root coordinator)
  error.rs                                (266 lines)
  phase6.rs                               (740 lines)
  registry.rs                             (308 lines)
  retry.rs                                (85 lines)
  shared.rs                               (86 lines)
Total: 2,045 lines across 6 files
```

**data_layer_postgres_execution_adapter.rs** — Decomposed (#5328):
```
data_layer_postgres_execution_adapter.rs  (704 lines, core adapter)
  codec.rs                                (159 lines)
  error.rs                                (153 lines)
  migration.rs                            (64 lines)
  validation.rs                           (63 lines)
Total: 1,143 lines across 5 files
```

### 2.3 Next Frontier

The contract and infrastructure layers are complete. Execution of the next architectural milestone is now initiated under **#5338**, hardened by **#5340**, scenario-matrix stabilized by **#5342**, taxonomy/order contract-hardened by **#5344**, runtime-to-matrix taxonomy bridge-hardened by **#5346**, bounded load-profile matrix-hardened by **#5348**, role-profile matrix-hardened by **#5350**, ordered two-node role-pair matrix-hardened by **#5352**, bounded parallel role-pair lane-hardened by **#5354**, asymmetric parallel lane-hardened by **#5356**, parallel lane order-invariance hardened by **#5358**, multi-permutation invariance hardened by **#5360**, fingerprint-schema contracts hardened by **#5362**, topology-scope contracts hardened by **#5364**, topology-permutation invariance hardened by **#5366**, topology host-pair contracts hardened by **#5368**, topology host-pair directionality hardened by **#5370**, topology-id host-pair mapping hardened by **#5372**, and topology-id lane-set mapping hardened by **#5374** for the first env-gated slice of **live integration testing** against PostgreSQL and **daemon runtime end-to-end validation**. Multi-host distributed lanes remain follow-up work.

---

## 3. Structural Concerns

### 3.1 Large Files (12 production source files over 1,000 lines, -4 from R45)

Both R45 monolith decompositions removed files from the >1K list. The remaining 12 files have been stable for 5+ reviews:

| File | Lines | Stability |
|------|-------|-----------|
| `kamn-core/src/channel_models.rs` | 1,829 | stable 8+ reviews |
| `kamn-core/src/message_lifecycle.rs` | 1,782 | stable 8+ reviews |
| `kamn-core/src/task_operations.rs` | 1,733 | stable 8+ reviews |
| `kamn-core/src/p2p_transport/p2p_transport_live.rs` | 1,711 | stable 8+ reviews |
| `kamn-core/src/did_registry.rs` | 1,679 | stable 8+ reviews |
| `kamn-core/src/zk_message_proofs.rs` | 1,482 | stable 8+ reviews |
| `kamn-core/src/block_pipeline/block_pipeline_support.rs` | 1,214 | stable 8+ reviews |
| `kamn-core/src/data_layer_m5_vector_integration.rs` | 1,097 | stable 5+ reviews |
| `kamn-core/src/runtime_peer_coordination.rs` | 1,095 | stable 5+ reviews |
| `kamn-core/src/data_layer_m4_escrow_integration.rs` | 1,081 | stable 5+ reviews |
| `kamn-core/src/durable_guard_store.rs` | 1,038 | stable 8+ reviews |
| `kamn-core/src/signer_backend.rs` | 1,022 | stable 8+ reviews |

No urgent decomposition needed. All are established domain modules with clear ownership boundaries.

### 3.2 service_api_endpoint.rs — RESOLVED (R44)

Decomposed into root (665 lines) + auth, middleware_impl, payload, server, websocket submodules.

### 3.3 data_layer_m10_partition_archival.rs — RESOLVED (R46)

Decomposed into root (560 lines) + error, phase6, registry, retry, shared submodules.

### 3.4 data_layer_postgres_execution_adapter.rs — RESOLVED (R46)

Decomposed into root (704 lines) + codec, error, migration, validation submodules.

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0
- **Production `unwrap()` count:** 0
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings)

Zero production panicking paths maintained since R36 (10 consecutive reviews).

### 4.2 Shell Surface Health

Shell LOC decreased 596 lines (121,554 → 120,958) via:
- Shell-to-Python migration of ratio guardrail engine (#5315)
- Thin wrapper consolidation to exec-dispatch (#5318)
- Performance smoke wrapper thinning (#5318)

---

## 5. Spec and Governance

### 5.1 Spec Volume (Growing)

- **653 spec directories** (+10 from R45)
- Spec-to-module ratio: **7.3:1** (653 / 90 modules)

Ratio grew slightly since module count held flat while spec directories grew by 10. The data layer and performance baseline work each contributed spec artifacts.

### 5.2 Governance-Feature Activity Ratio

R46 commit activity: 3 feat / 25 governance (0.12:1). This is a deliberate consolidation cycle — governance commits directly addressed flagged R45 gaps (decomposition, test consolidation, audit). Context-appropriate.

### 5.3 Governance-Feature Activity Ratio Markers (R46)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=25
- feature_activity_commit_count=3
- activity_total_commit_count=28
- governance_activity_commit_ratio=0.8929
- feature_activity_commit_ratio=0.1071

---

## 6. Branch Hygiene — RESOLVED (R46 follow-up)

### 6.1 Branch Re-Accumulation (54 branches, resolved)

Follow-up cleanup runs completed successfully on 2026-02-20 (`actions/runs/22224132204` and `actions/runs/22230821547`), and measured remote branch count is now 54 (`git ls-remote --heads origin | wc -l`).

Total trajectory: 126 (R42) → 78 (R43) → 91 (R44) → 134 (R45) → 145 (R46) → 54 (R46 follow-up).

**Resolution status:** closed (target <100 exceeded).

---

## 7. Architecture

### 7.1 Crate Structure

The 4-crate monorepo structure remains sound:
- `kamn-core`: 90 modules, 167K LOC — domain logic + data layer infrastructure
- `kamn-node`: binary + runtime — deployment target
- `kamn-kolme`: wire format — protocol layer
- `kamn-sdk`: client libraries — external interface

### 7.2 kamn-core Module Count (90, stable)

90 public modules, unchanged from R45. The module surface stabilized after the R45 data layer activation sprint.

### 7.3 Typed-DID Migration — COMPLETE (R45)

All three waves (A, B, C) completed. No remaining raw `String` DID callsites in scope.

### 7.4 Data Layer Architecture Maturity

The data layer has completed its three-phase evolution:

1. **R38–R42:** Pure contract layer (15 modules, ~9.6K lines, zero I/O)
2. **R45:** Infrastructure activation (6 new modules, Phases 1–6 complete)
3. **R46:** Structural consolidation (both monoliths decomposed)

**Next milestone (initiated via #5338, hardened via #5340, matrix-stabilized via #5342, taxonomy/order-hardened via #5344, taxonomy-bridge-hardened via #5346, load-profile-hardened via #5348, role-profile-hardened via #5350, role-pair-hardened via #5352, bounded-parallel-lane-hardened via #5354, asymmetric-parallel-lane-hardened via #5356, order-invariance-hardened via #5358, multi-permutation-hardened via #5360, fingerprint-schema-hardened via #5362, topology-scope-hardened via #5364, topology-permutation-hardened via #5366, topology-host-pair-hardened via #5368, topology-host-pair-directionality-hardened via #5370, topology-id-host-pair-mapping-hardened via #5372, topology-id-lane-set-mapping-hardened via #5374):** expand PostgreSQL live-integration and daemon runtime end-to-end validation beyond the current env-gated bounded-profile same-host parallel matrix conformance slice into multi-host distributed lanes.

### 7.5 Performance Baseline Infrastructure (New)

R46 added:
- Benchmark workload fixture matrix (#4000) with schema contracts
- Performance baseline provenance and drift-seed contracts (#4001)
- Shell-surface mitigation for performance smoke (#5318)

This establishes infrastructure for deterministic performance regression detection.

---

## 8. Priority Summary

| Priority | Issue | Location | Effort | Status |
|----------|-------|----------|--------|--------|
| ~~Medium~~ | ~~data_layer_m10 monolith~~ | ~~kamn-core/src/~~ | ~~Medium~~ | **RESOLVED R46** |
| ~~Medium~~ | ~~data_layer_pg_adapter monolith~~ | ~~kamn-core/src/~~ | ~~Medium~~ | **RESOLVED R46** |
| ~~Medium~~ | ~~Doc-contract consolidation (122)~~ | ~~crates/*/tests/~~ | ~~Large~~ | **69 (-53) wave 3+4A+4B** |
| ~~Medium~~ | ~~Branch re-accumulation (145)~~ | ~~Remote branches~~ | ~~Small~~ | **RESOLVED (54 branches)** |
| ~~Medium~~ | ~~Signer env guard flake~~ | ~~main_tests/signer_tests.rs:1521~~ | ~~Small~~ | **RESOLVED (#5336)** |
| ~~Medium~~ | ~~Doc-contract wave 4 (remaining tranche)~~ | ~~69 remaining test files~~ | ~~Medium~~ | **RESOLVED (<70 achieved)** |
| Low | Shell LOC reduction | 120,958 lines | Ongoing | Improving (-596 this cycle) |
| Info | Spec-to-module ratio (7.3:1) | 653 specs / 90 modules | Ongoing | Growing |
| Info | Ignored tests (12, audited) | 2 test files | Periodic audit | Stable 7 cycles |
| Info | Prod files >1K (12, down from 16) | kamn-core/src/ | None urgent | Stable |

---

## 9. Trend Table

| Review | Commit | Rust LOC | Tests | Failures | Prod expect() | Shell LOC | Ratio | Modules | Grade |
|--------|--------|----------|-------|----------|----------------|-----------|-------|---------|-------|
| R39 | b00b948a | 139,470 | 2,754 | 0 | 0 | 123K | 0.88:1 | 82 | A |
| R40 | 9a3e3652 | 140,492 | 2,767 | 0 | 0 | 123K | 0.88:1 | 82 | A |
| R41 | 82850743 | 142,536 | 2,808 | 0 | 0 | 123K | 0.86:1 | 82 | A+ |
| R42 | 229fe3ee | 144,242 | 2,838 | 0 | 0 | 123K | 0.85:1 | 82 | A |
| R43 | 4af28701 | 148,022 | 2,888 | 0 | 0 | 122K | 0.82:1 | 82 | A+ |
| R44 | 92c9958c | 151,091 | 2,911 | 0 | 0 | 122K | 0.80:1 | 84 | A+ |
| R45 | ed913983 | 166,297 | 3,080 | 1 | 0 | 122K | 0.73:1 | 90 | A |
| **R46** | **f95a039e** | **166,664** | **3,104** | **0** | **0** | **121K** | **0.73:1** | **90** | **A+** |
