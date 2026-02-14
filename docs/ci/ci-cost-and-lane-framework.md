# CI Cost and Lane Framework

## Purpose
Keep merge-critical CI fast and cost-bounded while preventing silent growth in shell-lane maintenance surface.

## Budget Sources
- Runtime/runner budgets: `.ci/ci-budget.env`
- Fast-gate delta baseline/thresholds: `.ci/fast-gate-budget-delta.env`
- Script-surface budgets: `.ci/script-surface-budget.env`
- Script-surface delta baseline: `.ci/script-surface-baseline.env`
- Test-harness soft budget: `.ci/test-harness-loc-soft-budget.env`
- Test-harness baseline: `.ci/test-harness-loc-baseline.env`
- Test-harness trend thresholds: `.ci/test-harness-loc-trend-thresholds.env`
- Kolme command-surface soft budget: `.ci/kolme-command-surface-soft-budget.env`
- Kolme command-surface baseline: `.ci/kolme-command-surface-baseline.env`
- Kolme command-surface trend thresholds: `.ci/kolme-command-surface-trend-thresholds.env`

## Enforcers
- Runtime budget gate: `scripts/ci/evaluate_budget.sh`
- Fast-gate delta report generator: `scripts/ci/generate_fast_gate_budget_delta_report.sh`
- Fast-gate delta threshold gate: `scripts/ci/check_fast_gate_budget_delta_threshold.sh`
- Fast-gate delta contract lane: `scripts/ci/run_fast_gate_budget_delta_contract_lane.sh`
- Script surface gate: `scripts/ci/check_script_duplication_budget.sh`
- Test-harness soft-budget contract lane: `scripts/ci/run_test_harness_loc_soft_budget_contract_lane.sh`
- Kolme harness+command-surface trend report: `scripts/ci/generate_kolme_test_harness_loc_trend_report.sh`
- Kolme harness+command-surface budget contract lane: `scripts/ci/run_kolme_test_harness_loc_soft_budget_contract_lane.sh`
- CI helper regression suite: `scripts/ci/test_ci_tools.sh`

## Dispatcher Migration Log
- `#2854` migrates `scripts/signer/run_signer_provider_deep_lane.sh` to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/signer/run_signer_provider_deep_lane.sh`
  - implementation: `scripts/signer/run_signer_provider_deep_lane_impl.sh`
  - manifest: `scripts/framework/manifests/signer_signer_provider_deep_lane.json`
- `#2856` migrates `scripts/signer/run_signer_incident_recovery_deep_lane.sh` to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/signer/run_signer_incident_recovery_deep_lane.sh`
  - implementation: `scripts/signer/run_signer_incident_recovery_deep_lane_impl.sh`
  - manifest: `scripts/framework/manifests/signer_signer_incident_recovery_deep_lane.json`
- `#2858` migrates dashboard run-lane wrappers to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/dashboard/run_backend_session_auth_freshness_lane.sh`
  - implementation: `scripts/dashboard/run_backend_session_auth_freshness_lane_impl.sh`
  - manifest: `scripts/framework/manifests/dashboard_backend_session_auth_freshness_lane.json`
  - wrapper: `scripts/dashboard/run_dashboard_stale_error_budget_lane.sh`
  - implementation: `scripts/dashboard/run_dashboard_stale_error_budget_lane_impl.sh`
  - manifest: `scripts/framework/manifests/dashboard_stale_error_budget_lane.json`
- `#2860` migrates classification/redaction run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/compliance/run_classification_redaction_lane.sh`
  - implementation: `scripts/compliance/run_classification_redaction_lane_impl.sh`
  - manifest: `scripts/framework/manifests/compliance_classification_redaction_lane.json`
- `#2862` migrates deployment slo/rollback run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/deploy/run_deployment_slo_rollback_lane.sh`
  - implementation: `scripts/deploy/run_deployment_slo_rollback_lane_impl.sh`
  - manifest: `scripts/framework/manifests/deploy_deployment_slo_rollback_lane.json`
- `#2864` migrates frontend shell determinism run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/frontend/run_dashboard_shell_determinism_matrix_lane.sh`
  - implementation: `scripts/frontend/run_dashboard_shell_determinism_matrix_lane_impl.sh`
  - manifest: `scripts/framework/manifests/frontend_dashboard_shell_determinism_matrix_lane.json`
