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
- Legacy R26 umbrella chain remains active with refreshed truth scope:
  - `#3626` (epic, refreshed 2026-02-15), with active stories:
    - `#3630` TLS completion + release-governance integration.
    - `#3631` anti-flake merge gate reliability.
    - `#3632` unified API-observability contract hardening (post-migration governance).
- New R26.5 closure tranche opened for remaining production-service operational gaps:
  - Milestone: `R26.5 Observability and transport resilience hardening` (`#37`).
  - Epic chain: `#3333 -> #3772`.
  - Story chains:
    - `#3772 -> #3773` tracing standardization + observability-serving hardening.
    - `#3772 -> #3774` shared Kolme transport retry/reconnect hardening.
  - Task chains:
    - `#3773 -> #3775 -> (#3782, #3783)`.
    - `#3773 -> #3781 -> (#3789, #3788)`.
    - `#3773 -> #3776 -> (#3784, #3785)`.
    - `#3774 -> #3778 -> (#3790, #3791)`.
    - `#3774 -> #3779 -> (#3793, #3792)`.
    - `#3774 -> #3780 -> (#3794, #3795)`.
- New production gaps continue to be decomposed under `#3333` with epic/story/task/subtask hierarchy before implementation work starts.

### R26.2 Signer Secret-Lifecycle Contract Closure
- Closure chain: `#3911 -> #3915 -> (#3916, #3917)`.
- Deterministic closure markers:
  - `signer_secret_hardening_closure_chain=#3911->#3915->(#3916,#3917)`
  - `signer_secret_lifecycle_policy_contract_status=active`
  - `signer_secret_lifecycle_policy_contract_version=v1`
  - `signer_secret_lifecycle_docs_contract_status=active`
  - `signer_secret_lifecycle_contract_guard_command=cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract -- --nocapture`
- Closure criteria remains fail-closed on fallback secret reason-code drift and missing lifecycle marker declarations across CI/docs.

### R27.49 Partition-Healing / Convergence Governance
- Active chain: `#4593 -> #4594 -> #4596 -> (#4600, #4601)`.
- Current delivered markers and policy contracts for partition/rejoin reconciliation governance include:
  - `transport_evidence_schema_version=kamn.runtime.libp2p-transport-transition-evidence.v1`
  - `transport_evidence_normalization_status=verified`
  - `transport_evidence_source_contract_status=verified`
  - `reconciliation_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1`
  - `reconciliation_reason_codes_csv=reconciliation_partition_transition_failed,reconciliation_rejoin_transition_failed,reconciliation_publish_drop_recovery_failed,reconciliation_peer_churn_recovery_failed,reconciliation_split_head_unresolved,reconciliation_replay_instability,reconciliation_fixture_contract_failed,reconciliation_unclassified_scenario_failed,reconciliation_runtime_budget_exceeded,reconciliation_ci_fast_gate_failed`
- Deterministic fail-closed drift markers now include:
  - `block_reconciliation_partition_rejoin_policy_transport_evidence_normalization_status_mismatch`
  - `block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_csv_mismatch`
- Active failover governance chain under the same tranche:
  - `#4593 -> #4595 -> #4599 -> (#4606, #4607)`.
  - delivered failover preflight governance markers:
    - `failover_promotion_gate_status=verified`
    - `live_node_drift_parity_status=verified`
    - `ci_local_promotion_budget_boundary_status=verified`
    - `failover_readiness_reason_taxonomy_version=kamn.runtime.failover-readiness-reason-taxonomy.v1`
    - `failover_readiness_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded`
  - deterministic fail-closed drill reasons for parity/budget drift:
    - `failover_readiness_progress_stalled`
    - `live_node_drift_marker_parity_mismatch`
    - `ci_local_promotion_budget_boundary_exceeded`

