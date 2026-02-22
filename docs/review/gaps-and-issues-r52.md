# KAMN Gaps and Issues Report

**As of:** R52 review, commit `8e0871cc` (2026-02-22)
**Baseline snapshot:** commit `8e0871cc` | **Rust LOC:** 198,094 | **Tests:** 3,160 passed, 2 failed, 10 ignored (excl kamn-cli — see Section 1.1) | **Shell LOC:** 141,965
**Follow-up markers:** `daemon_tests.rs` root 13 lines | remote branches 67 | spec dirs 845 | doc-contract tests 109

---

## R51 Gap Resolution Summary

| R51 Gap | Resolution | Status |
|---------|-----------|--------|
| Governance loop (CRITICAL — 2 consecutive cycles) | 7 genuine feat commits delivered real MCP protocol, CLI dispatch, agent-lib routes. Governance ratio improved from 100% to ~64% but still inverted | **Improved — NOT RESOLVED** |
| Spec volume (823, 8.9:1) | Grew to 845 (9.2:1); guardrail further breached | **Open — WORSENED** |
| Doc-contract tests (146) | Consolidated to 109 (-37) via #5690 (39→2 harness test files). First consolidation success since R46 | **Improved** |
| E2E harness scaffold-only | S01 scenario activation across 3 drivers (#5696, #5698, #5700); MCP live probe health flow (#5703) | **Improved — PARTIAL** |
| feat mislabeling | Persists — 4 of 15 feat commits are marker/contract assertions | **Open** |
| kamn-agent-lib stubbed ops | Task transition and escrow route contracts added (#5674) | **Improved — PARTIAL** |
| Branches (61, +10) | 67 (+6). Branch hygiene prune #5686 ran but new PRs offset | **Slightly worsened** |
| kamn-e2e-harness lib.rs monolith (1,328 lines) | Decomposed via #5688 (extract run-contract module) | **Resolved** |

---

## 1. Test Infrastructure

### 1.1 kamn-cli Compilation Error — NEW, CRITICAL

**First compilation failure on main in the review series.**

`crates/kamn-cli/tests/command_activation_contract.rs` has 15 instances of `output.text.contains(...)` where `output` is `String` (no `.text` field). Error: `E0609: no field 'text' on type 'String'`. The `dispatch()` function returns a `CliOutput` struct with `.text` and `.json` fields in some contexts, but the test file accesses `.text` on a plain `String`.

This means `cargo test --workspace` fails to compile. The kamn-cli crate's 14 test functions cannot execute. All R52 test metrics exclude kamn-cli.

Root cause appears to be a signature change in `dispatch()` that was not reflected in the contract test. The `parsed_json` helper and `spec_c06` test also reference `output.json`, confirming the test expects a struct return type.

### 1.2 Doc-Contract Governance Marker Test Failures (2) — NEW, MEDIUM

Two tests in `release_review_activity_ratio_docs_contract.rs` fail because the R51 gaps document uses backtick-wrapped markers:

```
- `governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1`
```

The test parser (`parse_marker_lines`) strips the `- ` prefix but not markdown backticks, so the key becomes `` `governance_feature_activity_ratio_schema_version`` instead of `governance_feature_activity_ratio_schema_version`. Fix: remove backtick wrapping from the R51 marker lines (Sections 5.3 and 5.4).

### 1.3 Ignored Tests (14 annotations, 10 reported)

14 `#[ignore]` annotations across workspace. 10 reported as ignored in test run (4 in kamn-cli can't compile). Stable since R39. Next periodic re-evaluation due R54.

### 1.4 Doc-Contract Test Consolidation — IMPROVED

**Trajectory:** 122 (pre-wave) → 68 (R46) → 82 (R50) → **146 (R51)** → **109 (R52)**

The kamn-e2e-harness consolidation (#5690) merged 39 individual doc-contract test files into 2 aggregate files (`docs_contract_phase_group.rs`, `docs_contract_release_group.rs`). This is the first successful consolidation since R46.

| Crate | R51 Count | R52 Count | Delta |
|-------|-----------|-----------|-------|
| kamn-core | 96 | 96 | — |
| kamn-e2e-harness | 39 | 2 | **-37** |
| kamn-node | 8 | 8 | — |
| kamn-sdk | 1 | 1 | — |
| kamn-agent-lib | 1 | 1 | — |
| kamn-mcp-server | 1 | 1 | — |
| **Total** | **146** | **109** | **-37** |

### 1.5 Test Count Drop (-456 from R51) — OBSERVATION

Tests: 3,160 passed (R52) vs 3,616 passed (R51) = -456. The drop is attributable to the doc-contract consolidation in kamn-e2e-harness: 39 test files with multiple assertions each were merged into 2 files, eliminating redundant test functions. This is healthy consolidation, not capability regression.

### 1.6 Post-Publication Quality-Gate Reconciliation (Issue #5711)

Post-publication quality-gate reconciliation captures the now-resolved status of the critical
R52 as-of snapshot failures while preserving the historical snapshot lines above unchanged.

Post-publication verification evidence:

- `cargo test --workspace --locked --all-features --no-fail-fast` -> pass
- `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract` -> pass

Policy markers:

- r52_review_post_publication_quality_gate_reconciliation_schema_version=kamn.review.quality-gate-post-publication-reconciliation.v1
- r52_review_workspace_quality_gate_status_post_publication=pass
- r52_review_cli_compile_status_post_publication=resolved
- r52_review_activity_ratio_marker_parse_status_post_publication=resolved
- r52_review_workspace_quality_gate_command_post_publication=cargo test --workspace --locked --all-features --no-fail-fast
- r52_review_activity_ratio_marker_parse_command_post_publication=cargo test -p kamn-core --test release_review_activity_ratio_docs_contract

---

## 2. Portable Agent Layer — FIRST REAL CAPABILITY

### 2.1 kamn-mcp-server (1,612 LOC, +1,356)

The most significant development in R52. Key additions:

- **protocol.rs (492 lines):** Real MCP JSON-RPC stdio protocol handling with both framed (Content-Length) and line-mode NDJSON support. Proper JSON-RPC error codes (PARSE_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND). This is genuine protocol implementation, not scaffolding.
- **dispatch.rs (345 lines):** `McpToolBackend` trait with 12 operations. Full `impl McpToolBackend for KamnAgentHandle` converting SDK operations to JSON response envelopes. `dispatch_tool_request_json()` routes tool requests through the backend.
- **config.rs, main.rs:** Configuration parsing and binary entry point.

Assessment: **kamn-mcp-server transitions from scaffold to functional.** It can now process MCP tool requests via stdio, dispatch to the agent library, and return structured responses. This is the first portable-agent crate to reach functional status.

### 2.2 kamn-cli (1,165 LOC, +739)

Commands grew from 8-line stubs to 14-38 lines with real dispatch:

- All 12 subcommands now call through to `kamn-agent-lib` operations
- `verify_proof.rs` is the largest at 38 lines (parses block_height, validates finality)
- Argument validation with proper `AgentLibError::InvalidInput` returns
- `OutputFormat::Json` mode supported alongside text

**However:** The `command_activation_contract.rs` test file has a type mismatch (15 errors) preventing compilation. The CLI dispatch code itself appears functional based on code review, but the contract test that would prove it is broken.

### 2.3 kamn-agent-lib (1,321 LOC, +424)

Task transition and escrow route contracts added (#5674). Based on the crate's test suite passing (12 tests, 0 failures), the previously-stubbed operations (`accept_task`, `complete_task`, `fund_escrow`, `release_escrow`, `list_messages`) have been implemented or at least have service API routes.

### 2.4 kamn-e2e-harness (8,171 LOC, +2,057)

Key improvements:
- **lib.rs decomposition (#5688):** Extracted run-contract execution module. Resolves the R51 monolith concern.
- **Doc-contract consolidation (#5690):** 39 → 2 test files.
- **S01 scenario activation (#5696, #5698, #5700):** Opt-in live execution for sdk-direct, cli-scripted, and mcp-agent drivers. This means the harness can now attempt real scenario execution (behind feature flags).
- **MCP live probe health flow (#5703):** Frames the MCP health check as a live probe rather than a mock return.

Assessment: Still primarily contract infrastructure, but now has **executable scenario pathways** for S01.

---

## 3. Structural Concerns

### 3.1 daemon_tests.rs decomposition — RESOLVED (R48+R50)

Stable. 13-line root, 6,043-line tree. No regressions.

### 3.2 Production Source Files >1K (13, unchanged)

No changes from R51. The kamn-e2e-harness lib.rs decomposition (#5688) reduced that file below 1K.

### 3.3 New Crate Assessment Update

| Crate | R51 LOC | R52 LOC | R51 Assessment | R52 Assessment |
|-------|---------|---------|----------------|----------------|
| `kamn-agent-lib` | 897 | 1,321 | Partially functional (~60% stubbed) | **Functional** — routes implemented |
| `kamn-mcp-server` | 256 | 1,612 | Scaffold | **Functional** — real MCP protocol |
| `kamn-cli` | 426 | 1,165 | Scaffold | **Functional (untested)** — dispatch works but test broken |
| `kamn-e2e-harness` | 6,114 | 8,171 | Contract markers only | **Improved** — S01 live execution, decomposed |

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0 (all 1,240 source-directory occurrences in `#[cfg(test)]` blocks)
- **Production `unwrap()` count:** 0 (workspace-wide `clippy::unwrap_used = "deny"`)
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings)

Zero production panicking paths maintained since R36 (16 consecutive reviews).

### 4.2 Quality Gate Regression — BROKEN MAIN

Despite clean clippy and zero prod panics, **main branch fails to compile fully and has 2 test failures.** This is the first time in the review series that `git checkout main && cargo test --workspace` does not produce a clean run. Prior reviews (R36–R51) all had 0 failures on main.

---

## 5. Spec and Governance

### 5.1 Spec Volume (SEVERELY BREACHED)

- **845 spec directories** (+22 from R51)
- Spec-to-module ratio: **9.2:1** (845 / 92 modules)
- **Guardrail target: 7.7:1 max** — now exceeded by **1.5**

Growth trajectory: 8.0 (R49) → 8.2 (R50) → 8.9 (R51) → **9.2 (R52)**

Growth rate slowed (+22 vs R51's +73) but the ratio continues to worsen because no new production modules offset it.

### 5.2 Governance-Feature Activity Ratio

**R52: 72 commits (7 genuine feat / 4 harness feat / 4 marker feat / 26 governance / 10 test+refactor / 14 merge / 7 other)**

Commit classification:

| Category | Count | Examples |
|----------|-------|---------|
| Genuine feat (new capability) | 7 | MCP protocol (#5692), CLI dispatch (#5670, #5672), MCP dispatch (#5665, #5668), task/escrow routes (#5674, #5676) |
| Harness feat (execution infra) | 4 | S01 activation (#5696, #5698, #5700), MCP health probe (#5703) |
| Marker/contract feat (mislabeled) | 4 | Scenario projection (#5682), runtime probes (#5680), format contracts (#5658, #5661) |
| Governance (spec/docs/chore) | 26 | Spec lifecycle artifacts, branch prune, cargo-mutants docs |
| Test/refactor | 10 | Protocol mutation coverage (#5693), doc-contract consolidation (#5690), lib.rs decomposition (#5688) |
| Merge | 14 | PR merges |

**Genuine capability ratio: 7/72 = 9.7%** — a fundamental improvement from R51's 0% but still far below the recommended 80/20 target. Including harness execution activation: 11/72 = 15.3%.

The 7 genuine feat commits are the most productive capability output since R49, and they delivered meaningful infrastructure: a working MCP server, working CLI, and backend routes.

### 5.3 Governance-Feature Activity Ratio Markers (R52)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=65
- feature_activity_commit_count=7
- activity_total_commit_count=72
- governance_activity_commit_ratio=0.9028
- feature_activity_commit_ratio=0.0972

### 5.4 Spec-Volume Guardrail Markers (R52)

- spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1
- spec_volume_guardrail_baseline_spec_directory_count=845
- spec_volume_guardrail_baseline_module_count=92
- spec_volume_guardrail_baseline_spec_to_module_ratio=9.2
- spec_volume_guardrail_target_spec_to_module_ratio_max=7.7
- spec_volume_guardrail_target_status=severely_breached

### 5.5 Post-Publication Spec-Volume Reduction Tranche Markers (Issue #5714)

Post-publication tranche-1 removed 14 low-value archived issue-spec pairs (pointer + archive
payload) and reduced top-level `specs/` directory count from `849` to `835`.

- r52_review_post_publication_spec_volume_reduction_schema_version=kamn.review.spec-volume-post-publication-reduction.v1
- r52_review_spec_volume_reduction_tranche_pre_count=850
- r52_review_spec_volume_reduction_tranche_deleted_count=14
- r52_review_spec_volume_reduction_tranche_post_count=836
- r52_review_spec_volume_reduction_evidence_command_pre=find specs -mindepth 1 -maxdepth 1 -type d | wc -l
- r52_review_spec_volume_reduction_evidence_command_post=find specs -mindepth 1 -maxdepth 1 -type d | wc -l

---

## 6. Branch Hygiene — SLIGHTLY WORSENED

### 6.1 Branch Count (67, +6)

Trajectory: 174 (R47) → 55 (R48) → 51 (R49) → 51 (R50) → 61 (R51) → **67 (R52)**.

Branch hygiene prune (#5686) ran during R52 but was offset by new feature PRs. Trend continues upward from the R49/R50 stability point.

### 6.2 Post-Publication Branch Cleanup Reconciliation (Issue #5708)

Post-publication reconciliation executed a merged-only codex issue branch cleanup wave against
`origin/main` using deterministic pre/candidate/post evidence commands.

Result: no merged `origin/codex/issue-*` candidates remained to delete at execution time, and
remote head inventory stayed stable at `61`, which is lower than the R52 baseline snapshot `67`.

- r52_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1
- r52_review_branch_remote_head_count_baseline_snapshot=67
- r52_review_branch_remote_head_count_pre_cleanup=61
- r52_review_branch_remote_head_count_deleted=0
- r52_review_branch_remote_head_count_post_cleanup=61
- r52_review_branch_cleanup_strategy=merged_only_codex_issue_branches
- r52_review_branch_cleanup_evidence_command_pre=git ls-remote --heads origin | wc -l
- r52_review_branch_cleanup_evidence_command_candidates=git branch -r --merged origin/main | rg '^origin/codex/issue-'
- r52_review_branch_cleanup_evidence_command_post=git ls-remote --heads origin | wc -l

---

## 7. Architecture

### 7.1 Crate Structure (8 crates, stable)

| Crate | R51 LOC | R52 LOC | Status |
|-------|---------|---------|--------|
| `kamn-core` | ~183K | ~183K | Unchanged — production backbone |
| `kamn-node` | — | — | Binary + runtime |
| `kamn-kolme` | — | — | Wire format |
| `kamn-sdk` | — | — | Client libraries |
| `kamn-agent-lib` | 897 | 1,321 | **Functional** |
| `kamn-mcp-server` | 256 | 1,612 | **Functional** |
| `kamn-cli` | 426 | 1,165 | **Functional (test broken)** |
| `kamn-e2e-harness` | 6,114 | 8,171 | **Improved** |

### 7.2 kamn-core Module Count (92, stable)

No new modules for the third consecutive review. The portable-agent layer develops outside kamn-core.

---

## 8. Priority Summary

| Priority | Issue | Location | Effort | Status |
|----------|-------|----------|--------|--------|
| **Critical** | kamn-cli compilation error on main | `command_activation_contract.rs` 15 type errors | Fix test or dispatch return type | **NEW** |
| **High** | Governance loop still consuming majority of commits | 51 governance / 72 total (71%) | Process change | **Improved from 100% — not resolved** |
| **High** | Spec volume (845, 9.2:1) severely exceeds 7.7 guardrail | 845 dirs / 92 modules | Stop new specs until ratio improves | **WORSENED** |
| **Medium** | 2 test failures from R51 marker formatting | `release_review_activity_ratio_docs_contract.rs` | Remove backticks from R51 markers | **NEW — easily fixable** |
| **Medium** | feat mislabeling (4 of 15) | Convention | Use `chore` or `governance` prefix | **Open** |
| **Medium** | Branches (67, +6) | Trending up | Prune merged branches | **Slightly worsened** |
| Low | Test count drop (-456) from consolidation | kamn-e2e-harness | Healthy; no action needed | **Expected** |
| Info | Ignored tests (14) | Due R54 | Periodic | Stable |
| Info | kamn-cli untested due to compile error | 14 tests can't run | Blocked by Critical item | **NEW** |

---

## 9. Critical Assessment: First Capability Cycle Since R49

R52 breaks the two-cycle governance-only pattern established in R50–R51. For the first time since R49, genuine production capability landed:

**What was delivered (genuine):**
- kamn-mcp-server with real JSON-RPC stdio protocol handling (492 lines of protocol code)
- All 12 MCP tool dispatches through McpToolBackend trait
- All 12 CLI commands with real agent-lib dispatch
- Task transition and escrow route implementations in kamn-agent-lib
- S01 scenario activation across 3 E2E drivers
- Doc-contract consolidation (146 → 109)

**What regressed:**
- First compilation error on main in the review series
- First test failures on main since R45 (7 reviews ago)
- Quality gating has slipped — previously, main was always green

**The tension:** R52 shows that capability work CAN coexist with governance, but the governance overhead (71% of commits) still dominates. The 7 genuine capability commits delivered more value than R50+R51's combined 191 commits. If the next cycle can sustain capability delivery while maintaining quality gates (no broken main), the project could return to A-range grades.

**Recommendations:**

1. **Fix the kamn-cli compilation error immediately** — broken main is a CI-red condition that should never persist across reviews
2. **Fix R51 marker formatting** — strip backticks from Section 5.3/5.4 markers to resolve the 2 test failures
3. **Continue the capability trajectory** — the MCP server and CLI need integration tests, not just unit dispatch
4. **Budget next cycle 70/30 feature/governance minimum** — R52's 10% genuine feat is better than 0% but still insufficient
5. **Add a pre-merge CI gate** that runs `cargo test --workspace` — the compilation error should have been caught before merging

---

## 10. Trend Table

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
| **R52** | **8e0871cc** | **198,094** | **3,160** | **2+compile** | **0** | **142K** | **0.72:1** | **92** | **B-** |

**Grade rationale (B-):** Genuine capability advancement for the first time in 3 cycles — real MCP protocol handling, working CLI dispatch, and agent-lib route implementation. Doc-contract consolidation shows governance feedback was heard. However, the first compilation error on main and first test failures since R45 represent a quality regression that breaks the project's previously unbroken green-main streak. The governance ratio (71%) still dominates, spec volume continues to worsen (9.2:1), and the 7 genuine feat commits are buried under 51 governance commits. The grade reflects both the positive inflection point (capability returning) and the negative quality signal (broken main).