- `#2866` migrates governance lifecycle/rollback run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/governance/run_governance_lifecycle_rollback_lane.sh`
  - implementation: `scripts/governance/run_governance_lifecycle_rollback_lane_impl.sh`
  - manifest: `scripts/framework/manifests/governance_lifecycle_rollback_lane.json`
- `#2868` migrates governance quorum-attestation replay guard run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/governance/run_quorum_attestation_replay_guard_lane.sh`
  - implementation: `scripts/governance/run_quorum_attestation_replay_guard_lane_impl.sh`
  - manifest: `scripts/framework/manifests/governance_quorum_attestation_replay_guard_lane.json`
- `#2870` migrates sdk live-transport smoke parity run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/sdk/run_live_transport_smoke_parity_lane.sh`
  - implementation: `scripts/sdk/run_live_transport_smoke_parity_lane_impl.sh`
  - manifest: `scripts/framework/manifests/sdk_live_transport_smoke_parity_lane.json`
- `#2872` migrates sdk live-transport replay/tamper fast-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh`
  - implementation: `scripts/sdk/run_live_transport_replay_tamper_fast_lane_impl.sh`
  - manifest: `scripts/framework/manifests/sdk_live_transport_replay_tamper_fast_lane.json`
- `#2874` migrates sdk live-transport replay/tamper deep-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/sdk/run_live_transport_replay_tamper_deep_lane.sh`
  - implementation: `scripts/sdk/run_live_transport_replay_tamper_deep_lane_impl.sh`
  - manifest: `scripts/framework/manifests/sdk_live_transport_replay_tamper_deep_lane.json`
- `#2876` migrates runtime live-network smoke run-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/runtime/run_live_network_smoke_lane.sh`
  - implementation: `scripts/runtime/run_live_network_smoke_lane_impl.sh`
  - manifest: `scripts/framework/manifests/runtime_live_network_smoke_lane.json`
- `#2878` migrates runtime live-network pilot deep-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/runtime/run_live_network_pilot_deep_lane.sh`
  - implementation: `scripts/runtime/run_live_network_pilot_deep_lane_impl.sh`
  - manifest: `scripts/framework/manifests/runtime_live_network_pilot_deep_lane.json`
- `#2880` migrates runtime partition/reconnect smoke-lane wrapper to shared non-Kolme dispatcher + manifest wiring:
  - wrapper: `scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh`
  - implementation: `scripts/runtime/run_live_network_partition_reconnect_smoke_lane_impl.sh`
  - manifest: `scripts/framework/manifests/runtime_live_network_partition_reconnect_smoke_lane.json`
- Contract coverage for this migration slice:
  - `scripts/signer/test_run_signer_provider_deep_lane.sh`
  - `scripts/signer/test_run_signer_incident_recovery_deep_lane.sh`
  - `scripts/signer/test_run_signer_emulator_contract_lane.sh`
  - `scripts/dashboard/test_run_backend_session_auth_freshness_lane.sh`
  - `scripts/dashboard/test_run_dashboard_stale_error_budget_lane.sh`
  - `scripts/compliance/test_run_classification_redaction_lane.sh`
  - `scripts/deploy/test_run_deployment_slo_rollback_lane.sh`
  - `scripts/frontend/test_run_dashboard_shell_determinism_matrix_lane.sh`
  - `scripts/governance/test_run_governance_lifecycle_rollback_lane.sh`
  - `scripts/governance/test_run_quorum_attestation_replay_guard_lane.sh`
  - `scripts/sdk/test_run_live_transport_smoke_parity_lane.sh`
  - `scripts/sdk/test_run_live_transport_replay_tamper_fast_lane.sh`
  - `scripts/sdk/test_run_live_transport_replay_tamper_deep_lane.sh`
  - `scripts/runtime/test_run_live_network_smoke_lane.sh`
  - `scripts/runtime/test_run_live_network_pilot_deep_lane.sh`
  - `scripts/runtime/test_run_live_network_partition_reconnect_smoke_lane.sh`