### R27.53 Closure Tranche (`#4653`)
- Delivered chain: `#4653 -> (#4654, #4655) -> (#4656, #4657, #4658, #4659) -> (#4660-#4667)`.
- Closure highlights:
  - deployment preflight startup-budget governance now fails closed with deterministic taxonomy:
    - `preflight_budget_exceeded`
    - `startup_latency_budget_status_mismatch`
    - `startup_latency_budget_reason_code_mismatch`
  - managed-signer rollout governance now fails closed for promotion/custody drift:
    - `signer_rotation_promotion_stalled`
    - `quorum_evidence_custody_sha256_mismatch`
  - ci-local promotion budget boundary enforced by managed-signer contract lane:
    - `ci_local_promotion_budget_boundary_status=verified`

### R27.16 Retry/TLS CI Smoke Closure
- Delivered chain: `#4100 -> #4104 -> (#4111, #4112)`.
- Closure highlights:
  - retry/TLS smoke governance coverage is now embedded in the fast-gate Kolme version-compatibility contract lane through dry-run runtime-commit + policy checks and live HTTPS dependency-posture checks.
  - deterministic closure markers:
    - `retry_tls_smoke_contract_status=verified`
    - `retry_tls_live_https_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1`
    - `retry_tls_submit_finality_taxonomy_version=kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1`
  - retry/tls local-heavy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.
  - documentation lineage for marker parity is maintained across:
    - `docs/ci/strategy.md`
    - `docs/planning/kolme-devnet-ops.md`

### R27.29 Transport/Observability/TLS CI Smoke Convergence Closure
- Active chain: `#4293 -> #4295 -> #4299 -> (#4306, #4307)`.
- Convergence closure markers:
  - `transport_observability_tls_ci_smoke_convergence_status=verified`
  - `transport_observability_tls_reason_taxonomy_version=kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1`
  - `transport_observability_tls_ci_smoke_max_seconds=120`
  - `transport_observability_tls_local_heavy_max_seconds=900`
- Composite smoke checker coverage:
  - `python3 scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/transport-observability-tls-ci-smoke-convergence-report.json`
  - `bash scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh`
- Local-heavy boundaries remain explicit and outside ci-fast-gate:
  - `KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900`
  - `KAMN_LOCAL_OBSERVABILITY_SCRAPE_OPT_IN=1 bash scripts/runtime/validate_local_observability_scrape_live.sh --mode run --output-json /tmp/local-observability-scrape-live-summary.json`
  - `KAMN_LIVE_TRANSPORT_FAULT_MATRIX_OPT_IN=1 bash scripts/runtime/validate_live_transport_fault_matrix_live.sh --mode run --ci-fast-gate FAIL --output-json /tmp/live-transport-fault-matrix-live-summary.json`

### R27.30 Partition-Finality CI Smoke Governance Closure
- Active chain: `#4250 -> #4254 -> (#4261, #4262)`.
- Convergence closure markers:
  - `partition_finality_ci_smoke_convergence_status=verified`
  - `partition_finality_ci_smoke_reason_taxonomy_version=kamn.ci.partition-finality-ci-smoke-convergence-reason-taxonomy.v1`
  - `partition_finality_ci_smoke_max_seconds=120`
  - `partition_finality_local_heavy_max_seconds=900`
- Composite smoke checker coverage:
  - `python3 scripts/ci/check_partition_finality_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/partition-finality-ci-smoke-convergence-report.json`
  - `bash scripts/ci/test_check_partition_finality_ci_smoke_convergence.sh`
- Local-heavy partition-finality boundaries remain explicit and outside ci-fast-gate:
  - `bash scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh --mode run --lane-profile deep --ci-fast-gate FAIL --output-json /tmp/libp2p-convergence-process-isolated-live-deep-summary.json`

### R27.27 Websocket Session CI Smoke Governance Closure
- Active chain: `#4265 -> #4269 -> (#4276, #4277)`.
- Convergence closure markers:
  - `websocket_session_ci_smoke_convergence_status=verified`
  - `websocket_session_ci_smoke_reason_taxonomy_version=kamn.ci.websocket-session-ci-smoke-convergence-reason-taxonomy.v1`
  - `websocket_session_ci_smoke_max_seconds=120`
  - `websocket_session_local_heavy_max_seconds=900`
