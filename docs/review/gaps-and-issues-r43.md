# KAMN Gaps and Issues Report

**As of:** R43 review, commit `4af28701` (2026-02-19)
**Rust LOC:** 148,022 | **Tests:** 2,888 passed, 12 ignored | **Shell LOC:** 121,652

---

## R42 Gap Resolution Summary

Six gaps were identified in R42. All six have been addressed in R43:

| R42 Gap | Resolution | Commit/PR |
|---------|-----------|-----------|
| Signer test mutex poisoning | Lock uses `.unwrap_or_else(\|e\| e.into_inner())` | #5199 |
| observability_endpoint.rs monolith (1,454 lines) | Decomposed into root + 4 submodules | #5180 |
| PRD file at repo root | Relocated to `docs/planning/` | #5179 |
| Draft specs 3881-3887 | All 5 advanced from Draft to Reviewed | #5179 |
| Branch accumulation (126 branches) | Reduced to 78 + auto-cleanup workflow added | #5183 |
| Ignored tests (13) | Reduced to 12 | #5179 |

---

## 1. Test Infrastructure

### 1.1 Signer Test Mutex Poisoning — RESOLVED

Lock poison recovery implemented in `main_tests.rs` via `lock_signer_env_guard()` returning `.unwrap_or_else(|error| error.into_inner())`. Parallel test cascading no longer occurs.

### 1.2 Ignored Tests (12 total, -1 from R42)

Down from 13 to 12. Remaining ignored tests are spread across:
- `crates/kamn-core/tests/input_mutation_coverage_guided.rs`
- `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`

These have been stable for 10+ reviews. Should be periodically audited for re-enablement or removal.

### 1.3 Doc-Contract Test Proliferation (127 files)

127 test files use `include_str!()` to load markdown docs and assert marker presence (+2 from R42). The pattern remains deeply entrenched:
- ~110 in `crates/kamn-core/tests/`
- ~15 in `crates/kamn-node/tests/`
- ~2 in `crates/kamn-sdk/tests/`

