# KAMN Gaps and Issues Report

**As of:** R51 review, commit `0be43ed5` (2026-02-22)
**Baseline snapshot:** commit `0be43ed5` | **Rust LOC:** 193,077 | **Tests:** 3,616 passed, 0 failed, 12 ignored | **Shell LOC:** 141,965
**Follow-up markers:** `daemon_tests.rs` root 13 lines | remote branches 61 | spec dirs 823 | doc-contract tests 146

---

## R50 Gap Resolution Summary

| R50 Gap | Resolution | Status |
|---------|-----------|--------|
| Self-referential governance loop (HIGH) | Snapshot-semantics guardrail landed (#5549); loop did NOT stop — 75 new spec dirs from continued governance churn | **Open — WORSENED** |
| Spec volume (750, 8.2:1) | Grew to 823 (8.9:1); guardrail further breached | **Open — SEVERELY WORSENED** |
| Doc-contract tests (82) | Grew to 146 (+64); bulk from new kamn-e2e-harness crate | **Open — SEVERELY WORSENED** |
| Governance ratio (0.11:1) | Worsened further; see Section 5.2 | **Open — WORSENED** |

---

## 1. Test Infrastructure

### 1.1 Signer Env Guard — RESOLVED (R46)

Stable. Zero reproductions since R46 fix.

### 1.2 Ignored Tests (12 total, audited)

Stable since R39 (12+ consecutive reviews). Next periodic re-evaluation due R54 (#5465).

### 1.3 Doc-Contract Test Files (146, +64 from R50) — SEVERELY WORSENED

**Trajectory:** 122 (pre-wave) → 68 (R46 post-wave) → 70 (R48) → 82 (R49) → 82 (R50) → **146 (R51)**

The consolidation from R46's wave is now fully eroded and reversed. Breakdown by crate:

| Crate | Count | Change from R50 |
|-------|-------|-----------------|
| kamn-core | 96 | +14 |
| kamn-e2e-harness | 39 | **+39 (all new)** |
| kamn-node | 8 | — |
| kamn-sdk | 1 | — |
| kamn-agent-lib | 1 | **+1 (new)** |
| **Total** | **146** | **+64** |

The kamn-e2e-harness crate alone accounts for 39 new doc-contract tests — one per documentation artifact created during the PRD implementation phases. Each is a thin test file that loads a markdown document and asserts marker presence. This is the same proliferation pattern flagged in R42 (then at 125 files).

### 1.4 Public API Surface Ratchet Gate

No changes to kamn-core modules. Gate functioning as designed.

---

## 2. Data Layer and Validation

### 2.1–2.7 Status Carry-Forward

All items carry forward unchanged from R50. No new data layer work landed in R51.

---

## 3. Structural Concerns

### 3.1 daemon_tests.rs decomposition — RESOLVED (R48+R50)

Stable. 13-line root, 6,043-line tree across 9 files. No regressions.

### 3.2 Production Source Files >1K (13, +1) — NEW ADDITION

| File | Lines | Notes |
|------|-------|-------|
| `channel_models.rs` | 1,831 | Unchanged |
| `message_lifecycle.rs` | 1,784 | Unchanged |
| `task_operations.rs` | 1,735 | Unchanged |
| `p2p_transport/p2p_transport_live.rs` | 1,711 | Unchanged |
| `did_registry.rs` | 1,679 | Unchanged |
| `zk_message_proofs.rs` | 1,482 | Unchanged |
| **`kamn-e2e-harness/src/lib.rs`** | **1,328** | **NEW — see Section 3.3** |
| `block_pipeline/block_pipeline_support.rs` | 1,214 | Unchanged |
| `data_layer_m5_vector_integration.rs` | 1,097 | Unchanged |
| `runtime_peer_coordination.rs` | 1,095 | Unchanged |
| `data_layer_m4_escrow_integration.rs` | 1,081 | Unchanged |
| `durable_guard_store.rs` | 1,038 | Unchanged |
| `signer_backend.rs` | 1,022 | Unchanged |

### 3.3 kamn-e2e-harness lib.rs Monolith (1,328 lines) — NEW, MEDIUM

The new `kamn-e2e-harness/src/lib.rs` has grown to 1,328 lines in a single cycle. Content analysis reveals it is primarily **marker definitions and contract validation functions** rather than executable harness logic:

- 39 functions, 40 marker/contract references, 3 assertions
- Zero real infrastructure operations (`Command`, `spawn`, `docker`, `tokio`, `async`)
- The `infrastructure.rs` module (36 lines) contains only an enum and a phase-ordering struct — no actual process management
- All 15 scenario files (`s01` through `s15`) are 10-line stubs returning a `ScenarioDefinition` with `{id, name, priority}`
- All 3 driver files (`sdk_direct.rs`, `cli_scripted.rs`, `mcp_agent.rs`) are scaffolds that return hardcoded `status: "pass"`

**The crate cannot execute any E2E test.** It is a contract-assertion shell around the PRD's design vocabulary. The 1,328-line lib.rs should be decomposed once real functionality lands.

### 3.4 New Crate Assessment — 4 New Workspace Members

R51 added 4 new crates from the E2E PRD implementation:

| Crate | Total LOC | Real Capability | Assessment |
|-------|-----------|----------------|------------|
| `kamn-agent-lib` | 897 | **Genuine** — wraps `kamn-sdk` with auth, envelope, identity, nonce, Kolme proof verification. 5 of 12 operations stub out with `UnsupportedOperation` | **Good start, ~60% stubbed** |
| `kamn-mcp-server` | 256 | **Scaffold** — config parsing + tool name list. No MCP protocol implementation | **Stub** |
| `kamn-cli` | 426 | **Scaffold** — clap subcommand defs. Each command file is 8 lines returning a placeholder | **Stub** |
| `kamn-e2e-harness` | 6,114 | **Contract markers** — see Section 3.3. Cannot run any scenario | **Marker-heavy** |

`kamn-agent-lib` is the only crate with real executable code. It properly delegates to `kamn-sdk`'s `ServiceApiClient` and adds auth header construction, identity management, and Kolme proof verification. However, `accept_task`, `complete_task`, `fund_escrow`, `release_escrow`, and `list_messages` all return `UnsupportedOperation` errors.

### 3.5 Governance-Driven Architecture Inflation — HIGH PRIORITY (CONTINUED FROM R50)

The R50 review identified the self-referential governance loop as HIGH priority. R51 shows the pattern has **intensified rather than abated**:

**149 commits yielded:**
- **75 new spec directories** (5499–5655) — zero from new capabilities, all from governance lifecycle artifacts
- **110 unique issue/PR references** — nearly all governance issues
- **+9,816 Rust LOC** — largely contract markers, scaffold code, and doc-contract test assertions
- **+237 tests** — primarily marker-assertion tests, not behavioral tests
- **0 new kamn-core production modules** — zero production capability advancement

The governance loop pattern from R50 (branch-count marker reconciliation, 7 issues / 20 commits) has metastasized into a broader pattern where:
1. A PRD is written (genuine design work)
2. PRD phases generate spec artifacts per issue
3. Spec artifacts require lifecycle markers
4. Lifecycle markers require non-regression ratchet guardrails
5. Guardrail assertions require doc-contract tests
6. Each step generates more issues with more spec directories

**R51 is the first cycle where the governance machinery has consumed the entire cycle.** No production code was advanced. The E2E harness crates exist but cannot execute.

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0 (all 489 occurrences in `#[cfg(test)]` blocks)
- **Production `unwrap()` count:** 0 (workspace-wide `clippy::unwrap_used = "deny"`)
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings)

Zero production panicking paths maintained since R36 (15 consecutive reviews).

The new crates (`kamn-agent-lib`, `kamn-mcp-server`, `kamn-cli`, `kamn-e2e-harness`) also maintain zero expect/unwrap in production code.

---

## 5. Spec and Governance

### 5.1 Spec Volume (SEVERELY BREACHED)

- **823 spec directories** (+73 from R50)
- Spec-to-module ratio: **8.9:1** (823 / 92 modules)
- **Guardrail target: 7.7:1 max** — now exceeded by **1.2**

Growth trajectory: 7.7 (R48) → 8.0 (R49) → 8.2 (R50) → **8.9 (R51)**

All 73 new spec directories are from governance lifecycle artifacts. Zero from new production capabilities.

### 5.2 Governance-Feature Activity Ratio

**R51: 0 feat / 149 total commits (governance-only cycle)**

Classifying the 149 commits:
- Merge commits: 19
- Spec artifacts: 31
- Governance/guardrail/ratchet/reconciliation: 45
- Service API metrics exposure: 39
- E2E harness contract markers (labeled `feat`): 46
- PRD phase scaffold: 10
- **Genuine new production capability: 0**

The `feat` prefix is used extensively but misleadingly. Commits labeled `feat(kamn-e2e-harness): enforce verification hash-format contract` deliver a test assertion, not a feature. The `feat: phase-5c spawn timeline contracts` delivers a contract marker struct, not process spawning.

### 5.3 Governance-Feature Activity Ratio Markers (R51)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=149
- feature_activity_commit_count=0
- activity_total_commit_count=149
- governance_activity_commit_ratio=1.0000
- feature_activity_commit_ratio=0.0000

### 5.4 Spec-Volume Guardrail Markers (R51)

- spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1
- spec_volume_guardrail_baseline_spec_directory_count=823
- spec_volume_guardrail_baseline_module_count=92
- spec_volume_guardrail_baseline_spec_to_module_ratio=8.9
- spec_volume_guardrail_target_spec_to_module_ratio_max=7.7
- spec_volume_guardrail_target_status=severely_breached

---

## 6. Branch Hygiene — WORSENED

### 6.1 Branch Count (61, +10)

Trajectory: 174 (R47) → 55 (R48) → 51 (R49) → 51 (R50) → **61 (R51)**.

After 2 cycles of stability at 51, branches grew +10 from the high volume of governance PRs. Still within the 174-peak boundary but trending upward.

---

## 7. Architecture

### 7.1 Crate Structure (8 crates, +4 from R50)

| Crate | Modules | LOC | Status |
|-------|---------|-----|--------|
| `kamn-core` | 92 | ~183K | Unchanged — production backbone |
| `kamn-node` | — | — | Binary + runtime |
| `kamn-kolme` | 23 | — | Wire format |
| `kamn-sdk` | 7 | — | Client libraries |
| **`kamn-agent-lib`** | **7** | **897** | **NEW — partially functional** |
| **`kamn-mcp-server`** | **2** | **256** | **NEW — scaffold** |
| **`kamn-cli`** | **—** | **426** | **NEW — scaffold** |
| **`kamn-e2e-harness`** | **7** | **6,114** | **NEW — contract markers** |

The PRD v3 design is reflected in the crate structure but implementation is shallow. `kamn-agent-lib` is the most mature, with real integration against `kamn-sdk`. The other three are scaffolds awaiting real functionality.

### 7.2 kamn-core Module Count (92, stable)

No new modules. R51 was entirely governance/scaffolding work.

---

## 8. Priority Summary

| Priority | Issue | Location | Effort | Status |
|----------|-------|----------|--------|--------|
| **Critical** | Governance loop consuming all development capacity | Process | Process change | **NEW — 100% governance for 2nd consecutive cycle** |
| **High** | Spec volume (823, 8.9:1) | 823 dirs / 92 modules | Ongoing | **Severely exceeded guardrail by 1.2** |
| **High** | Doc-contract test proliferation (146) | +64 in one cycle | Consolidation | **Consolidation fully reversed** |
| **High** | E2E harness is scaffold-only | `kamn-e2e-harness` 6,114 LOC, 0 executable | Implementation | **NEW** |
| Medium | `feat` commit mislabeling | 46 commits labeled feat that deliver markers | Convention | **NEW** |
| Medium | kamn-agent-lib 5/12 ops stubbed | accept_task, complete_task, etc. | SDK extension | **NEW** |
| Medium | Branches (61, +10) | Trending up | Cleanup | **Worsened** |
| Low | kamn-e2e-harness lib.rs (1,328 lines) | New monolith candidate | Decompose when real code lands | **NEW** |
| Info | Ignored tests (12) | Due R54 | Periodic | Stable |
| Info | Prod files >1K (13) | +1 (harness lib.rs) | Monitor | Stable-ish |

---

## 9. Critical Assessment: Two Consecutive Governance-Only Cycles

R50 and R51 together represent **191 commits (42 + 149) with zero production capability advancement**. The codebase has the same 92 kamn-core modules, the same 3,379 base tests (237 new tests are all marker assertions), and the same runtime behavior as R49.

What was delivered:
- PRD v1/v2/v3 design documents (genuine value)
- `kamn-agent-lib` with partial SDK client wrapping (partial value)
- 3 scaffold crates with placeholder implementations (near-zero runtime value)
- 73 spec directories, 64 doc-contract test files, dozens of governance guardrail markers (governance overhead)

**Recommendations:**

1. **Immediate moratorium on spec directory creation** until the ratio drops below 8.0:1 (requires ~8 spec deletions per new module, or no new specs until ~12 new modules land)
2. **Stop labeling marker/contract work as `feat`** — use `chore` or `governance` prefix for accuracy
3. **Prioritize kamn-agent-lib completion** — fill the 5 stubbed operations, then kamn-mcp-server becomes viable
4. **Defer doc-contract tests** for scaffold crates until those crates have real functionality
5. **Budget the next cycle 80/20 feature/governance** — at least 80% of commits should advance production capability

---

## 10. Trend Table

| Review | Commit | Rust LOC | Tests | Failures | Prod expect() | Shell LOC | Ratio | Modules | Grade |
|--------|--------|----------|-------|----------|----------------|-----------|-------|---------|-------|
| R43 | 4af28701 | 148,022 | 2,888 | 0 | 0 | 122K† | 0.82:1† | 82 | A+ |
| R44 | 92c9958c | 151,091 | 2,911 | 0 | 0 | 122K† | 0.80:1† | 84 | A+ |
| R45 | ed913983 | 166,297 | 3,080 | 1 | 0 | 122K† | 0.73:1† | 90 | A |
| R46 | f95a039e | 166,664 | 3,104 | 0 | 0 | 121K† | 0.73:1† | 90 | A+ |
| R47 | 5570c2b8 | 171,635 | 3,188 | 0 | 0 | 121K† | 0.70:1† | 90 | A- |
| R48 | f044f9bd | 176,259 | 3,258 | 0 | 0 | ~141K* | 0.80:1* | 91 | A |
| R49 | cc0f9f00 | 183,111 | 3,379 | 0 | 0 | 142K | 0.78:1 | 92 | A- |
| R50 | 49efe252 | 183,261 | 3,379 | 0 | 0 | 142K | 0.78:1 | 92 | B+ |
| **R51** | **0be43ed5** | **193,077** | **3,616** | **0** | **0** | **142K** | **0.74:1** | **92** | **C** |

† Shell LOC understated in original reviews
* Corrected from originally reported 121K

**Grade rationale (C):** First C-grade in the review series. Zero production capability advancement across 149 commits. Spec-to-module ratio at 8.9:1 (severely breaching 7.7 guardrail). Doc-contract tests reversed from consolidation. Three scaffold crates delivered with no executable functionality. The codebase is technically healthy (zero prod panics, clean clippy, 0 failures) but development velocity on actual capabilities has stalled for two consecutive cycles.