## Fast-Gate Delta Policy
`scripts/ci/generate_fast_gate_budget_delta_report.sh` compares current fast-gate telemetry against a versioned baseline and emits:

- baseline runtime/cost (`elapsed_seconds`, `runner_minutes`)
- current runtime/cost
- absolute and percentage variance

`scripts/ci/check_fast_gate_budget_delta_threshold.sh` fails closed when positive variance exceeds configured limits without a valid waiver.

Fast-gate threshold metadata contract:

- `FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON` must be present in `.ci/fast-gate-budget-delta.env`.
- `FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS` must be present in `.ci/fast-gate-budget-delta.env`.
- stale threshold metadata is fail-closed via `reason_codes=fast_gate_delta_threshold_file_stale`.
- corrupt threshold metadata is fail-closed via `reason_codes=fast_gate_delta_threshold_file_corrupt`.
- contract lane command: `bash scripts/ci/run_fast_gate_budget_delta_contract_lane.sh --output-json /tmp/fast-gate-budget-delta-contract-report.json`
- refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata

## Staging Soak Telemetry Policy
Staging soak/rehearsal evidence for rollout readiness is generated and checked with:

- `scripts/deploy/generate_staging_rehearsal_bundle.sh`
- `scripts/deploy/check_staging_rehearsal_policy.sh`
- `scripts/deploy/run_staging_rehearsal_contract_lane.sh`
- `scripts/deploy/run_staging_rehearsal_deep_lane.sh` (manual/deep validation path)

Deterministic runtime telemetry threshold fields in `kamn.release.staging-rehearsal.v1`:

- `runtime_submit_success_rate_bps` with threshold `min_runtime_submit_success_rate_bps`
- `runtime_finality_timeout_count` with threshold `max_runtime_finality_timeout_count`
- `signer_profile_drift_events` with threshold `max_signer_profile_drift_events`

Fail-closed reason codes for threshold overruns:

- `runtime_submit_success_rate_below_threshold`
- `runtime_finality_timeout_threshold_exceeded`
- `signer_profile_drift_threshold_exceeded`

Escalation path when telemetry thresholds fail:

1. Open/update a tracking issue with threshold overrun evidence and candidate mitigation.
2. Re-run `run_staging_rehearsal_contract_lane.sh` after mitigation.
3. Use `run_staging_rehearsal_deep_lane.sh` for manual drift rehearsal before rollout approval.

## Deployment Preflight Runtime-Signer Drift Telemetry Policy
Kolme deployment admission preflight emits deterministic runtime signer drift telemetry in
`kamn.kolme.local-live-deployment-preflight-summary.v1`:

- `runtime_signer_drift_telemetry_schema_version=kamn.kolme.runtime-signer-drift-telemetry.v1`
- `runtime_signer_drift_telemetry` bundle with rotation and quorum drift fields
- `contracts.runtime_signer_drift_telemetry_required=true`
- `runtime_signer_drift_thresholds_schema_version=kamn.kolme.runtime-signer-drift-thresholds.v1`
- `runtime_signer_drift_thresholds_bundle`
- `runtime_signer_drift_admission_matrix_decision=GO|WARN|NO-GO`
- `runtime_signer_drift_admission_matrix_class=healthy|warning-edge|hard-fail`
- `signer_key_source=managed-external`
- `contracts.required_signer_key_source_for_production=managed-external`
- `contracts.signer_key_source_production_requirement_reason_code=signer_key_source_production_managed_external_required`

Fail-closed reason codes for missing/malformed telemetry:

- `runtime_signer_drift_telemetry_missing`
- `runtime_signer_drift_telemetry_schema_version_mismatch`
- `runtime_signer_drift_telemetry_rotation_delta_invalid`
- `runtime_signer_drift_quorum_fail_threshold_exceeded`
- `runtime_signer_drift_rotation_fail_threshold_exceeded`
- `signer_key_source_production_managed_external_required`

Deterministic response matrix:

1. `GO` + `healthy`: continue promotion with standard artifact archival.
2. `WARN` + `warning-edge`: freeze promotion, rotate signer evidence within threshold, rerun preflight lane + policy checker, then resume only on `GO`.
3. `NO-GO` + `hard-fail`: block rollout, execute signer rollback runbook, refresh quorum/custody/provenance evidence, and require a clean rerun before unfreeze.