- Composite smoke checker coverage:
  - `python3 scripts/ci/check_websocket_session_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/websocket-session-ci-smoke-convergence-report.json`
  - `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh`
- Local-heavy websocket session boundary remains explicit and outside ci-fast-gate:
  - `bash scripts/runtime/validate_service_api_websocket_live_contract_lane.sh --output-json /tmp/service-api-websocket-live-contract-lane-report.json`

### R27.28 Drift/Failover CI Smoke Governance Closure
- Active chain: `#4278 -> #4280 -> #4284 -> (#4291, #4292)`.
- Convergence closure markers:
  - `failover_drift_ci_smoke_convergence_status=verified`
  - `failover_drift_ci_smoke_reason_taxonomy_version=kamn.ci.failover-drift-ci-smoke-convergence-reason-taxonomy.v1`
  - `failover_drift_ci_smoke_max_seconds=120`
  - `failover_drift_local_heavy_max_seconds=900`
- Composite smoke checker coverage:
  - `python3 scripts/ci/check_failover_drift_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/failover-drift-ci-smoke-convergence-report.json`
  - `bash scripts/ci/test_check_failover_drift_ci_smoke_convergence.sh`
- Local-heavy failover boundaries remain explicit and outside ci-fast-gate:
  - `KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled bash scripts/runtime/run_failover_sync_drill_deep_lane.sh --output-json /tmp/failover-sync-deep-report.json`

## Cost and CI Policy Boundaries
- Heavy local integration run-mode lanes remain excluded from `ci-fast-gate` and fast `ci-tools` blocks.
- Deterministic dry-run contract checks remain in PR path.
- Local-heavy/live-node validations remain opt-in and bounded for cost control.
- Reference policy doc: `docs/ci/strategy.md`.

### Script-Surface Trend Governance Refresh (Task #3740)
- Combined shell-surface baseline refreshed to current post-migration snapshot:
  - `fixtures/ci/combined_shell_surface_trend_baseline.json` (`script_count=408`, `shell_line_total=33972`, `rust_line_total=115293`, `shell_to_rust_ratio=0.294658`).
- Script-surface budget envelope refreshed for current dispatcher-migration state:
  - `.ci/script-surface-budget.env` (`SHELL_LINE_TOTAL_MAX=36000`).
  - `.ci/script-surface-baseline.env` (`SCRIPT_COUNT_BASELINE=408`, `SHELL_LINE_TOTAL_BASELINE=33972`).
- Combined trend gate remains fail-closed for future drift using:
  - `scripts/ci/check_combined_shell_surface_trend_policy.sh`
  - `fixtures/ci/combined_shell_surface_trend_thresholds.json`

### R27.32 Wrapper Migration Tranche (Issue #4341)
- Selected task deep-lane wrappers were migrated to manifest-dispatch execution:
  - `scripts/task/run_task_operation_snapshot_deep_lane.sh`
  - `scripts/task/run_federated_delegation_settlement_deep_lane.sh`
- New deep-lane implementation modules and manifests anchor the migration:
  - `scripts/task/run_task_operation_snapshot_deep_lane_impl.sh`
  - `scripts/task/run_federated_delegation_settlement_deep_lane_impl.sh`
  - `scripts/framework/manifests/task_task_operation_snapshot_deep_lane.json`
  - `scripts/framework/manifests/task_federated_delegation_settlement_deep_lane.json`
- Non-Kolme dispatcher unknown-wrapper fallback is now deterministic:
  - `fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`
  - `fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped`
  - `fallback_reason_code=dispatcher_unknown_wrapper`

## Validation Commands (Low-Cost Truth Guard)
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `bash scripts/ci/test_kolme_live_integration_architecture_contract.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`

## Historical Baseline (Superseded)
The earlier 2026-02-14 draft language that described ingress/storage/runtime composition as unresolved scaffolding is retained only as historical context and is superseded by the issue-chain status above.

Use #3333 as the single source of truth for roadmap status transitions.
