# KAMN: Production Service Next Steps (Truth-Refresh)

**Document date:** 2026-02-14 (refreshed 2026-02-15)
**Authoritative tracker:** #3333
**Scope:** reconcile roadmap claims with current implementation and open issue chains.

## Why This Refresh Exists
The original 2026-02-14 draft mixed a historical baseline snapshot with forward planning.
That produced contradictory signals (for example, marking ingress/storage as scaffolding when those chains were already delivered in later waves).

This refreshed version separates:
- delivered roadmap items and their closed chains,
- active production gaps (when present) and their issue chains,
- explicit local-heavy validation scope and CI cost boundaries.

## Status Truth Snapshot

| Item | Current Truth | Primary Chain |
|---|---|---|
| 1. HTTP ingress runtime | Delivered (`axum` server + auth/ws integration) | #3254 -> #3305 -> #3306 |
| 2. Persistent storage | Delivered (sqlite backend adapters + migration parity) | #3221 -> #3309 -> #3310 |
| 3. Real P2P transport | Delivered (live libp2p provider + lifecycle/fault hardening) | #3228 -> #3229 -> #3313 |
| 4. Transport-fed consensus pipeline | Delivered (transport-fed convergence + go/no-go evidence gate) | #3228 -> #3413 -> #3414 |
| 5. Serde API/domain serialization | Delivered | #3263 -> #3264 -> #3265 |
| 6. Prometheus/metrics contract | Delivered | #3263 -> #3268 -> #3269 |
| 7. Graceful shutdown path | Delivered | #3263 -> #3272 -> #3273 |
| 8. Request validation + route error contracts | Delivered | #3263 -> #3276 -> #3277 |
| 9. Compose topology hardening | Delivered | #3280 -> #3285 -> #3286 |
| 10. Combined runtime mode (`full`) | Delivered | #3280 -> #3281 -> #3282 |
| 11. Real I/O integration test matrix | Delivered | #3289 -> #3290 -> #3291 |
| 12. Structured API error envelope | Delivered | #3289 -> #3294 -> #3295 |
| 13. Rate limiting and request caps | Delivered | #3289 -> #3298 -> #3299 |

## Delivered Chains (Closed or Downstream-Closed)

### Item 1: Axum ingress
- Delivered chain: `#3254 -> #3305 -> #3306 -> (#3307, #3308)`.
- Runtime now serves through `axum::serve(...)` with route wiring in `crates/kamn-node/src/service_api_endpoint.rs`.

### Item 2: Sqlite persistence
- Delivered chain: `#3221 -> #3309 -> #3310 -> (#3354, #3355, #3311, #3312)`.
- Sqlite backend and adapters are live in `kamn-core` (`SqliteStoreBackend`, `Sqlite*SnapshotStore`).

### Items 5-13 (infrastructure maturity tranche)
- Serde: `#3263 -> #3264 -> #3265 -> (#3266, #3267)`.
- Prometheus metrics: `#3263 -> #3268 -> #3269 -> (#3270, #3271)`.
- Graceful shutdown: `#3263 -> #3272 -> #3273 -> (#3274, #3275)`.
- Request validation: `#3263 -> #3276 -> #3277 -> (#3278, #3279)`.
- Compose topology: `#3280 -> #3285 -> #3286 -> (#3360, #3361, #3287, #3288)`.
- Runtime-mode full: `#3280 -> #3281 -> #3282 -> (#3358, #3359, #3283, #3284)`.
- Real-I/O integration matrix: `#3289 -> #3290 -> #3291 -> (#3362, #3363, #3292, #3293)`.
- Structured errors: `#3289 -> #3294 -> #3295 -> (#3364, #3365, #3296, #3297)`.
- Rate limiting: `#3289 -> #3298 -> #3299 -> (#3366, #3367, #3300, #3301)`.

## Tier-5 Closure Roll-up (Delivered)

### Item 3: Real libp2p transport delivery
- Delivered chain: `#3228 -> #3229 -> #3313 -> (#3356, #3314, #3315, #3319, #3470)`.
- Delivery includes deterministic swarm composition, lifecycle/discovery fail-closed evidence, and local-heavy fault-matrix lane policy validation.
- Reference scripts:
  - `scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh`
  - `scripts/runtime/check_live_transport_fault_matrix_live_policy.sh`

### Item 4 follow-on: Transport-fed consensus convergence
- Delivered chain: `#3228 -> #3413 -> #3414 -> (#3443, #3444, #3446, #3447, #3448)`.
- Delivery includes transport-fed pipeline wiring, partition/publish-drop/churn convergence drills, and deterministic go/no-go artifact gating.