Coverage split for cost control:

- fast lane: `run_local_kolme_live_deployment_preflight_lane.sh` +
  `check_local_kolme_live_deployment_preflight_policy.py` (ci-fast-gate eligible)
- contract lane: `run_local_kolme_live_deployment_preflight_contract_lane.sh` for docs/policy parity
- local-heavy/deep integration remains outside fast gate and opt-in only
  - workflow boundary requires dual selector gating:
    - `steps.scope.outputs.run_kolme_local_heavy_contract_tests == 'true'`
    - `steps.scope.outputs.kolme_local_heavy_selector_opt_in == 'true'`
  - fail-closed policy reason for missing dual gate: `local_heavy_lane_not_opt_in_selector_gated`
  - fail-closed policy reason when local-heavy commands leak into ci-tools fast mode:
    - `local_heavy_lane_commands_in_ci_tools_fast_mode`

## Managed-Signer Backend SLO Telemetry Artifact
Managed-signer backend SLO telemetry is emitted through:

- `scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh`
- `scripts/kolme/run_managed_signer_backend_slo_telemetry_contract_lane.sh`
- `scripts/kolme/check_managed_signer_backend_slo_policy.py`
- `scripts/kolme/run_managed_signer_backend_slo_policy_contract_lane.sh`

Deterministic contract markers:

- `kamn.kolme.managed-signer-backend-slo-telemetry.v1`
- `kamn.kolme.managed-signer-backend-slo-policy-report.v1`
- `kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1`
- `signer_key_source=managed-external`
- `contracts.required_signer_key_source=managed-external`
- `managed_signer_backend_slo_within_threshold`
- `managed_signer_backend_no_action_required`
- `managed_signer_backend_timeout_rate_threshold_exceeded`
- `managed_signer_backend_unavailable_rate_threshold_exceeded`
- `managed_signer_backend_error_rate_threshold_exceeded`
- `managed_signer_backend_ci_fast_gate_failed`
- `managed_signer_backend_reduce_timeout_burst`
- `managed_signer_backend_failover_endpoint`
- `managed_signer_backend_enable_circuit_breaker`
- `managed_signer_backend_replay_ci_fast_gate`

Cost boundary:

- generation + policy + contract-lane checks are offline, bounded, and PR fast-gate friendly.
- no local-heavy selector or external metrics backend calls are required.

## Managed-Signer Startup Live Validation Contract Lane
Managed-signer startup validation for production profile admission and fail-closed reason-code drills is emitted through:

- `scripts/kolme/run_managed_signer_startup_live_validation_contract_lane.sh`
- `scripts/kolme/contracts/managed_signer_startup_live_validation_contract_lane.py`

Deterministic contract markers:

- `kamn.kolme.managed-signer-startup-live-validation-contract-report.v1`
- `deployment_preflight_passed`
- `checkpoint_failed_signer_provenance_contract`
- `checkpoint_failed_signer_profile_contract`
- `checkpoint_failed_signer_rotation_freshness_contract`
- `signer_key_source_production_managed_external_required`
- `signer_profile_mismatch`
- `signer_rotation_epoch_stale`
- `execution_scope=local-scheduled`

Cost boundary:

- lane runtime is hard-bounded by `--max-seconds`.
- lane defaults to local/scheduled execution (`ci_fast_gate_eligible=false`) to avoid PR fast-gate runtime cost.

## Script-Surface Budget Policy
`scripts/ci/check_script_duplication_budget.py` computes deterministic metrics over non-test shell command surface under `scripts/**/*.sh`:

- files named `test_*.sh` are excluded from metric totals
- symlink wrappers remain counted for `script_count`
- symlink wrappers contribute a fixed `1` line each to `shell_line_total` (prevents target-body double counting)
- symlink wrappers are excluded from `duplicate_content`

- `script_count`
- `shell_line_total`
- `duplicate_basename`
- `duplicate_content` (regular files only)

The checker also computes per-PR deltas against `.ci/script-surface-baseline.env` and emits:

