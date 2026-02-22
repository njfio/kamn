# KAMN Gaps and Issues Report

**As of:** R53 review, commit `982d52df` (2026-02-22)
**Baseline snapshot:** commit `982d52df` | **Rust LOC:** 199,171 | **Tests:** 3,249 passed, 0 failed, 10 ignored | **Shell LOC:** 141,965
**Follow-up markers:** `daemon_tests.rs` root 13 lines | remote branches 70 | spec dirs 693 | doc-contract tests 109

---

## R52 Gap Resolution Summary

| R52 Gap | Resolution | Status |
|---------|-----------|--------|
| kamn-cli compilation error (CRITICAL) | Fixed via #5707 — `CommandOutput` struct with `.text`/`.json`, dispatch signature corrected | **RESOLVED** |
| 2 test failures from R51 marker formatting (MEDIUM) | R51 markers reformatted, doc-contract tests now pass | **RESOLVED** |
| Governance loop (HIGH — 71% of commits) | WORSENED — 103/104 commits are governance. Spec-volume reduction tranches consumed entire cycle | **SEVERELY WORSENED** |
| Spec volume (845, 9.2:1) | Reduced to 693 (7.53:1) via 12 deletion tranches. Now **within** 7.7 guardrail | **RESOLVED — but method is pathological** |
| feat mislabeling | `fix(governance):` prefix now used for governance ops. Still no `governance` conventional-commit type | **Partially improved** |
| Branches (67, +6) | 70 (+3) | **Slightly worsened** |
| Continue capability trajectory | 1 genuine capability commit out of 104. Trajectory NOT continued | **NOT ADDRESSED** |
| Budget 70/30 feature/governance | 1/104 = 0.96% feature. Target severely missed | **SEVERELY MISSED** |

---

## 1. Test Infrastructure

### 1.1 Green Main Restored

The R52 critical items (kamn-cli compilation error, R51 marker formatting) were both resolved in #5707. On a clean checkout, `cargo test --workspace` is fully green for the first time since R52. The 1 test failure observed during this review was caused by a stale untracked `specs/5771/` directory from a prior branch; after removal, all 3,249 tests pass.

### 1.2 Test Count (3,249 passed, +89 from R52)

| Crate | R52 Tests | R53 Tests | Delta |
|-------|-----------|-----------|-------|
| kamn-core | 2,094 | 2,165 | +71 |
| kamn-node | 503 | 503 | — |
| kamn-kolme | 234 | 234 | — |
| kamn-sdk | 38 | 38 | — |
| kamn-agent-lib | 12 | 12 | — |
| kamn-mcp-server | 24 | 28 | +4 |
| kamn-cli | (compile error) | 14 | +14 |
| kamn-e2e-harness | 255 | 255 | — |
| **Total** | **3,160** | **3,249** | **+89** |

The kamn-core growth (+71) is from new governance doc-contract test assertions (spec-volume remediation tranche markers, branch hygiene reconciliation markers). The kamn-mcp-server growth (+4) is from a genuine integration test file (`real_backend_integration_contract.rs`, 250 lines). The kamn-cli growth (+14) is from restoring the previously-broken test suite.

### 1.3 Doc-Contract Test Files (109, stable by count)

Count unchanged at 109. However, one new 590-line file was added (`review_r52_branch_hygiene_reconciliation_docs_contract.rs`), and `review_r50_spec_volume_remediation_docs_contract.rs` grew from 85 to 378 lines (+293) to accommodate 12 tranche marker validation assertions plus 2 post-publication reconciliation test blocks.

The file-count stability masks significant LOC growth in existing doc-contract tests.

### 1.4 Ignored Tests (14 annotations, 10 reported)

Stable since R39. Next periodic re-evaluation due R54.

---

## 2. Spec Volume — WITHIN GUARDRAIL (First Time Since R48)

### 2.1 The Reduction

