# KAMN Gaps and Issues Report

**As of:** R42 review, commit `229fe3ee` (2026-02-19)
**Rust LOC:** 144,242 | **Tests:** 2,838 passed, 13 ignored | **Shell LOC:** 123,089

---

## 1. Test Infrastructure

### 1.1 Signer Test Mutex Poisoning (Moderate)

**Location:** `crates/kamn-node/src/main_tests/signer_tests.rs`

20 signer tests fail under parallel execution due to mutex poisoning cascade. A shared `signer_env_lock()` (`Mutex<()>` in `main_tests.rs:44-47`) serializes env-var-mutating tests. When one test panics while holding the lock, all subsequent tests that call `.expect()` on the poisoned lock fail with `PoisonError`.

- **Root test:** `regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch` (line 1428) — assertion expects `managed_signer_backend_response_provenance_mismatch` but the provenance adapter routing from #3955 changed behavior
- **Cascade:** 19 additional tests fail from the poisoned lock
- **CI impact:** None currently (CI happens to run in a thread order that avoids the cascade), but fragile
- **Fix:** (a) Update the root test's env setup for the new provenance routing, (b) change all `.expect("signer env lock...")` to `.unwrap_or_else(|e| e.into_inner())` to prevent cascading

### 1.2 Ignored Tests (13 total, stable)