- `delta_script_count`
- `delta_shell_line_total`
- `delta_duplicate_basename`
- `delta_duplicate_content`

The checker fails closed when any metric exceeds its configured threshold and emits deterministic remediation guidance.

## Waiver Rules
Temporary exceptions are allowed through `.ci/script-surface-budget-waiver.json`.
Required fields:

- `reason` (non-empty string)
- `expires_on` (`YYYY-MM-DD`)
- `allow_metrics` (non-empty string list)

Policy constraints:

- Expired waivers fail closed.
- Malformed waivers fail closed.
- Only explicitly listed metrics are waived.

Fast-gate delta overruns may be waived through `.ci/fast-gate-budget-delta-waiver.json`.
Required fields:

- `reason` (non-empty string)
- `expires_on` (`YYYY-MM-DD`)
- `allow_metrics` (non-empty string list; allowed values: `elapsed_seconds_delta_pct`, `runner_minutes_delta_pct`)

Escalation path for temporary overruns:

1. Open/update a tracking issue with root cause and expected rollback date.
2. Add waiver with explicit metric scope and short expiry.
3. Remove waiver in the follow-up PR after regression is resolved.

## Local Validation
Run these before opening a PR that modifies CI/lane surfaces:

```bash
bash scripts/ci/check_script_duplication_budget.sh
bash scripts/ci/test_check_script_duplication_budget.sh
bash scripts/ci/test_generate_fast_gate_budget_delta_report.sh
bash scripts/ci/test_check_fast_gate_budget_delta_threshold.sh
bash scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh
bash scripts/ci/test_ci_tools.sh
bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh
bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh
```

Kolme harness + command-surface trend validation:

```bash
bash scripts/ci/generate_kolme_test_harness_loc_report.sh --output-json /tmp/kolme-test-harness-loc-report.json
bash scripts/ci/check_kolme_test_harness_loc_soft_budget.sh --report-file /tmp/kolme-test-harness-loc-report.json --output-json /tmp/kolme-test-harness-loc-soft-budget-report.json
bash scripts/ci/generate_kolme_test_harness_loc_trend_report.sh --output-json /tmp/kolme-test-harness-loc-trend-report.json
bash scripts/ci/run_kolme_test_harness_loc_soft_budget_contract_lane.sh --output-json /tmp/kolme-test-harness-loc-soft-budget-contract-report.json
```

Generic test-harness soft-budget contract validation:

```bash
bash scripts/ci/generate_test_harness_loc_report.sh --output-json /tmp/test-harness-loc-report.json
bash scripts/ci/check_test_harness_loc_soft_budget.sh --report-file /tmp/test-harness-loc-report.json --budget-file .ci/test-harness-loc-soft-budget.env --baseline-file .ci/test-harness-loc-baseline.env --trend-threshold-file .ci/test-harness-loc-trend-thresholds.env --output-json /tmp/test-harness-loc-soft-budget-report.json
bash scripts/ci/run_test_harness_loc_soft_budget_contract_lane.sh --output-json /tmp/test-harness-loc-soft-budget-contract-report.json
```

Deterministic command-surface drift markers in trend artifacts:
- `command_surface_trend_status=within|warn|fail|invalid`
- `command_surface_policy_decision=GO|WARN|NO-GO`
- `command_surface_script_count_trend_warn_delta_exceeded`
- `command_surface_script_count_trend_fail_delta_exceeded`
- `command_surface_shell_line_total_trend_warn_delta_exceeded`
- `command_surface_shell_line_total_trend_fail_delta_exceeded`
- `command_surface_budget_status_fail`

Optional hard-fail mode for command-surface drift:
- `bash scripts/ci/generate_kolme_test_harness_loc_trend_report.sh --enforce-command-surface-fail --output-json /tmp/kolme-test-harness-loc-trend-report.json`

## Artifact Contract
`check_script_duplication_budget.py` supports `--output-json` and writes:

- `schema_version=kamn.ci.script-surface-budget-report.v1`
- metric values
- baseline metric values
- metric deltas
- threshold values
- violation/waiver state
- remediation guidance

This keeps cost governance machine-verifiable for CI and audits.