**Progress:** R43 began consolidating doc-contract suites into shared harness matrices (#5184, #5192, #5193). This is the right direction — migrating from one-file-per-doc toward data-driven harness slices. The overall file count grew slightly (+2) because migration creates harness files before removing originals, but the trend is positive.

**Recommendation:** Continue harness migration. Target reducing below 100 doc-contract files by R46.

### 1.4 Shell Test Migration Wave 1 (New)

20 shell test scripts were migrated to Rust suites in R43 (529 → 509 scripts). Migration is tracked in `shell_test_surface_migration_wave1.rs` (1,524 lines). This is a large new test file, but it replaces distributed shell scripts and should be monitored for potential decomposition as migration continues.

### 1.5 Public API Surface Ratchet Gate (New)

A new `public_api_surface_policy` test enforces baseline checks against public API surface growth. It uses configurable warn/fail thresholds with waiver support. The baseline fixture at `fixtures/ci/kamn_core_public_api_surface_baseline.env` tracks 82 modules and 1,736 public items. Any module or public item additions require explicit baseline updates.

---

## 2. Data Layer Integration

### 2.1 Integration Status (11/15 integrated with core types)

No data_layer changes landed in R43. Integration has been stalled at this level since R41 (3 review cycles).

| Module | Status | Imports From Core |
|--------|--------|-------------------|
| M0 (append-only ledger) | INTEGRATED | `CanonicalMessageEnvelope`, `DirectMessageCiphertext` |
| M1 (merkle anchoring) | INTEGRATED | `kolme_runtime_commit`, `AgentDid` |
| M2 (gateway access) | INTEGRATED | `AgentDid`, `KamnDid`, `KamnDidError` |
| M3 (blind-index search) | INTEGRATED | `ContentRetrievalError/Request/Scope` |
| M4 (escrow) | INTEGRATED | `EscrowStatus` |
| M5 (vector embeddings) | INTEGRATED | `AgentDid`, `ContentLifecycleManager`, `ContentRetentionClass`, `KamnDid` |
| M6 (graph trust) | INTEGRATED | `KamnDid` |
| M7 (timeseries telemetry) | INTEGRATED | `ObservabilityMonitor/Report/Sample/SloProfile/Snapshot` |
| M8 (compliance lifecycle) | INTEGRATED | `ContentLifecycleManager`, `ContentRetentionClass`, `KamnDid` |
| M9 (realtime delivery) | INTEGRATED | `AgentDid`, `AntiSpamEngine`, `ChannelStore`, `BackpressureController`, `PeerLifecycleState`, `KamnDid` |
| M10 (partition archival) | INTEGRATED | `DataLayerM8ComplianceRegistry`, `KamnDid` |
| M11 closure evidence | CROSS-DL ONLY | Only imports other `DataLayer*` types |
| **M11 hardening readiness** | **STANDALONE** | **No `use crate::` imports** |
| **PRD conformance** | **STANDALONE** | **No `use crate::` imports** |
| Shell neutral policy | CROSS-DL ONLY | Only imports `DataLayerPrdCriticalScenarioConformanceReport` |

### 2.2 Remaining Integration Gaps

**M11 hardening readiness** and **PRD conformance** remain standalone. Both are meta-assessment modules where standalone status is acceptable by design.

---

## 3. Structural Concerns

### 3.1 Large Files (25 files over 1,000 lines, -1 from R42)

`observability_endpoint.rs` dropped off the list (1,454 → 428 lines after decomposition). `signer.rs` also dropped off (1,033 → 908 lines). One new test file joined (`shell_test_surface_migration_wave1.rs` at 1,524 lines).

**Production source files over 1,000 lines:**

| File | Lines | Change from R42 |
|------|-------|-----------------|
| `kamn-core/src/channel_models.rs` | 1,829 | unchanged |
| `kamn-node/src/service_api_endpoint.rs` | 1,813 | unchanged, now largest kamn-node file |
| `kamn-core/src/message_lifecycle.rs` | 1,782 | unchanged |
| `kamn-core/src/task_operations.rs` | 1,733 | unchanged |
| `kamn-core/src/p2p_transport/p2p_transport_live.rs` | 1,711 | unchanged |
| `kamn-core/src/did_registry.rs` | 1,679 | unchanged |
| `kamn-core/src/zk_message_proofs.rs` | 1,482 | unchanged |
| `kamn-core/src/runtime_tests.rs` | 1,418 | test file |
| `kamn-core/src/block_pipeline/block_pipeline_support.rs` | 1,214 | unchanged |
| `kamn-core/src/data_layer_m5_vector_integration.rs` | 1,097 | unchanged |
| `kamn-core/src/data_layer_m4_escrow_integration.rs` | 1,081 | unchanged |
| `kamn-core/src/durable_guard_store.rs` | 1,038 | unchanged |
| `kamn-core/src/signer_backend.rs` | 1,022 | unchanged |
| `kamn-core/src/runtime_peer_coordination.rs` | 1,010 | unchanged |

**Test files over 1,000 lines:** 11 files (service_api_endpoint_tests at 2,167 is the largest).

### 3.2 observability_endpoint.rs — RESOLVED

Decomposed in R43 (#5180) after being flagged since R31 (11 consecutive reviews):

```
Before: observability_endpoint.rs (1,454 lines, monolith)
After:  observability_endpoint.rs     (428 lines, root coordinator)
        observability_endpoint/
          endpoint_server.rs           (278 lines)
          payload_contract.rs          (451 lines)
          payload_render.rs            (174 lines)
          tls_mode.rs                  (167 lines)
```

Total: 1,498 lines across 5 files with clean separation of concerns.

### 3.3 service_api_endpoint.rs (1,813 lines, new top concern)

With `observability_endpoint.rs` resolved, `service_api_endpoint.rs` is now the largest kamn-node source file. It combines HTTP endpoint handlers, request parsing, and response construction. A similar decomposition into submodules would improve maintainability.

### 3.4 Public API Surface (1,412 items, ratchet-gated)

kamn-core exports 1,412 public items across 82 modules. This is now tracked by the `public_api_surface_policy` ratchet gate with configurable warn/fail thresholds. Growth beyond the baseline requires explicit fixture updates.

### 3.5 Signer Subsystem (2,488 lines, reversed growth)

`signer.rs` shrank from 1,033 to 908 lines in R43, reversing its R41→R42 growth. The 5-file decomposition remains healthy:
- `signer.rs`: 908 lines (was 1,033)
- `signer/managed_backend.rs`: 690 lines
- `signer/signer_policy.rs`: 593 lines
- `signer/signer_adapter.rs`: 209 lines
- `signer/nonce.rs`: 88 lines
- **Total:** 2,488 lines (was 2,513)

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0
- **Production `unwrap()` count:** 0
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings with `-D warnings`)

Zero production panicking paths maintained since R36 (7 consecutive reviews).

### 4.2 Symlink Health (Clean)

402 symlinks in `scripts/`, zero broken. The exec_registry.json (221 entries) has zero dead references.

---

## 5. Spec and Governance

### 5.1 Spec Volume (Growing)

- **591 spec directories** (+29 from R42)
- **0 specs** missing Test Mapping sections
- **29 milestones** defined

The spec-to-module ratio is **7.2:1** (591 specs / 82 modules), up from 6.9:1 in R42. Spec directories are growing faster than production modules. While specs map to issues rather than modules, this ratio indicates increasing governance overhead.

### 5.2 Draft Specs — RESOLVED

The 5 previously-flagged Draft specs (3881, 3884-3887) have all been advanced to Reviewed status in R43. No remaining stale drafts identified.

### 5.3 PRD File — RESOLVED

`kamn-data-layer-prd.docx.md` relocated from repo root to `docs/planning/kamn-data-layer-prd.docx.md`.

### 5.4 Shell Test Scripts vs Rust Tests

- **509 shell test scripts** (`test_*.sh`, non-symlink) — down from 529 (-20)
- **299 Rust test files** (`crates/*/tests/*.rs`) — up from 292 (+7)

The shell-to-Rust test file ratio improved to **1.70:1** (was 1.81:1). The wave-1 shell test migration (#5189) is making measurable progress.

### 5.5 Governance Activity Ratio (New Concern)

R43 commit activity was ~95% governance/maintenance and ~5% new features. Only 2 of 53 commits delivered new runtime capabilities (signer rotation freshness #3956, quorum policy markers #3958). The project risks becoming a governance machine that produces specs and tests about itself rather than shipping new capabilities.

### 5.6 Governance-Feature Activity Ratio Markers

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=51
- feature_activity_commit_count=2
- activity_total_commit_count=53
- governance_activity_commit_ratio=0.9623
- feature_activity_commit_ratio=0.0377

---

## 6. Branch Hygiene — RESOLVED

Remote branches reduced from 126 to **78** (-38%). A safe merged-branch cleanup workflow was added (#5183) to automate stale branch deletion. This directly addresses the R42 concern about branch accumulation eroding the R41 pruning wave.

---

## 7. Architecture

### 7.1 Crate Structure

The 4-crate monorepo structure remains sound:
- `kamn-core`: 82 modules, 148K LOC — domain logic
- `kamn-node`: binary + runtime — deployment target
- `kamn-kolme`: wire format — protocol layer
- `kamn-sdk`: client libraries — external interface

No new crates or dependencies were added in R43.

### 7.2 kamn-core Module Count (82, stable since R38)

82 public modules, unchanged for 5 review cycles. The module surface has stabilized.

### 7.3 DID Type Taxonomy

Three DID types coexist:
- `AgentDid` (validates `kamn:did:agent:*`) — used in M1, M2, M5, M9
- `KamnDid` (validates broader `kamn:did:*`) — used in M2, M5, M6, M8, M9, M10
- Raw `String` — used in older modules not yet migrated

The gradual migration from `String` to typed DIDs remains incomplete in some non-data_layer modules.

### 7.4 Typed-DID Backlog Markers (R43 Follow-Up)

- `typed_did_migration_inventory_schema_version=kamn.typed-did-migration.inventory.v1`
- `typed_did_migration_inventory_non_data_layer_module_count=20`
- `typed_did_migration_inventory_non_data_layer_did_string_callsite_count=77`
- `typed_did_migration_backlog_issue_ids=#5223,#5228,#5229,#5230`
- `typed_did_migration_wave_issue_ids=#5228,#5229,#5230`
- `typed_did_migration_wave_a_scope=bridge_and_marketplace`
- `typed_did_migration_wave_b_scope=operator_and_governance`
- `typed_did_migration_wave_c_scope=runtime_proof_and_reputation`

Wave issue linkage:
- Wave A: `#5228` (bridge + marketplace surfaces)
- Wave B: `#5229` (operator + governance surfaces)
- Wave C: `#5230` (runtime/proof/reputation surfaces)

---

## 8. Priority Summary

| Priority | Issue | Location | Effort | Status |
|----------|-------|----------|--------|--------|
| ~~High~~ | ~~Signer test mutex poisoning~~ | ~~`main_tests/signer_tests.rs`~~ | ~~Small~~ | **RESOLVED R43** |
| ~~Medium~~ | ~~observability_endpoint.rs decomposition~~ | ~~`kamn-node/src/observability_endpoint.rs`~~ | ~~Medium~~ | **RESOLVED R43** |
| ~~Medium~~ | ~~PRD file relocation~~ | ~~`kamn-data-layer-prd.docx.md`~~ | ~~Trivial~~ | **RESOLVED R43** |
| ~~Low~~ | ~~Draft spec staleness review~~ | ~~specs/3881-3887~~ | ~~Small~~ | **RESOLVED R43** |
| ~~Low~~ | ~~Branch auto-cleanup~~ | ~~CI/GitHub config~~ | ~~Small~~ | **RESOLVED R43** |
| Medium | service_api_endpoint.rs decomposition | `kamn-node/src/service_api_endpoint.rs` (1,813 lines) | Medium | New |
| Medium | Doc-contract test consolidation | 127 test files across crates | Large | Open (migration started) |
| Medium | Shell test migration continuation | 509 remaining shell test scripts | Large | In progress (wave 1 done) |
| Low | Governance-to-feature ratio rebalancing | 95% governance / 5% features | Process | New |
| Low | Data layer integration stall | No data_layer changes in 3 cycles | Design decision | Open |
| Low | Data layer M11/PRD standalone status | `kamn-core/src/data_layer_m11_*` | Design decision | Open |
| Info | Public API surface monitoring | kamn-core 1,412 items (ratchet-gated) | Automated | Gated |
| Info | Spec-to-module ratio (7.2:1) | 591 specs / 82 modules | Ongoing | Growing |