### Composed full-stack E2E against `kolme_fork`
- Delivered chain: `#3333 -> #3419 -> #3420 -> (#3432, #3433, #3434)`.
- Delivery includes composed local-heavy integration evidence bundle, release go/no-go linkage, and architecture/documentation lineage.
- Promotion evidence gate lineage now fails closed when operator runbook markers drift or go missing (`milestone_review_operator_runbook_missing`, `milestone_review_operator_runbook_markers_missing`) in `scripts/deploy/gonogo_evidence_contract.py`.
- Local validation drill override for runbook-marker regression uses `KAMN_GONOGO_RUNBOOK_DOC_FILE=<path>`.
- Milestone closure criteria (chain `#3716 -> #3718`) for release promotion now additionally require deterministic combined lane markers in `kamn.runtime.go-no-go-gate-report.v1`:
  - `combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1`
  - `combined_transport_reason_codes=["fork_choice_stale_block_height"]`
  - `combined_kolme_runtime_reason_code` in `{"not_run","live_runtime_integration_passed"}`
  - `kolme_runtime_commit_failure_taxonomy_version=v1`
  - `kolme_fixture_profile=real-node-non-synthetic-v1` and `kolme_fixture_profile_version=v1`
  - `combined_lane_marker_contract_status=verified`
- Milestone bundle reason-code surface fails closed for marker/taxonomy drift via:
  - `milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch`
  - `milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch`
  - `milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch`
  - `milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch`
  - `milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch`
  - `milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch`
  - `milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch`
  - `milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch`

### Runtime observability reason-code/checkpoint projection
- Delivered chain: `#3333 -> #3471 -> #3472 -> #3473 -> #3474 -> #3490`.
- Delivery includes deterministic `reason_code` and checkpoint-failure counters in daemon and `kolme-live` reports, parity projection into `/metrics`, `/healthz`, `/metrics.stream`, and bounded local scrape failure-drill validation.
- Key validation script: `scripts/runtime/test_validate_local_observability_scrape_live.sh`.
- Validation marker remains tracked in `main_tests::observability_endpoint_tests::functional_observability_endpoint_readiness_reason_taxonomy_covers_dependency_probe_matrix` (issue `#3489`).

### Async observability ingress compatibility hardening
- Active chain: `#3508 -> #3509 -> #3513 -> #3514`.
- Scope adds deterministic negative-matrix compatibility contracts for:
  - unknown path (`/unknown`) -> `404 not found`
  - malformed method/request parsing -> fail-closed route handling
  - idle-timeout failure path -> deterministic timeout reason string
- Reference validations:
  - `scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
  - `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
  - `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
  - `scripts/ci/test_check_observability_endpoint_drift_contract.sh`
- Readiness parity selector coverage:
  - `main_tests::observability_endpoint_tests::functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces` (issue `#3519`).

### Docs truth synchronization (this tranche)
- Delivered chain: `#3333 -> #3424 -> #3425 -> #3426`.
- Scope: keep this document and CI docs-contract guards synchronized with real state as issue chains close.

## Active Open Chains
- None in this document scope as of 2026-02-15.
- New production gaps should be decomposed under `#3333` with a new epic/story/task/subtask chain before implementation work starts.

## Cost and CI Policy Boundaries
- Heavy local integration run-mode lanes remain excluded from `ci-fast-gate` and fast `ci-tools` blocks.
- Deterministic dry-run contract checks remain in PR path.
- Local-heavy/live-node validations remain opt-in and bounded for cost control.
- Reference policy doc: `docs/ci/strategy.md`.

### Script-Surface Trend Governance Refresh (Task #3740)
- Combined shell-surface baseline refreshed to current post-migration snapshot:
  - `fixtures/ci/combined_shell_surface_trend_baseline.json` (`script_count=408`, `shell_line_total=33972`, `rust_line_total=115293`, `shell_to_rust_ratio=0.294658`).
- Script-surface budget envelope refreshed for current dispatcher-migration state:
  - `.ci/script-surface-budget.env` (`SHELL_LINE_TOTAL_MAX=34000`).
  - `.ci/script-surface-baseline.env` (`SCRIPT_COUNT_BASELINE=408`, `SHELL_LINE_TOTAL_BASELINE=33972`).
- Combined trend gate remains fail-closed for future drift using:
  - `scripts/ci/check_combined_shell_surface_trend_policy.sh`
  - `fixtures/ci/combined_shell_surface_trend_thresholds.json`

## Validation Commands (Low-Cost Truth Guard)
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `bash scripts/ci/test_kolme_live_integration_architecture_contract.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`

## Historical Baseline (Superseded)
The earlier 2026-02-14 draft language that described ingress/storage/runtime composition as unresolved scaffolding is retained only as historical context and is superseded by the issue-chain status above.

Use #3333 as the single source of truth for roadmap status transitions.