Across 3 files:
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-core/tests/input_mutation_coverage_guided.rs`
- `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`

These have been stable at 13 since R33 (9 reviews). No regression, but should be periodically audited to determine if they can be re-enabled or should be removed.

### 1.3 Doc-Contract Test Proliferation (125 files)

125 test files use `include_str!()` to load markdown docs and assert marker presence. This pattern is deeply entrenched across:
- 109 in `crates/kamn-core/tests/`
- 14 in `crates/kamn-node/tests/`
- 2 in `crates/kamn-sdk/tests/`

While individually valid, the aggregate cost is:
- Brittle coupling between docs and test suites
- Any doc restructuring requires updating multiple test files
- Test output noise when markers drift

**Recommendation:** Consider a unified doc-contract assertion framework (single test binary with data-driven marker tables) rather than one file per doc.

---

## 2. Data Layer Integration

### 2.1 Integration Status (11/15 integrated with core types)

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

**M11 hardening readiness** — Defines `DataLayerM11OperatorReadinessReport` and decision types independently. Could bridge to existing operator/deployment readiness concepts if any exist in the crate. Low priority since this is a meta-assessment module.

**PRD conformance** — Self-contained conformance checker. Standalone by design (it evaluates all other modules), so this is acceptable.

**No data_layer changes landed in R42.** Integration has been stalled at this level since R41.

---

## 3. Structural Concerns

### 3.1 Large Files (26 files over 1,000 lines)

**Production source files over 1,000 lines:**

| File | Lines | Concern |
|------|-------|---------|
| `kamn-core/src/channel_models.rs` | 1,829 | Largest kamn-core source file |
| `kamn-node/src/service_api_endpoint.rs` | 1,813 | Largest kamn-node source file |
| `kamn-core/src/message_lifecycle.rs` | 1,782 | |
| `kamn-core/src/task_operations.rs` | 1,733 | |
| `kamn-core/src/p2p_transport/p2p_transport_live.rs` | 1,711 | |
| `kamn-core/src/did_registry.rs` | 1,679 | |
| `kamn-core/src/zk_message_proofs.rs` | 1,482 | |
| `kamn-node/src/observability_endpoint.rs` | 1,454 | Open concern since R31 (11 reviews) |
| `kamn-core/src/runtime_tests.rs` | 1,418 | Test file |
| `kamn-core/src/block_pipeline/block_pipeline_support.rs` | 1,214 | |
| `kamn-core/src/data_layer_m5_vector_integration.rs` | 1,097 | |
| `kamn-core/src/data_layer_m4_escrow_integration.rs` | 1,081 | |
| `kamn-core/src/durable_guard_store.rs` | 1,038 | |
| `kamn-node/src/signer.rs` | 1,033 | Grew from 908 in R41 (+125 lines) |
| `kamn-core/src/signer_backend.rs` | 1,022 | |
| `kamn-core/src/runtime_peer_coordination.rs` | 1,010 | |

**Test files over 1,000 lines:** 10 additional files (service_api_endpoint_tests at 2,159 is the largest).

`signer.rs` crossed the 1,000-line threshold this cycle (was 908 in R41). The signer subsystem now spans 5 files totaling 2,513 lines (`signer.rs` + `managed_backend.rs` + `signer_policy.rs` + `signer_adapter.rs` + `nonce.rs`). The decomposition is good but the parent module is growing.

### 3.2 observability_endpoint.rs (1,454 lines, open 11 reviews)

This file has been flagged since R31 and hasn't changed. It combines HTTP endpoint handlers, streaming logic, and route configuration in a single file. Unlike `signer.rs` (which was decomposed into submodules), this file remains monolithic.

### 3.3 Public API Surface (1,412 items)

kamn-core exports 1,412 public items (`pub fn`, `pub struct`, `pub enum`, `pub trait`). This is a large surface area for a core library crate. Consider whether all 82 public modules need full public visibility or if some could use `pub(crate)`.

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0
- **Production `unwrap()` count:** 0
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings with `-D warnings`)

This is excellent. The codebase has maintained zero production panicking paths since R36.

### 4.2 Symlink Health (Clean)

401 symlinks in `scripts/`, zero broken. The exec_registry.json (221 entries) has zero dead references.

---

## 5. Spec and Governance

### 5.1 Spec Volume

- **562 spec directories** (each with spec.md, plan.md, tasks.md)
- **Status breakdown:** 41 Reviewed, 41 Implemented, 5 Draft
- **0 specs** missing Test Mapping sections
- **29 milestones** defined

The spec-to-module ratio is ~6.9:1 (562 specs / 82 modules). While specs map to issues rather than modules, this ratio indicates heavy governance overhead relative to production code.

### 5.2 Draft Specs (5 pending)

The following specs are still in Draft status:
- `specs/3881/spec.md`
- `specs/3884/spec.md`
- `specs/3885/spec.md`
- `specs/3886/spec.md`
- `specs/3887/spec.md`

These should be reviewed for staleness.

### 5.3 PRD File at Repository Root

`kamn-data-layer-prd.docx.md` (1,205 lines) sits at the repo root. This is a converted Word document that should be moved to `docs/` or `specs/` for organizational consistency.

### 5.4 Shell Test Scripts vs Rust Tests

- **529 shell test scripts** (`test_*.sh`, non-symlink)
- **292 Rust test files** (`crates/*/tests/*.rs`)

The shell-to-Rust test file ratio is 1.81:1. Many shell tests are thin wrappers that could potentially be consolidated or migrated to Rust integration tests.

---

## 6. Branch Hygiene

Remote branches grew from 90 (after R41 pruning) to 126 in one cycle (+36). The R41 pruning wave (444 to 90) is being eroded. Consider:
- Automating stale branch cleanup (e.g., delete branches for merged PRs older than 7 days)
- Setting up branch protection rules to prevent accumulation

---

## 7. Architecture

### 7.1 Crate Structure

The 4-crate monorepo structure is sound:
- `kamn-core`: 82 modules, 144K LOC — domain logic
- `kamn-node`: binary + runtime — deployment target
- `kamn-kolme`: wire format — protocol layer
- `kamn-sdk`: client libraries — external interface

No new crates or dependencies were added in R42. Cargo.toml files are unchanged.

### 7.2 kamn-core Module Count (82, stable)

82 public modules has been stable since R38 (when the 15 data_layer modules landed). No new modules have been added in 4 review cycles. This suggests the module surface has stabilized.

### 7.3 DID Type Taxonomy

Three DID types coexist:
- `AgentDid` (validates `kamn:did:agent:*`) — used in M1, M2, M5, M9
- `KamnDid` (validates broader `kamn:did:*`) — used in M2, M5, M6, M8, M9, M10
- Raw `String` — used in older modules not yet migrated

The type taxonomy is clear but the gradual migration from `String` to typed DIDs is incomplete in some non-data_layer modules.

---

## 8. Priority Summary

| Priority | Issue | Location | Effort |
|----------|-------|----------|--------|
| High | Signer test mutex poisoning | `main_tests/signer_tests.rs` | Small (1-2 hours) |
| Medium | observability_endpoint.rs decomposition | `kamn-node/src/observability_endpoint.rs` | Medium |
| Medium | Doc-contract test consolidation | 125 test files across crates | Large |
| Medium | PRD file relocation | `kamn-data-layer-prd.docx.md` | Trivial |
| Low | Draft spec staleness review | specs/3881-3887 | Small |
| Low | Branch auto-cleanup | CI/GitHub config | Small |
| Low | signer.rs growth monitoring | `kamn-node/src/signer.rs` | Ongoing |
| Low | Data layer M11/PRD standalone status | `kamn-core/src/data_layer_m11_*` | Design decision |
| Info | Public API surface audit | kamn-core 1,412 items | Large |
| Info | Shell test migration opportunities | 529 shell test scripts | Large, ongoing |