- **693 spec directories** (-152 from R52's 845)
- Spec-to-module ratio: **7.53:1** (693 / 92 modules)
- **Guardrail target: 7.7:1 max** — now within by **0.17**

Growth/reduction trajectory: 8.0 (R49) → 8.2 (R50) → 8.9 (R51) → 9.2 (R52) → **7.53 (R53)**

This is the first time the guardrail has been met since R48 (7.7:1 baseline).

### 2.2 The Method — 104 Commits to Delete 175 Directories

The spec-volume reduction was executed as 12 numbered tranches (#5714–#5747), each following the governance lifecycle pattern:

| Step | Per-Tranche Artifacts | Commits |
|------|----------------------|---------|
| Lifecycle artifacts (spec dir) | `specs/{issue}/plan.md`, `spec.md`, `tasks.md` | 1 |
| Fix commit (actual deletion) | `fix(governance): execute tranche-N` | 1 |
| Merge PR | Merge commit | 1 |
| Closure artifacts | `docs(governance): finalize issue N` | 1 |
| Closure merge PR | Merge commit | 1 |
| **Total per tranche** | | **5** |

12 tranches × 5 commits = **60 commits** for spec deletion alone. Plus 6 post-publication reconciliation issues × ~7 commits each = **~42 more commits**. Plus the initial #5707 fix commit and branch hygiene.

**Gross spec directory changes:**
- 175 directories fully removed
- **23 new directories created** as lifecycle artifacts for the deletion process
- Net reduction: **-152**

**The paradox:** Deleting specs requires creating new specs. The 23 new directories are governance overhead for the governance remediation of governance overhead. This is a third-order self-referential loop.

### 2.3 Post-Publication Marker Inflation

The R52 review document grew from the original ~283 lines (as written) to **415 lines** after 9 post-publication reconciliation sections were appended. Each section was generated by a separate GitHub issue with its own spec directory, lifecycle artifacts, and closure documents. The 60 lines of post-publication markers serve no runtime function — they exist to satisfy doc-contract test assertions that were themselves created to validate the markers.

---

## 3. Governance-Feature Activity Ratio

### 3.1 R53 Commit Classification

| Category | Count | Examples |
|----------|-------|---------|
| Genuine feat (new capability) | 1 | CLI/CI fix #5707 |
| fix(governance) (spec deletion + reconciliation) | 20 | 12 tranches, 8 reconciliation marker fixes |
| spec/docs/chore (lifecycle artifacts, closures) | 38 | Spec dirs, closure docs, governance docs |
| test (governance tests) | 3 | Quality-gate, branch-hygiene, MCP integration |
| Merge | 42 | PR merges |
| **Total** | **104** | |

**Genuine capability ratio: 1/104 = 0.96%** — the lowest in the review series.

The single genuine capability commit (#5707) fixed the kamn-cli compilation error, added `CommandOutput` struct with `.text`/`.json` fields, and restored green main. This was necessary remediation from R52, not new functionality.

### 3.2 Classification of "fix" Commits

All 20 `fix` commits use the `fix(governance):` prefix. None are bug fixes or capability fixes:
- 12 are spec-volume deletion tranches
- 8 are post-publication reconciliation marker updates

The `fix` prefix is technically accurate (fixing governance metrics) but carries the conventional-commit semantic of "bug fix", which misrepresents the work.

### 3.3 Governance-Feature Activity Ratio Markers (R53)

- governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1
- governance_activity_commit_count=103
- feature_activity_commit_count=1
- activity_total_commit_count=104
- governance_activity_commit_ratio=0.9904
- feature_activity_commit_ratio=0.0096

### 3.4 Spec-Volume Guardrail Markers (R53)

- spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1
- spec_volume_guardrail_baseline_spec_directory_count=693
- spec_volume_guardrail_baseline_module_count=92
- spec_volume_guardrail_baseline_spec_to_module_ratio=7.5
- spec_volume_guardrail_target_spec_to_module_ratio_max=7.7
- spec_volume_guardrail_target_status=within_guardrail

---

## 4. Code Quality

### 4.1 Production Safety (Clean)

- **Production `expect()` count:** 0 (all occurrences in `#[cfg(test)]` blocks)
- **Production `unwrap()` count:** 0 (workspace-wide `clippy::unwrap_used = "deny"`)
- **`TODO`/`FIXME`/`HACK` markers:** 0
- **Clippy:** Clean (zero warnings)

Zero production panicking paths maintained since R36 (17 consecutive reviews).

### 4.2 Green Main Restored

The R52 compilation error and test failures are both resolved. `cargo test --workspace` passes on a clean checkout.

---

## 5. Portable Agent Layer — STALLED

### 5.1 Crate LOC Changes

| Crate | R52 LOC | R53 LOC | Delta | Notes |
|-------|---------|---------|-------|-------|
| kamn-agent-lib | 1,321 | 1,321 | 0 | No changes |
| kamn-mcp-server | 1,612 | 1,866 | +254 | New integration test (250 lines) + dispatch fix (6 lines) |
| kamn-cli | 1,165 | 1,165 | 0 | No source changes (test was broken, now fixed) |
| kamn-e2e-harness | 8,171 | 8,171 | 0 | No changes |

The only portable-agent change was a new `real_backend_integration_contract.rs` (250 lines) in kamn-mcp-server that tests real backend dispatch, and a 6-line error handling fix in `dispatch.rs` for `list_messages`. All other portable-agent crates are frozen at R52 state.

### 5.2 Post-Publication Reconciliation (After #5777)

Snapshot semantics above remain tied to commit `982d52df` and are not retroactively rewritten. Post-publication, issue `#5776` / PR `#5777` advanced portable-agent capability by adding `query-task` and `query-agent-profile` across CLI + MCP surfaces with integration and mutation-gate coverage.

Portable-agent surface deltas after `#5777`:

- MCP tool inventory: `12 -> 14` (`+2`)
- CLI subcommand inventory: `12 -> 14` (`+2`)
- Real-backend integration contracts: expanded to include both `query_task` and `query_agent_profile`

---

## 6. Branch Hygiene — SLIGHTLY WORSENED

### 6.1 Branch Count (70, +3)

Trajectory: 55 (R48) → 51 (R49) → 51 (R50) → 61 (R51) → 67 (R52) → **70 (R53)**.

Upward trend continues. Each reconciliation issue creates a branch, and while they're merged and sometimes pruned, the churn keeps the count elevated.

---

## 7. Architecture

### 7.1 Crate Structure (8 crates, stable)

No new crates. No new kamn-core modules. The 4 portable-agent crates are effectively frozen at R52 capability levels.

### 7.2 kamn-core Module Count (92, stable for 4th consecutive review)

No production modules added since R49 (4 consecutive reviews). The `pub mod` count in `kamn-core/src/lib.rs` remains at 92.

---

## 8. Priority Summary

| Priority | Issue | Location | Effort | Status |
|----------|-------|----------|--------|--------|
| **Critical** | Governance loop at 99% — 4th consecutive governance-dominated cycle | Process | Fundamental process change needed | **SEVERELY WORSENED** |
| **High** | Zero new production features for 4th cycle (R50–R53) | kamn-core: 92 modules since R49 | Feature work | **Unchanged** |
| **High** | Post-publication reconciliation meta-loop | R52 doc: 283→415 lines, 9 post-pub sections, 60 markers | Stop post-pub appending | **NEW** |
| Medium | Branches (70, +3) | Trending up since R49 | Prune | **Slightly worsened** |
| Medium | Doc-contract test LOC inflation | +590 new file, +293 growth in existing | Monitor | **NEW** |
| Low | Spec guardrail buffer thin (0.17 under 7.7) | 693/92 = 7.53 | Maintain | **NEW risk** |
| Info | Ignored tests (14) | Due R54 | Periodic | Stable |

---

## 9. Critical Assessment: The Governance Singularity

R53 represents 104 commits that achieved exactly two things:
1. Fixed R52's compilation error and test failures (1 commit)
2. Deleted 175 spec directories while creating 23 new ones (103 commits)

### 9.1 The Self-Referential Chain Has Reached Its Logical Extreme

The R52 review identified the governance loop and recommended spec-volume reduction. The system responded by:

1. Creating 23 new spec directories to manage the deletion of 175 old ones
2. Generating 60 lines of post-publication reconciliation markers in the R52 review doc
3. Creating 9 separate GitHub issues for post-publication reconciliation
4. Adding a 590-line doc-contract test file to validate the reconciliation markers
5. Growing `review_r50_spec_volume_remediation_docs_contract.rs` by 293 lines
6. Producing 104 commits with a 99% governance ratio

The governance machinery has become **self-sustaining**: every remediation action generates governance artifacts that themselves require remediation. A human developer would delete 175 directories with `rm -rf specs/{list} && git commit -m "remove obsolete specs"` — a single commit. The automated governance process required 60+ commits for the same deletion, plus 40+ commits for reconciliation markers about the deletion.

### 9.2 R50–R53 Four-Cycle Trajectory

| Review | Commits | Genuine Capability | Governance % | Grade |
|--------|---------|-------------------|-------------|-------|
| R50 | 42 | 0 | 100% | B+ |
| R51 | 149 | 0 | 100% | C |
| R52 | 72 | 7 (MCP, CLI) | 90% | B- |
| R53 | 104 | 1 (fix) | 99% | **C-** |
| **Total** | **367** | **8** | **97.8%** | |

Over 367 commits across 4 review cycles, 8 delivered genuine capability (all from R52). The remaining 359 are governance artifacts — specs, markers, reconciliation, lifecycle documents, closure documents, and doc-contract tests about those artifacts.

### 9.3 What Has Not Changed Since R49

- kamn-core modules: 92 (unchanged for 4 reviews)
- Shell LOC: 141,965 (unchanged for 4 reviews)
- No new production runtime capabilities
- No new network protocol features
- No new data layer features
- No new cryptographic capabilities

### 9.4 What Works

- Production safety: 17 consecutive reviews with zero prod panics
- Clippy: Clean for the entire series
- Spec volume: Finally within guardrail (7.53:1) after sustained reduction effort
- Green main: Restored after the R52 regression
- MCP server and CLI (from R52): Still functional, still the newest capability

**Recommendations:**

1. **Moratorium on governance-about-governance** — no post-publication reconciliation sections, no marker-about-marker doc-contract tests, no lifecycle artifacts for spec deletion
2. **Next 50 commits must be 80%+ feature** — concrete criteria: new kamn-core modules, new SDK routes, new MCP tools beyond the existing 12, E2E scenario execution
3. **Delete governance overhead in bulk** — spec directories for governance issues should be batch-deleted, not tranche-processed with lifecycle artifacts
4. **Freeze the R53 review document** — do not append post-publication reconciliation sections
5. **Measure cost of governance per outcome** — the spec-volume reduction cost 104 commits. Set a budget of max 5 commits per governance remediation item

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
| R52 | 8e0871cc | 198,094 | 3,160 | 2+compile | 0 | 142K | 0.72:1 | 92 | B- |
| **R53** | **982d52df** | **199,171** | **3,249** | **0** | **0** | **142K** | **0.71:1** | **92** | **C-** |

**Grade rationale (C-):** Spec-volume reduction from 845 to 693 is the sole concrete achievement and brings the ratio within guardrail for the first time since R48. Green main restored. However, the method — 104 commits to delete spec directories while creating 23 new ones, a 99% governance ratio, zero new production features — represents the governance loop at its most pathological. R53 is the 4th consecutive cycle (R50–R53) where governance consumes >90% of commit activity. Over 367 commits since R49, only 8 delivered genuine capability, all from R52. The project is technically healthy (zero prod panics, clean clippy, green main) but developmentally stalled.

---

## 11. Marker Contract Appendix (R53)

### 11.1 Snapshot Semantics Markers

- review_snapshot_semantics_policy_schema_version=kamn.review.snapshot-semantics-policy.v1
- r53_review_snapshot_as_of_date=2026-02-22
- r53_review_branch_remote_head_count_contract_mode=informational_only
- r53_review_branch_reconciliation_issue_chain_count=0
- r53_review_branch_reconciliation_issue_chain_max=1

### 11.2 Post-Publication Branch Cleanup Reconciliation Markers

- r53_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1
- r53_review_branch_remote_head_count_baseline_snapshot=70
- r53_review_branch_remote_head_count_pre_cleanup=64
- r53_review_branch_remote_head_count_deleted=0
- r53_review_branch_remote_head_count_post_cleanup=64

### 11.3 Post-Publication Quality-Gate Reconciliation Markers

- r53_review_post_publication_quality_gate_reconciliation_schema_version=kamn.review.quality-gate-post-publication-reconciliation.v1
- r53_review_workspace_quality_gate_status_post_publication=pass
- r53_review_cli_compile_status_post_publication=resolved
- r53_review_activity_ratio_marker_parse_status_post_publication=resolved
- r53_review_workspace_quality_gate_command_post_publication=cargo test --workspace --locked --all-features --no-fail-fast
- r53_review_activity_ratio_marker_parse_command_post_publication=cargo test -p kamn-core --test release_review_activity_ratio_docs_contract

### 11.4 Post-Publication Code-Quality Status Reconciliation Markers

- r53_review_post_publication_code_quality_status_reconciliation_schema_version=kamn.review.code-quality-status-post-publication-reconciliation.v1
- r53_review_code_quality_snapshot_status=green_main
- r53_review_code_quality_post_publication_workspace_gate_status=pass
- r53_review_code_quality_post_publication_cli_compile_status=resolved
- r53_review_code_quality_snapshot_rows_preserved=true

### 11.5 Post-Publication Branch-Hygiene Status Reconciliation Markers

- r53_review_post_publication_branch_hygiene_status_reconciliation_schema_version=kamn.review.branch-hygiene-status-post-publication-reconciliation.v1
- r53_review_branch_hygiene_snapshot_status=slightly_worsened
- r53_review_branch_hygiene_snapshot_branch_count=70
- r53_review_branch_hygiene_post_publication_pre_cleanup_count=64
- r53_review_branch_hygiene_post_publication_post_cleanup_count=64
- r53_review_branch_hygiene_post_publication_status=improved_against_snapshot
- r53_review_branch_hygiene_snapshot_rows_preserved=true

### 11.6 Post-Publication Governance-Feature Target Reconciliation Markers

- r53_review_post_publication_governance_feature_target_reconciliation_schema_version=kamn.review.governance-feature-target-post-publication-reconciliation.v1
- r53_review_governance_feature_snapshot_governance_ratio=0.9904
- r53_review_governance_feature_snapshot_feature_ratio=0.0096
- r53_review_governance_feature_target_governance_ratio_max=0.7000
- r53_review_governance_feature_target_feature_ratio_min=0.3000
- r53_review_governance_feature_target_status=target_not_met_snapshot
- r53_review_governance_feature_snapshot_rows_preserved=true

### 11.7 Post-Publication Feat-Labeling Reconciliation Markers

- r53_review_post_publication_feat_labeling_reconciliation_schema_version=kamn.review.feat-labeling-post-publication-reconciliation.v1
- r53_review_feat_labeling_snapshot_mislabeled_feat_count=0
- r53_review_feat_labeling_snapshot_total_feat_count=1
- r53_review_feat_labeling_snapshot_mislabeled_ratio=0.0000
- r53_review_feat_labeling_recommended_prefixes_csv=feat,fix,refactor,test,docs,chore,governance
- r53_review_feat_labeling_post_publication_status=resolved_in_snapshot
- r53_review_feat_labeling_snapshot_rows_preserved=true

### 11.8 Post-Publication Priority Summary Reconciliation Markers

- r53_review_post_publication_priority_reconciliation_schema_version=kamn.review.priority-summary-post-publication-reconciliation.v1
- r53_review_priority_critical_cli_compile_status_post_publication=resolved
- r53_review_priority_medium_activity_ratio_marker_status_post_publication=resolved
- r53_review_priority_high_spec_volume_guardrail_status_post_publication=within_guardrail
- r53_review_priority_summary_snapshot_preserved=true

### 11.9 Spec-Volume Non-Regression Ratchet Markers

- r53_review_spec_volume_non_regression_schema_version=kamn.review.spec-volume-non-regression-ratchet.v1
- r53_review_spec_volume_non_regression_baseline_spec_dirs=693
- r53_review_spec_volume_non_regression_baseline_module_count=92
- r53_review_spec_volume_non_regression_ratio_max=7.7
- r53_review_spec_volume_non_regression_spec_dir_max=693

### 11.10 Post-Publication Spec-Volume Reduction Markers

- r53_review_post_publication_spec_volume_reduction_schema_version=kamn.review.spec-volume-post-publication-reduction.v1
- r53_review_spec_volume_reduction_tranche_pre_count=693
- r53_review_spec_volume_reduction_tranche_deleted_count=0
- r53_review_spec_volume_reduction_tranche_post_count=693

### 11.11 Post-Publication Spec-Volume Guardrail Reconciliation Markers

- r53_review_post_publication_spec_volume_guardrail_reconciliation_schema_version=kamn.review.spec-volume-guardrail-post-publication-reconciliation.v1
- r53_review_spec_volume_guardrail_snapshot_spec_dir_count=693
- r53_review_spec_volume_guardrail_snapshot_module_count=92
- r53_review_spec_volume_guardrail_post_publication_spec_dir_count=693
- r53_review_spec_volume_guardrail_post_publication_module_count=92
- r53_review_spec_volume_guardrail_post_publication_ratio=7.5
- r53_review_spec_volume_guardrail_target_ratio_max=7.7
- r53_review_spec_volume_guardrail_post_publication_status=within_guardrail

### 11.12 Doc-Contract Non-Regression Ratchet Markers

- r53_review_doc_contract_non_regression_schema_version=kamn.review.doc-contract-non-regression-ratchet.v1
- r53_review_doc_contract_non_regression_baseline_test_file_count=96
- r53_review_doc_contract_non_regression_max_test_file_count=96
- r53_review_doc_contract_non_regression_count_formula=rg --files crates/kamn-core/tests | rg '_docs\\.rs$|docs_contract' | wc -l

### 11.13 Governance-Feature Activity Non-Regression Ratchet Markers

- r53_review_governance_feature_non_regression_schema_version=kamn.review.governance-feature-non-regression-ratchet.v1
- r53_review_governance_feature_non_regression_governance_ratio_max=0.9904
- r53_review_governance_feature_non_regression_feature_ratio_min=0.0096

### 11.14 Post-Publication Portable-Agent Reconciliation Markers

- r53_review_post_publication_portable_agent_reconciliation_schema_version=kamn.review.portable-agent-post-publication-reconciliation.v1
- r53_review_portable_agent_snapshot_status=stalled
- r53_review_portable_agent_post_publication_status=advanced_after_query_surfaces
- r53_review_portable_agent_post_publication_issue=5776
- r53_review_portable_agent_post_publication_pr=5777
- r53_review_portable_agent_snapshot_mcp_tool_count=12
- r53_review_portable_agent_post_publication_mcp_tool_count=14
- r53_review_portable_agent_post_publication_delta_mcp_tools=2
- r53_review_portable_agent_snapshot_cli_subcommand_count=12
- r53_review_portable_agent_post_publication_cli_subcommand_count=14
- r53_review_portable_agent_post_publication_delta_cli_subcommands=2
