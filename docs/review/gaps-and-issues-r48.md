# KAMN Gaps and Issues Report

**As of:** R48 review follow-up, issue `#5449` (2026-02-21)
**Baseline snapshot:** commit `f044f9bd` (2026-02-20) | **Rust LOC:** 176,259 | **Tests:** 3,258 passed, 0 failed, 12 ignored | **Shell LOC:** 121,158
**Follow-up markers:** `daemon_tests.rs` root 13 lines (`wc -l crates/kamn-node/src/main_tests/daemon_tests.rs`) | remote branches 55 (`git ls-remote --heads origin | wc -l`)

---

## R47 Gap Resolution Summary

Four gaps were identified in R47. Three are now resolved and one remains open:

| R47 Gap | Resolution | Status |
|---------|-----------|--------|
| daemon_tests.rs (4,897 lines) | Phase-1/2/3 decomposition completed into bounded submodules (#5402, #5418, #5420) | **RESOLVED (13-line root; 5 extracted suites)** |
| Branch explosion (174) | Cleanup waves executed | **RESOLVED (55)** |
| 100% governance ratio | 5 feat commits landed (#4013, #4014, #4004, #4006, #4018) | **IMPROVED (0.19:1)** |
| Spec volume (682, 7.6:1) | Grew to 700 (7.7:1) | **Open** |

---

## 1. Test Infrastructure

### 1.1 Signer Env Guard — RESOLVED (R46)

Stable across R47 and R48. Zero reproductions of the flake.

### 1.2 Ignored Tests (12 total, audited)

Stable since R39 (9 consecutive reviews). Formally audited with explicit dispositions (#5329).

R49 periodic re-evaluation completed via #5465 with no inventory drift (`12` baseline-aligned entries, `reason_codes=none`).

Deterministic carry-forward markers:
- `ignored_test_periodic_review_cycle=R49`
- `ignored_test_periodic_review_status=completed`
- `ignored_test_periodic_review_issue=5465`
- `ignored_test_periodic_review_next_due_cycle=R54`

### 1.3 Doc-Contract Test Files (70, +2 from R47)

Two new doc-contract test files were added in R48:
- `observability_stack_docs.rs` — new observability docs coverage
- `service_api_ops_configuration_docs.rs` — new ops configuration docs coverage

These are genuine new coverage, not a consolidation reversal. The wave 3+4 harness consolidation remains effective.

### 1.4 Public API Surface Ratchet Gate

Baseline refreshed for `cross_store_replay_consistency` module (#4013). Gate functioning as designed.

---

## 2. Data Layer and Validation

### 2.1 Infrastructure Activation (Phases 1–6) — RESOLVED R45

Complete. See `data-layer-roadmap.md`.

### 2.2 Live PostgreSQL Validation Framework — In Progress

The env-gated live-postgres daemon validation framework built in R47 (#5338–#5400) now has:
- Phase-1 fixture decomposition completed (#5402)
- Cross-store replay consistency production module landed (#4013)
- 35+ topology coherence contract dimensions validated

The framework is architecturally complete for same-host bounded-profile parallel lanes. Multi-host distributed lanes remain follow-up work.

### 2.3 SQLite Durability Framework — NEW (R48)

A new testing dimension targeting SQLite persistence under failure:

| Issue | Scope |
|-------|-------|
| #4014 | Crash-recovery dry-run governance checker |
| #4015 | Journal/WAL schema fixtures and commit-boundary markers |
| #4016 | Partial-write fault injection regressions |
| #4017 | Crash-restart lane runner |
| #4018 | Crash-restart policy checker with runbook parity |

This validates atomic commit boundaries and WAL recovery — essential for the SQLiteStoreBackend that underpins all snapshot persistence.

### 2.4 Capacity Load Testing — NEW (R48)

Infrastructure for performance/capacity validation:
- Local-heavy capacity load lane runner (#4004)
- Load lane policy checker (#4005)
- Capacity dry-run go-no-go governance (#4006)
- Parent closeout (#3998)

---

## 3. Structural Concerns

### 3.1 daemon_tests.rs decomposition — RESOLVED (R48 follow-up)

The monolithic root file concern is closed. Decomposition phases completed under #5402, #5418, and #5420 reduced the root to 13 lines and extracted bounded suites:

```
crates/kamn-node/src/main_tests/daemon_tests.rs                                      13
crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs            1,615
crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests.rs 2,400
crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs 1,030
crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs               726
crates/kamn-node/src/main_tests/daemon_tests/live_postgres_distributed_execution_contract_tests.rs 173
```

### 3.2 Production Source Files >1K (12, stable)

Unchanged from R46. All 12 are established domain modules with 8+ review stability. No urgent action needed.

| File | Lines |
|------|-------|
| `channel_models.rs` | 1,831 |
| `message_lifecycle.rs` | 1,784 |
| `task_operations.rs` | 1,735 |
| `p2p_transport/p2p_transport_live.rs` | 1,711 |
| `did_registry.rs` | 1,679 |
| `zk_message_proofs.rs` | 1,482 |
| `block_pipeline/block_pipeline_support.rs` | 1,214 |
| `data_layer_m5_vector_integration.rs` | 1,097 |
| `runtime_peer_coordination.rs` | 1,095 |
| `data_layer_m4_escrow_integration.rs` | 1,081 |
| `durable_guard_store.rs` | 1,038 |
| `signer_backend.rs` | 1,022 |

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0
- **Production `unwrap()` count:** 0
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings)

Zero production panicking paths maintained since R36 (12 consecutive reviews).

---

## 5. Spec and Governance

### 5.1 Spec Volume (Growing)

- **700 spec directories** (+18 from R47)
- Spec-to-module ratio: **7.7:1** (700 / 91 modules)

Crossed the 700 threshold. Topology hardening (#5394–#5400), SQLite durability (#4014–#4018), and capacity testing (#3998, #4004–#4006) each contributed spec artifacts.

### 5.2 Governance-Feature Activity Ratio

R48: 5 feat / 26 governance (0.19:1). Improved from R47's 0.00:1 but still governance-heavy. The feat commits delivered substantive capabilities (cross-store replay, crash-recovery, capacity load).

### 5.3 Governance-Feature Activity Ratio Markers (R48)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=26
- feature_activity_commit_count=5
- activity_total_commit_count=31
- governance_activity_commit_ratio=0.8387
- feature_activity_commit_ratio=0.1613

### 5.4 Coherence Contract Batching Carry-Forward Policy

Carry forward the R45 coherence batching policy contract so future topology/coherence expansions remain bundled instead of one issue per dimension.

Deterministic carry-forward markers:

- `coherence_contract_batching_policy_schema_version=kamn.review.coherence-contract-batching-policy.v1`
- `coherence_contract_batching_dimension_baseline=28`
- `coherence_contract_batching_issue_baseline=28`
- `coherence_contract_batching_target_bundle_count_min=5`
- `coherence_contract_batching_target_bundle_count_max=8`
- `coherence_contract_batching_target_issue_cap=8`
- `coherence_contract_batching_expected_issue_reduction=20`

### 5.5 Spec-Volume Guardrail Policy

R48 remains open on spec-volume growth; guardrail markers provide deterministic trend tracking and target posture for follow-up cycles.

Deterministic spec-volume markers:

- `spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1`
- `spec_volume_guardrail_baseline_spec_directory_count=700`
- `spec_volume_guardrail_baseline_module_count=91`
- `spec_volume_guardrail_baseline_spec_to_module_ratio=7.7`
- `spec_volume_guardrail_target_spec_to_module_ratio_max=7.7`
- `spec_volume_guardrail_target_status=monitor`
- `spec_volume_guardrail_evidence_command_spec_dirs=find specs -maxdepth 1 -mindepth 1 -type d | wc -l`
- `spec_volume_guardrail_evidence_command_module_exports=rg "^pub mod " crates/kamn-core/src/lib.rs | wc -l`

---

## 6. Branch Hygiene — RESOLVED

### 6.1 Branch Count (55)

Branch hygiene remains resolved and improved from the R48 point-in-time snapshot.

Trajectory: 126 (R42) → 78 (R43) → 54 (R46 follow-up) → 174 (R47) → 68 (R48 snapshot) → **55 (R48 follow-up)**.

---

## 7. Architecture

### 7.1 Crate Structure

- `kamn-core`: **91 modules**, 176K LOC — domain logic + data layer infrastructure
- `kamn-node`: binary + runtime
- `kamn-kolme`: wire format
- `kamn-sdk`: client libraries

### 7.2 kamn-core Module Count (91, +1)

New module: `cross_store_replay_consistency` (412 lines, #4013). The first new production module since R45's data layer sprint. Implements deterministic cross-store replay divergence taxonomy.

### 7.3 New Production Module: cross_store_replay_consistency

Validates that replayed operations across different store backends (file, sqlite, postgres) produce deterministic, identical results. Exports:
- `evaluate_cross_store_replay_consistency()` — main evaluation function
- `CrossStoreReplayConsistencyReport` — structured outcome
- `CrossStoreReplayDivergenceClass` — taxonomy of divergence types
- Reason code CSV and taxonomy version constants

This fills a critical gap in the multi-backend architecture's correctness guarantees.

### 7.4 Data Layer Architecture Maturity

Evolution updated:

1. **R38–R42:** Pure contract layer (15 modules, ~9.6K lines, zero I/O)
2. **R45:** Infrastructure activation (6 new modules, Phases 1–6)
3. **R46:** Structural consolidation (monolith decomposition)
4. **R47:** Live-postgres validation framework (env-gated topology contracts)
5. **R48:** Cross-store replay consistency + SQLite durability + capacity load testing

The architecture now has comprehensive validation infrastructure across all three storage backends.

---

## 8. Priority Summary

| Priority | Issue | Location | Effort | Status |
|----------|-------|----------|--------|--------|
| ~~Medium~~ | ~~Branch explosion (174)~~ | ~~Remote branches~~ | ~~Small~~ | **RESOLVED (55)** |
| ~~Low~~ | ~~Governance-only cycle~~ | ~~0 feat / 75 gov~~ | ~~Process~~ | **IMPROVED (5 feat / 26 gov)** |
| ~~Medium~~ | ~~daemon_tests.rs monolith decomposition~~ | ~~`main_tests/daemon_tests.rs`~~ | ~~Medium~~ | **RESOLVED (#5402, #5418, #5420)** |
| Low | Spec volume (700, 7.7:1) | 700 dirs / 91 modules | Ongoing | Growing |
| Low | Doc-contract tests (70) | +2 new coverage files | Monitor | Stable |
| Info | Ignored tests (12, audited) | Due for re-evaluation R49 | Periodic | Stable 9 cycles |
| Info | Prod files >1K (12) | kamn-core/src/ | None urgent | Stable 5+ cycles |
| Info | Shell LOC (121,158) | +200 (noise) | Monitor | Stable |

---

## 9. Trend Table

| Review | Commit | Rust LOC | Tests | Failures | Prod expect() | Shell LOC | Ratio | Modules | Grade |
|--------|--------|----------|-------|----------|----------------|-----------|-------|---------|-------|
| R41 | 82850743 | 142,536 | 2,808 | 0 | 0 | 123K | 0.86:1 | 82 | A+ |
| R42 | 229fe3ee | 144,242 | 2,838 | 0 | 0 | 123K | 0.85:1 | 82 | A |
| R43 | 4af28701 | 148,022 | 2,888 | 0 | 0 | 122K | 0.82:1 | 82 | A+ |
| R44 | 92c9958c | 151,091 | 2,911 | 0 | 0 | 122K | 0.80:1 | 84 | A+ |
| R45 | ed913983 | 166,297 | 3,080 | 1 | 0 | 122K | 0.73:1 | 90 | A |
| R46 | f95a039e | 166,664 | 3,104 | 0 | 0 | 121K | 0.73:1 | 90 | A+ |
| R47 | 5570c2b8 | 171,635 | 3,188 | 0 | 0 | 121K | 0.70:1 | 90 | A- |
| **R48** | **f044f9bd** | **176,259** | **3,258** | **0** | **0** | **121K** | **0.69:1** | **91** | **A** |
