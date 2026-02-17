# Release Go/No-Go Checklist and Dry-Run Workflow (Issues #172, #173)

This checklist defines deterministic release gates and auditable evidence requirements before approving a protocol or runtime upgrade.
For semantic versioning policy and compatibility rules, see `docs/foundation/versioning-compatibility-matrix.md`.

## Preflight Gates
- Migration plan reviewed and signed.
- Compatibility matrix validated.
- Deployment topology preflight passed (`scripts/deploy/preflight_topology.sh`).
- CI fast gate and deferred deep lane both green.
- Rollback runbook version pinned.
- Release candidate artifact digest verified.
- Kolme live signer custody preflight passes with no fallback private-key marker evidence (`fallback_signer_secret_present_violation` is absent), signer quorum shortfall (`signer_quorum_shortfall` is absent), and custody evidence gaps (`custody_evidence_missing` is absent).

## Production-Mode Live Provider Enforcement Gate (Issue #4371)
- Contract lane command:
  - `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- Real-node profile command:
  - `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh`
- Required deterministic rejection markers:
  - `runtime_commit_in_memory_provider_reference_detected`
  - `runtime_commit_policy_check_in_memory_provider_reference_detected`
- In-memory provider marker forbidden in production command surfaces:
  - `InMemoryKolmeRuntimeCommitClient`
- Regression policy:
  - in-memory provider acceptance drift forces `NO-GO` (`Regression: #4371`).

## Runtime Signer Key-Source/Fallback Reason Mapping Gate (Issue #4356)
- Contract lane command:
  - `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json`
- Required deterministic taxonomy markers:
  - `key_source_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-key-source-reason-taxonomy.v1`
  - `key_source_reason_codes_csv=runtime_signer_key_source_contract_version_missing,runtime_signer_key_source_contract_version_mismatch,runtime_signer_key_source_contract_version_contract_mismatch,runtime_signer_key_source_missing,runtime_signer_key_source_invalid,runtime_signer_key_source_profile_pair_disallowed,runtime_signer_key_source_contract_mismatch,runtime_commit_signer_key_source_marker_missing,runtime_commit_fallback_private_key_command_marker_detected,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation`
  - `key_source_reason_codes_value=none|<csv>`
- Required fail-closed rejection markers:
  - `runtime_signer_key_source_contract_version_missing`
  - `runtime_signer_key_source_contract_version_contract_mismatch`
  - `runtime_commit_signer_key_source_marker_missing`
  - `runtime_commit_fallback_private_key_command_marker_detected`
  - `runtime_signer_fallback_private_key_present_violation`
- Regression policy:
  - implicit/default signer key-source acceptance or fallback-key leakage forces `NO-GO` (`Regression: #4356`).

## Invariant Property/Fuzz/Concurrency Reason Mapping Gate (Issue #4401)
- Contract lane command:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Policy checker command:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`
- Required deterministic taxonomy markers:
  - `reason_taxonomy_version=kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1`
  - `reason_codes_csv=property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch`
  - `reason_codes_value=none|<csv>`
  - `final_decision=GO|NO-GO`
- Required policy evidence markers:
  - `invariant_policy_reason_taxonomy_version=kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1`
  - `invariant_policy_reason_codes_value=none|<csv>`
  - `invariant_policy_expected_reason_codes_value=none|<csv>`
  - `invariant_policy_observed_reason_codes_value=none|<csv>`
  - `invariant_policy_final_decision=GO|NO-GO`
- Regression policy:
  - invariant-lane acceptance drift or unstable taxonomy/evidence output drift forces `NO-GO` (`Regression: #4401`).

## Message Anchoring Mismatch/Tamper Gate (Issue #4419)
- Contract lane command:
  - `bash scripts/kolme/run_message_proof_anchoring_contract_lane.sh --output-json /tmp/message-proof-anchoring-contract-report.json`
- Live validation command:
  - `bash scripts/kolme/validate_message_proof_anchoring_live.sh --output-json /tmp/message-proof-anchoring-live-report.json`
- Required deterministic taxonomy markers:
  - `anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1`
  - `anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required`
  - `anchoring_gate_reason_codes_value=none|<csv>`
- Required CI/local-heavy boundary markers:
  - `ci_smoke_local_heavy_boundary_status=verified`
  - `ci_smoke_lane_cost_profile=low`
  - `local_heavy_lane_execution_mode=opt_in`
- Fail-closed drills:
  - lifecycle-state mismatch before `Broadcast` must reject with `message_proof_anchor_invalid_state`.
  - tampered actor payload for same message+nonce idempotency window must reject with `message_proof_anchor_conflicting_key`.
- Regression policy:
  - mismatch/tamper acceptance drift forces `NO-GO` (`Regression: #4419`).

## Service API Protocol/Session Reason Mapping Gate (Issue #4318)
- Validation commands:
  - `cargo test -p kamn-node unit_service_api_protocol_session_reason_projection_is_deterministic -- --exact`
  - `cargo test -p kamn-node functional_service_api_protocol_session_docs_contract_validation_passes_release_checklist -- --exact`
  - `cargo test -p kamn-node integration_service_api_protocol_session_reason_projection_and_docs_contract_flow -- --exact`
  - `cargo test -p kamn-node regression_service_api_protocol_session_ws_upgrade_reason_class_stays_stable -- --exact`
- Required deterministic taxonomy markers:
  - `service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1`
  - `service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing`
- Required protocol/session reason markers:
  - `service_api_ws_upgrade_header_missing`
  - `service_api_ws_version_header_invalid`
  - `service_api_payload_json_syntax_invalid`
  - `service_api_auth_replay_nonce_detected`
  - `service_api_protocol_session_docs_marker_missing`
- Regression policy:
  - protocol/session reason-class drift or docs marker drift forces `NO-GO` (`Regression: #4318`).

## Shutdown Signal Lifecycle Reason Mapping Gate (Issue #4331)
- Validation commands:
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_full_supervisor_stop_contract_classifier_rejects_empty_or_non_numeric_signal_tick -- --exact`
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_shutdown_policy_rejects_os_signal_hooks_for_non_daemon_modes -- --exact`
  - `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_full_os_signal_timeout_stop_markers_project_shutdown_field_parity -- --exact`
- Required deterministic taxonomy markers:
  - `shutdown_signal_reason_taxonomy_version=kamn.runtime.shutdown-signal-lifecycle-reason-taxonomy.v1`
  - `shutdown_signal_reason_codes_csv=full_supervisor_stop_invalid_shutdown_drain_status,full_supervisor_stop_invalid_shutdown_snapshot_flush_status,full_supervisor_stop_not_signaled_status_mismatch,full_supervisor_stop_not_signaled_snapshot_flush_mismatch,full_supervisor_stop_missing_signal_tick,full_supervisor_stop_missing_drain_ticks,full_supervisor_stop_missing_timeout_ticks,full_supervisor_stop_missing_ignored_signals,full_supervisor_stop_graceful_status_mismatch,full_supervisor_stop_graceful_snapshot_flush_status_mismatch,full_supervisor_stop_graceful_timeout_status_mismatch,full_supervisor_stop_graceful_timeout_snapshot_flush_status_mismatch,full_supervisor_stop_unknown_completion_reason`
  - `shutdown_signal_reason_codes_value=none|<csv>`
- Required hook-policy markers:
  - `shutdown_signal_hook_runtime_modes=daemon|full`
  - `shutdown_signal_hook_explicit_override=--daemon-shutdown-os-signals`
- Fail-closed policy:
  - malformed graceful/graceful-timeout completion reasons with empty/non-numeric signal ticks must reject with `full_supervisor_stop_missing_signal_tick`.
  - non-daemon/full runtime modes must not enable OS-signal hooks.
- Regression policy:
  - signal hook mode drift or shutdown reason-class drift forces `NO-GO` (`Regression: #4331`).

## Shutdown Drain/Checkpoint Reconciliation Gate (Issues #4332, #4333)
- Validation commands:
  - `cargo test -p kamn-node regression_full_supervisor_stop_contract_classifier_rejects_graceful_drain_timeout_mismatch -- --exact`
  - `cargo test -p kamn-node unit_shutdown_checkpoint_reconciliation_classifier_rejects_timeout_reason_mapping_drift -- --exact`
  - `cargo test -p kamn-node regression_shutdown_checkpoint_reconciliation_validator_fails_closed_with_stable_reason -- --exact`
- Required deterministic taxonomy marker:
  - `shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1`
- Required fail-closed reason markers:
  - `full_supervisor_stop_graceful_drain_timeout_contract_mismatch`
  - `shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch`
  - `shutdown_checkpoint_reconciliation_timeout_checkpoint_mismatch`
  - `shutdown_checkpoint_reconciliation_graceful_reason_code_mismatch`
  - `shutdown_checkpoint_reconciliation_graceful_checkpoint_mismatch`
  - `shutdown_checkpoint_reconciliation_not_signaled_reason_code_mismatch`
  - `shutdown_checkpoint_reconciliation_not_signaled_checkpoint_mismatch`
  - `runtime_shutdown_invariant_violation`
- Regression policy:
  - shutdown drain/checkpoint reconciliation drift acceptance forces `NO-GO` (`Regression: #4333`).

## Runtime Observability Endpoint Payload Checker Gate (Issue #4328)
- Validation commands:
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::spec_c01_observability_endpoint_contract_checker_accepts_valid_surface_payloads -- --exact`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::spec_c02_observability_endpoint_contract_checker_rejects_missing_health_reason_code_field -- --exact`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::spec_c03_observability_endpoint_contract_checker_rejects_metrics_readiness_metric_drift -- --exact`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::spec_c04_observability_endpoint_contract_checker_rejects_stream_schema_version_drift -- --exact`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::spec_c05_observability_endpoint_contract_checker_fails_closed_with_stable_reason_markers -- --exact`
- Required deterministic taxonomy markers:
  - `reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1`
  - `reason_codes_csv=runtime_observability_policy_required_field_missing,runtime_observability_policy_schema_drift`
  - `reason_codes_value=none|<csv>`
- Required fail-closed envelope markers:
  - `schema_version=kamn.runtime.observability.endpoint-fail-closed.v1`
  - `status=fail_closed`
  - `final_decision=NO-GO`
- Fail-closed policy:
  - missing required endpoint payload fields must reject with `runtime_observability_policy_required_field_missing:<surface>.<field>`.
  - schema-version drift must reject with `runtime_observability_policy_schema_drift:<surface>.schema_version`.
- Regression policy:
  - endpoint payload schema/taxonomy drift acceptance forces `NO-GO` (`Regression: #4328`).

## Panic-Replacement Reason Taxonomy and Runtime Evidence Gate (Issue #4455)
- Checker command:
  - `scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --output-json /tmp/no-production-expect-report.json`
- Required deterministic taxonomy markers:
  - `panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1`
  - `panic_replacement_reason_codes_csv=scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default`
  - `panic_replacement_reason_codes_value=none|<csv>`
  - `panic_replacement_reason_class=stable|panic_reachability|unsafe_fallback|mixed|configuration`
- Required runtime evidence markers:
  - `runtime_panic_replacement_evidence_status=verified|violation`
  - `runtime_panic_replacement_evidence_violation_count=<n>`
  - `runtime_panic_replacement_evidence_files_csv=none|<csv>`
  - `runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv`
- Fail-closed policy:
  - reachable `.expect(`, `panic!`, `unreachable!`, or unsafe env fallback defaults force `NO-GO`.
  - missing checker roots must fail closed with `scan_root_not_found`.
- Regression policy:
  - panic-replacement taxonomy drift and runtime evidence-output drift force `NO-GO` (`Regression: #4455`).

## Dependency-License Metadata/Docs Mismatch Gate (Issue #4456)
- Validation commands:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- Required fail-closed mismatch markers:
  - `license_mismatch`
  - `license_missing`
  - `manifest_invalid_toml`
  - `package_section_missing`
  - `readme_webpki_roots_reference_missing`
  - `readme_no_default_features_marker_missing`
  - `ci_strategy_no_default_features_check_missing`
- Regression policy:
  - dependency/license metadata drift or docs mismatch acceptance forces `NO-GO`
    (`Regression: #4456`).

## TLS Dependency-Posture Gate (Issues #4480, #4481)
- Checker command:
  - `bash scripts/ci/check_kamn_core_live_https_dependency_posture.sh --output-json /tmp/kamn-core-live-https-dependency-posture-report.json`
- Required deterministic taxonomy markers:
  - `reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1`
  - `reason_codes_csv=none|<csv>`
  - `reason_codes_value=none|<csv>`
- Example fail-closed reasons:
  - `rustls_pemfile_dependency_optional_flag_mismatch`
  - `webpki_roots_dependency_missing`
  - `webpki_roots_feature_mapping_missing`
  - `readme_adr_link_missing`
  - `ci_strategy_live_https_feature_check_missing`
  - `ci_strategy_no_default_features_check_missing`
- Regression policy:
  - dependency-posture drift or unstable reason outputs force `NO-GO` (`Regression: #4480`, `Regression: #4481`, `Regression: #4107`).

## TLS Evidence Completeness/Freshness Convergence Gate (Issue #4477)
- TLS evidence generation command:
  - `bash scripts/ci/check_kamn_core_live_https_dependency_posture.sh --output-json /tmp/kamn-core-live-https-dependency-posture-report.json`
- Go/no-go bundle command:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-tls-evidence.json --release-candidate v1.0.0-rc.8 --schema-target-version 1.0.0 --runtime-image-digest sha256:tls-go --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2 --tls-evidence-report-file /tmp/kamn-core-live-https-dependency-posture-report.json --tls-evidence-max-age-seconds 1800`
- Go/no-go policy checker command:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-tls-evidence.json`
- Required deterministic TLS evidence gate markers:
  - generator and policy-checker command output must both project this marker set.
  - `tls_evidence_reason_taxonomy_version=kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1`
  - `tls_evidence_reason_codes_csv=gonogo_tls_evidence_file_missing,gonogo_tls_evidence_invalid_json,gonogo_tls_evidence_schema_mismatch,gonogo_tls_evidence_status_not_pass,gonogo_tls_evidence_reason_taxonomy_version_mismatch,gonogo_tls_evidence_freshness_window_exceeded`
  - `tls_evidence_reason_codes_value=none|<csv>`
  - `tls_evidence_gate_final_decision=GO|NO-GO`
- Fail-closed drills:
  - missing TLS evidence report must reject with `gonogo_tls_evidence_file_missing`.
  - stale TLS evidence report beyond max age must reject with `gonogo_tls_evidence_freshness_window_exceeded`.
  - invalid TLS evidence JSON must reject with `gonogo_tls_evidence_invalid_json`.
  - tampered TLS evidence gate payload must reject with `tls evidence gate convergence mismatch`.
- Regression policy:
  - TLS evidence completeness/freshness drift forces `NO-GO` (`Regression: #4477`, `Regression: #4298`).

## Audit-Trail Integrity/Tamper Convergence Gate (Issue #4466)
- Audit policy report command:
  - `bash scripts/runtime/check_sqlite_crash_recovery_live_policy.sh --report-file /tmp/sqlite-crash-recovery-live-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/sqlite-crash-recovery-live-policy-report.json`
- Go/no-go bundle command:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-audit-integrity.json --release-candidate v1.0.0-rc.11 --schema-target-version 1.0.0 --runtime-image-digest sha256:audit-integrity-go --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2 --audit-integrity-report-file /tmp/sqlite-crash-recovery-live-policy-report.json --audit-integrity-max-age-seconds 1800`
- Go/no-go policy checker command:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-audit-integrity.json`
- Required deterministic audit-integrity gate markers:
  - `audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1`
  - `audit_integrity_reason_codes_csv=gonogo_audit_integrity_file_missing,gonogo_audit_integrity_invalid_json,gonogo_audit_integrity_schema_mismatch,gonogo_audit_integrity_status_not_ok,gonogo_audit_integrity_final_decision_not_go,gonogo_audit_integrity_policy_status_not_verified,gonogo_audit_integrity_reason_taxonomy_version_mismatch,gonogo_audit_integrity_reason_codes_csv_mismatch,gonogo_audit_integrity_freshness_window_exceeded`
  - `audit_integrity_reason_codes_value=none|<csv>`
  - `audit_integrity_gate_final_decision=GO|NO-GO`
- Fail-closed drills:
  - missing audit-integrity policy report must reject with `gonogo_audit_integrity_file_missing`.
  - unstable source taxonomy/reason-code markers must reject with deterministic audit-integrity mismatch reason codes.
  - tampered audit-integrity gate payload must reject with `audit integrity gate convergence mismatch`.
- Regression policy:
  - audit-trail integrity evidence drift and tamper acceptance force `NO-GO` (`Regression: #4466`).

## SLO Threshold/Policy Gate Convergence Gate (Issue #4468)
- SLO policy report command:
  - `bash scripts/deploy/check_deployment_slo_rollback_policy.sh --report-file /tmp/deployment-slo-rollback-report.json`
- Go/no-go bundle command:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-slo-policy.json --release-candidate v1.0.0-rc.13 --schema-target-version 1.0.0 --runtime-image-digest sha256:slo-policy-go --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2 --slo-policy-report-file /tmp/deployment-slo-rollback-report.json --slo-policy-max-age-seconds 1800`
- Go/no-go policy checker command:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-slo-policy.json`
- Required deterministic SLO policy gate markers:
  - `slo_policy_reason_taxonomy_version=kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1`
  - `slo_policy_reason_codes_csv=gonogo_slo_policy_file_missing,gonogo_slo_policy_invalid_json,gonogo_slo_policy_schema_mismatch,gonogo_slo_policy_status_not_pass,gonogo_slo_policy_final_decision_not_go,gonogo_slo_policy_reason_key_mismatch,gonogo_slo_policy_reason_codes_not_empty,gonogo_slo_policy_freshness_window_exceeded`
  - `slo_policy_reason_codes_value=none|<csv>`
  - `slo_policy_gate_final_decision=GO|NO-GO`
- Fail-closed drills:
  - threshold drift in reason-key mapping must reject with `gonogo_slo_policy_reason_key_mismatch`.
  - non-empty source reason codes on GO path must reject with `gonogo_slo_policy_reason_codes_not_empty`.
  - tampered SLO policy gate payload must reject with `slo policy gate convergence mismatch`.
- Regression policy:
  - threshold drift and SLO gate mismatch acceptance force `NO-GO` (`Regression: #4468`).

## Unified API-Observability Payload Taxonomy Gate (Issue #4507)
- Validation command:
  - `bash scripts/runtime/validate_unified_api_observability_local_heavy_live.sh --mode dry-run --output-json /tmp/unified-api-observability-local-heavy-summary.json`
- Policy checker command:
  - `bash scripts/runtime/check_unified_api_observability_local_heavy_live_policy.sh --report-file /tmp/unified-api-observability-local-heavy-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/unified-api-observability-local-heavy-policy.json`
- Required deterministic payload markers:
  - `reason_taxonomy_version=kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1`
  - `reason_codes_csv=ci_fast_gate_failed,unified_api_observability_local_heavy_policy_artifact_paths_invalid,unified_api_observability_local_heavy_policy_ci_fast_gate_mismatch,unified_api_observability_local_heavy_policy_command_budget_exceeded,unified_api_observability_local_heavy_policy_command_count_invalid,unified_api_observability_local_heavy_policy_command_max_seconds_invalid,unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_schema_mismatch,unified_api_observability_local_heavy_policy_compatibility_policy_status_mismatch,unified_api_observability_local_heavy_policy_compatibility_report_schema_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_count_mismatch,unified_api_observability_local_heavy_policy_dry_run_command_status_mismatch,unified_api_observability_local_heavy_policy_dry_run_eligibility_mismatch,unified_api_observability_local_heavy_policy_dry_run_reason_code_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_iterations_executed_mismatch,unified_api_observability_local_heavy_policy_dry_run_soak_status_mismatch,unified_api_observability_local_heavy_policy_elapsed_seconds_invalid,unified_api_observability_local_heavy_policy_fast_gate_exclusion_reason_mismatch,unified_api_observability_local_heavy_policy_fast_gate_exclusion_status_mismatch,unified_api_observability_local_heavy_policy_final_decision_invalid,unified_api_observability_local_heavy_policy_final_decision_mismatch,unified_api_observability_local_heavy_policy_lane_mode_invalid,unified_api_observability_local_heavy_policy_max_seconds_invalid,unified_api_observability_local_heavy_policy_observability_policy_schema_mismatch,unified_api_observability_local_heavy_policy_observability_policy_status_mismatch,unified_api_observability_local_heavy_policy_observability_report_schema_mismatch,unified_api_observability_local_heavy_policy_observability_soak_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_count_mismatch,unified_api_observability_local_heavy_policy_run_mode_command_status_mismatch,unified_api_observability_local_heavy_policy_run_mode_exclusion_mismatch,unified_api_observability_local_heavy_policy_run_mode_reason_code_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_mismatch,unified_api_observability_local_heavy_policy_run_mode_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_run_mode_soak_status_mismatch,unified_api_observability_local_heavy_policy_runtime_budget_exceeded,unified_api_observability_local_heavy_policy_runtime_budget_status_mismatch,unified_api_observability_local_heavy_policy_schema_mismatch,unified_api_observability_local_heavy_policy_soak_iterations_executed_invalid,unified_api_observability_local_heavy_policy_soak_iterations_requested_invalid,unified_api_observability_local_heavy_policy_status_mismatch`
  - `reason_codes_value=none|<csv>`
- Fail-closed drill requirement:
  - tampered compatibility matrix status must reject with `unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch`.

## Transport Retry-Reconnect Failure Taxonomy Gate (Issue #4508)
- Validation command:
  - `bash scripts/runtime/validate_live_transport_fault_matrix_live.sh --mode dry-run --output-json /tmp/live-transport-fault-matrix-live-summary.json`
- Policy checker command:
  - `bash scripts/runtime/check_live_transport_fault_matrix_live_policy.sh --report-file /tmp/live-transport-fault-matrix-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/live-transport-fault-matrix-live-policy.json`
- Required deterministic payload markers:
  - `reason_taxonomy_version=kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1`
  - `reason_codes_csv=ci_fast_gate_failed,live_transport_fault_matrix_policy_command_count_invalid,live_transport_fault_matrix_policy_command_count_mismatch,live_transport_fault_matrix_policy_elapsed_seconds_invalid,live_transport_fault_matrix_policy_execution_reason_code_mismatch,live_transport_fault_matrix_policy_final_decision_invalid,live_transport_fault_matrix_policy_final_decision_mismatch,live_transport_fault_matrix_policy_lane_mode_invalid,live_transport_fault_matrix_policy_marker_missing,live_transport_fault_matrix_policy_peer_adapter_multi_process_validation_local_heavy_status_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_budget_exhausted_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch,live_transport_fault_matrix_policy_reason_codes_classification_mismatch,live_transport_fault_matrix_policy_reason_codes_invalid,live_transport_fault_matrix_policy_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_runtime_transport_mode_mismatch,live_transport_fault_matrix_policy_schema_mismatch,live_transport_fault_matrix_policy_status_invalid`
  - `reason_codes_value=none|<csv>`
- Fail-closed drill requirements:
  - partition/rejoin tamper must reject with `live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status`.
  - unstable reason classification tamper must reject with `live_transport_fault_matrix_policy_reason_codes_classification_mismatch`.

## Persisted Block Commit Mismatch/Tamper Gate (Issue #4321)
- Replay tamper matrix selector:
  - `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix`
- Required deterministic payload markers:
  - `block_commit_persistence_reason_taxonomy_version=kamn.runtime.block-commit-persistence-reason-taxonomy.v1`
  - `block_commit_persistence_reason_codes_csv=canonical_replay_payload_digest_mismatch,canonical_replay_checkpoint_missing,canonical_replay_block_height_mismatch,canonical_replay_transaction_ids_mismatch`
  - `block_commit_persistence_tamper_detection_status=verified`
- Fail-closed drift policy:
  - persisted digest/checkpoint/finality mismatch acceptance drift or tampered commit artifact acceptance forces `NO-GO` (`Regression: #4321`).

## Persistence Evidence Tamper/Freshness Gate (Issue #4389)
- Validation command:
  - `bash scripts/runtime/validate_persistence_adapters_live.sh --output-json /tmp/persistence-adapters-live-validation-report.json`
- Required deterministic payload markers:
  - `persistence_gate_reason_taxonomy_version=kamn.runtime.persistence-gate-reason-taxonomy.v1`
  - `persistence_gate_reason_codes_csv=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation`
  - `persistence_tamper_freshness_drift_fail_closed_status=verified`
  - `persistence_evidence_completeness_status=verified`
  - `persistence_ci_smoke_local_heavy_boundary_status=verified`
  - `persistence_ci_smoke_lane_cost_profile=low`
  - `persistence_local_heavy_execution_mode=opt_in`
- Fail-closed drift policy:
  - tampered marker acceptance, stale evidence acceptance, or incomplete persistence evidence acceptance forces `NO-GO` (`Regression: #4389`).

## Kolme Signer Custody Gate (Issue #2240)
- Deployment preflight lane command:
  - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Deployment preflight run-mode command:
  - `printf '%s\n' "custody-attestation=ops-primary:epoch-1" > /tmp/kolme-live-signer-custody.json`
  - `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=1111111111111111111111111111111111111111111111111111111111111111 bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode run --runtime-mode kolme-live --signer-profile ops-primary --required-approvals 2 --received-approvals 2 --custody-evidence-file /tmp/kolme-live-signer-custody.json --max-seconds 12 --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
- Required signer-custody markers:
  - `fallback_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`
  - `fallback_signer_secret_present=false`
  - `runtime_signer_drift_telemetry_schema_version=kamn.kolme.runtime-signer-drift-telemetry.v1`
  - `runtime_signer_drift_telemetry`
  - `contracts.fallback_private_key_path_allowed=false`
  - `contracts.runtime_signer_drift_telemetry_required=true`
  - `required_approvals=2`
  - `received_approvals=2`
  - `contracts.approval_quorum_required=2`
  - `contracts.custody_evidence_required=true`
- Fail-closed policy reason:
  - `fallback_signer_secret_present_violation`
  - `signer_quorum_shortfall`
  - `custody_evidence_missing`
  - `custody_evidence_sha256_invalid`
  - `runtime_signer_drift_telemetry_missing`
  - `runtime_signer_drift_telemetry_schema_version_mismatch`
  - `runtime_signer_drift_telemetry_rotation_delta_invalid`
  - `runtime_signer_drift_quorum_fail_threshold_exceeded`
  - `runtime_signer_drift_rotation_fail_threshold_exceeded`
- Admission policy decision matrix markers:
  - `runtime_signer_drift_admission_matrix_decision=GO|WARN|NO-GO`
  - `runtime_signer_drift_admission_matrix_class=healthy|warning-edge|hard-fail`
  - `runtime_signer_drift_admission_matrix_reason_codes`
  - `runtime_signer_drift_thresholds_schema_version=kamn.kolme.runtime-signer-drift-thresholds.v1`
  - `runtime_signer_drift_thresholds_bundle`

## Live Run-Mode Rehearsal Lineage Gate (Issue #3245)
Run-mode promotion requires deterministic local live-node rehearsal lineage evidence before GO decisions are accepted.

- Dry-run rehearsal lane command:
  - `bash scripts/kolme/run_local_live_node_validation_bundle_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json`
- Run-mode rehearsal lane command:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_live_node_validation_bundle_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --rollback-evidence-file /tmp/kolme-local-fork-process-lifecycle-rollback-evidence.json --recovery-evidence-file /tmp/kolme-local-fork-process-lifecycle-recovery-evidence.json --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json`
- Policy checker command:
  - `python3 scripts/kolme/check_local_live_node_validation_bundle_policy.py --report-file /tmp/kolme-local-live-node-validation-bundle-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code live_node_validation_bundle_passed --output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
- Contract lane command:
  - `bash scripts/kolme/run_local_live_node_validation_bundle_contract_lane.sh --output-json /tmp/kolme-local-live-node-validation-bundle-summary.json --policy-output-json /tmp/kolme-local-live-node-validation-bundle-policy.json`
- Required lineage markers:
  - `rollback_evidence_file`
  - `recovery_evidence_file`
  - `contracts.live_run_rehearsal_lineage_required=true`
  - `contracts.rollback_recovery_artifact_lineage_required=true`
- Fail-closed policy reasons:
  - `run_mode_check_status_mismatch:*`
  - `run_mode_check_reason_code_mismatch:*`
  - `live_run_rehearsal_lineage_required_contract_mismatch`
  - `rollback_evidence_file_missing`
  - `recovery_evidence_file_missing`
- Regression policy:
  - run-mode lineage marker drift or tampered check statuses force `NO-GO` (`Regression: #3245`).

## Deterministic Dry-Run Workflow
1. Create release candidate tag.
2. Rehearse migration on staging snapshot.
3. Execute bounded smoke and invariant suites.
4. Capture and sign dry-run evidence bundle.
5. Validate rollback precheck against last known-good snapshot.

## Go/No-Go Evidence Template
- Release candidate:
- Schema target version:
- Runtime image digest:
- Dry-run timestamp:
- CI evidence links:
- Rollback trigger status:
- Rollback precheck result: PASS
- Final decision: GO | NO-GO
- Approver signatures:

## Machine-Readable Evidence Bundle Contract (Issue #644)
Go/no-go decisions are captured as machine-readable JSON so release policy checks are auditable and deterministic.

- Generator:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo.json --release-candidate v1.0.0-rc.1 --schema-target-version 1.0.0 --runtime-image-digest sha256:abc123 --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2`
- Policy checker:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo.json`
- Fast contract lane:
  - `bash scripts/deploy/run_gonogo_evidence_contract_lane.sh`
- Stable shell wrappers:
  - `scripts/deploy/generate_gonogo_evidence_bundle.sh`
  - `scripts/deploy/check_gonogo_evidence_policy.sh`
- Shared Python implementation:
  - `scripts/deploy/gonogo_evidence_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_gonogo_evidence_deep_lane.sh`
- Required checklist evidence markers (machine-readable bundle):
  - `ci_fast_gate`
  - `ci_deep_lane`
  - `rollback_precheck`
  - `rollback_trigger_status`
  - `approval_quorum`
  - `runtime_image_digest`
- Fail-closed policy reason:
  - missing required checklist evidence markers force `NO-GO` (`Regression: #3240`).

## Milestone Review Aggregate Lineage Gate (Issue #3247)
Milestone go/no-go review must aggregate linked preflight/live/gate artifacts into a deterministic bundle surface before final approval.

- Aggregate evidence generator:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-milestone.json --release-candidate v1.0.0-rc.5 --schema-target-version 1.0.0 --runtime-image-digest sha256:abc123 --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2 --deployment-preflight-summary-file /tmp/kolme-local-live-deployment-preflight-summary.json --deployment-preflight-policy-file /tmp/kolme-local-live-deployment-preflight-policy.json --live-node-validation-summary-file /tmp/kolme-local-live-node-validation-bundle-summary.json --live-node-validation-policy-file /tmp/kolme-local-live-node-validation-bundle-policy.json --go-no-go-gate-report-file /tmp/go-no-go-gate-report.json`
- Aggregate policy checker:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-milestone.json`
- Required aggregate marker surface:
  - `milestone_review_bundle`
  - `schema_version=kamn.release.milestone-review-bundle.v1`
  - `lineage_status=verified|fail-closed`
  - `milestone_review_go_no_go_gate_report_missing`
  - `milestone_review_live_node_validation_runtime_provider_mismatch`
  - `milestone_review_operator_runbook_missing`
  - `milestone_review_operator_runbook_markers_missing`
  - `contracts.linked_artifact_lineage_required=true`
  - `contracts.operator_runbook_markers_required=true`
  - `contracts.live_bundle_runtime_provider_client_required=KolmeRuntimeCommitLiveProvider`
  - `contracts.go_no_go_gate_final_decision_required=GO`
- Decision contract:
  - aggregate lineage drift or missing linked artifacts force `NO-GO` through deterministic milestone reason codes.
  - missing operator runbook file/markers force `NO-GO` through milestone-review reason taxonomy.
  - policy checker fails closed on tampered milestone lineage payloads (`milestone review bundle lineage mismatch`).

## Live Go/No-Go Evidence Convergence and Boundary Governance Gate (Issue #4434)
Live go/no-go promotion evidence must expose deterministic live-gate taxonomy markers and explicit
CI smoke/local-heavy boundary governance.

- CI smoke contract-lane command:
  - `bash scripts/deploy/run_gonogo_evidence_contract_lane.sh --max-seconds 120`
- Local-heavy deep-lane command:
  - `KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900`
- Required live-go/no-go convergence markers:
  - `live_gonogo_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1`
  - `live_gonogo_reason_codes_csv=milestone_review_operator_runbook_missing,milestone_review_operator_runbook_markers_missing,milestone_review_deployment_preflight_summary_missing,milestone_review_deployment_preflight_summary_invalid_json,milestone_review_deployment_preflight_summary_schema_mismatch,milestone_review_deployment_preflight_summary_status_mismatch,milestone_review_deployment_preflight_scope_mismatch,milestone_review_deployment_preflight_policy_missing,milestone_review_deployment_preflight_policy_invalid_json,milestone_review_deployment_preflight_policy_schema_mismatch,milestone_review_deployment_preflight_policy_final_decision_mismatch,milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch,milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch,milestone_review_live_node_validation_summary_missing,milestone_review_live_node_validation_summary_invalid_json,milestone_review_live_node_validation_summary_schema_mismatch,milestone_review_live_node_validation_summary_status_mismatch,milestone_review_live_node_validation_scope_mismatch,milestone_review_live_node_validation_runtime_provider_mismatch,milestone_review_live_node_validation_lineage_contract_mismatch,milestone_review_live_node_validation_artifact_paths_missing,milestone_review_live_node_validation_rollback_lineage_missing,milestone_review_live_node_validation_recovery_lineage_missing,milestone_review_live_node_validation_policy_missing,milestone_review_live_node_validation_policy_invalid_json,milestone_review_live_node_validation_policy_schema_mismatch,milestone_review_live_node_validation_policy_final_decision_mismatch,milestone_review_go_no_go_gate_report_missing,milestone_review_go_no_go_gate_report_invalid_json,milestone_review_go_no_go_gate_schema_mismatch,milestone_review_go_no_go_gate_status_mismatch,milestone_review_go_no_go_gate_final_decision_mismatch,milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch,milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch,milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch,milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch,milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch,milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch`
  - `deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1`
  - `deployment_safety_gate_reason_codes_csv=none|<csv>`
  - `deployment_safety_gate_reason_codes_value=none|<csv>`
- Required live boundary governance markers:
  - `live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1`
  - `live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch`
  - `live_gonogo_ci_smoke_max_seconds=120`
  - `live_gonogo_local_heavy_max_seconds=900`
- Regression policy:
  - mismatch/tamper and partial evidence acceptance drift force `NO-GO` (`Regression: #4441`).
  - CI smoke/local-heavy boundary drift forces `NO-GO` (`Regression: #4442`).

## Staging Deploy + Rollback Rehearsal Contract (Issue #658)
Staging rehearsal automation must verify deploy and rollback outcomes before release decisions are accepted.

- Rehearsal bundle generator:
  - `bash scripts/deploy/generate_staging_rehearsal_bundle.sh --output-file /tmp/staging-rehearsal.json --release-candidate v1.1.0-rc.1 --deploy-status PASS --rollback-status PASS --rollback-target-hash state-hash-expected --post-rollback-hash state-hash-expected --recovery-time-seconds 420 --max-allowed-recovery-time-seconds 900 --evidence-complete true --ci-fast-gate PASS`
- Rehearsal policy checker:
  - `bash scripts/deploy/check_staging_rehearsal_policy.sh --bundle-file /tmp/staging-rehearsal.json`
- Fast contract lane:
  - `bash scripts/deploy/run_staging_rehearsal_contract_lane.sh`
- Stable shell wrappers:
  - `scripts/deploy/generate_staging_rehearsal_bundle.sh`
  - `scripts/deploy/check_staging_rehearsal_policy.sh`
- Shared Python implementation:
  - `scripts/deploy/staging_rehearsal_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_staging_rehearsal_deep_lane.sh`
- Staged signoff artifact schema:
  - `kamn.release.staged-rehearsal-signoff.v1`
  - policy output marker: `staged_rehearsal_signoff_status=verified|fail-closed`
- Regression policy:
  - rollback target hash mismatch and incomplete rehearsal evidence force `NO-GO` (`Regression: #623`).
  - MTTR evidence drift and out-of-bound recovery time force `NO-GO` (`Regression: #2337`).
  - staged signoff artifact drift fails closed (`staged rehearsal signoff artifact mismatch`) (`Regression: #3241`).
  - bounded MTTR policy emits deterministic markers: `recovery_time_seconds`, `max_allowed_recovery_time_seconds`, `mttr_within_bound`, and reason code `mttr-threshold-exceeded`.

## Durable Guard Migration + Recovery Matrix Evidence (Issue #691)
Durable guard schema evolution and restart invariants must be proven before a release is approved.

- PR fast contract lane:
  - `bash scripts/guard/run_durable_guard_recovery_contract_lane.sh`
- Stable shell wrapper:
  - `scripts/guard/run_durable_guard_recovery_contract_lane.sh`
- Shared Python implementation:
  - `scripts/guard/durable_guard_recovery_contract_lane_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/guard/run_durable_guard_recovery_deep_lane.sh`
- Required evidence:
  - schema mismatch errors are explicit for delivery and channel policy snapshots.
  - replay/nonce and retention invariants hold after restart recovery.
  - corrupted snapshot fixtures fail closed (`Regression: #679`).
  - PR budget check passes via `performance_durable_guard_recovery_contract_lane_budget`.
  - durable bundle store contract checks pass via `durable_guard_snapshot_store` and `performance_bundle_contract_lane_budget`.
  - nightly deep matrix executes `performance_durable_guard_recovery_matrix_deep_lane`.
  - nightly deep bundle store stress executes `performance_bundle_store_deep_lane_stress`.
  - shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1242`).

## Signer Incident Recovery Contract and Deep-Lane Cadence (Issue #989)
Signer incident response readiness must remain deterministic while keeping PR fast-gate cost bounded.

- Incident recovery lane:
  - `bash scripts/signer/run_signer_incident_recovery_lane.sh --output-json /tmp/signer-incident-recovery-report.json`
- Policy checker:
  - `bash scripts/signer/check_signer_incident_recovery_policy.sh --report-file /tmp/signer-incident-recovery-report.json`
- PR fast contract lane:
  - `bash scripts/signer/run_signer_incident_recovery_contract_lane.sh --output-file /tmp/signer-incident-recovery-contract-report.json`
- Scheduled deep lane entrypoint:
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled bash scripts/signer/run_signer_incident_recovery_deep_lane.sh --output-json /tmp/signer-incident-recovery-deep-report.json`
- Required schema/reason markers:
  - `kamn.signer.incident-recovery-report.v1`
  - `kamn.signer.incident-recovery-deep-summary.v1`
  - `signer_incident_recovery_reason_codes:GO:v1`
  - `signer_incident_recovery_deep_reason_codes:GO:v1`
- Runtime/cadence controls:
  - `KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_CONTRACT_MAX_SECONDS`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_SECONDS`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_ARTIFACT_AGE_SECONDS`
- Regression policy:
  - stale deep-lane artifacts, unscheduled deep-lane execution, and incident recovery policy drift force `NO-GO` (`Regression: #989`).

## Settlement Reconciliation Evidence Contract (Issue #687)
Escrow settlement outcomes require deterministic receipt/finality evidence before release approval.

- Evidence bundle generator:
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json`
- PR fast contract lane:
  - `bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh --output-json settlement-reconciliation-report.json`
- Race matrix runner:
  - `python3 scripts/escrow/run_settlement_reconciliation_race_matrix.py --fixture fixtures/escrow_reconciliation/finality_race_cases.json --output-json settlement-reconciliation-report.json`
- Regression policy:
  - missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).
  - timeout-before-finality pending receipts and failed receipts force `NO-GO` (`Regression: #678`).
  - missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).

## SOC2 Control Evidence Contract (Issue #744)
SOC2 audit gates require deterministic control-evidence bundles and replay-safe checker outcomes before release progression.

- Stable shell wrappers:
  - `scripts/compliance/generate_soc2_control_evidence_bundle.sh`
  - `scripts/compliance/check_soc2_control_evidence_policy.sh`
- Shared Python implementation:
  - `scripts/compliance/soc2_control_contract.py`
- Evidence bundle generator:
  - `bash scripts/compliance/generate_soc2_control_evidence_bundle.sh --output-file /tmp/soc2-control-evidence.json --control-id CC6.1 --audit-period-start 2026-01-01 --audit-period-end 2026-01-31 --collector-did did:kamn:auditor-001 --evidence-uri s3://kamn-audit/soc2/cc6_1/jan-2026/evidence.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --tamper-check PASS --completeness-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_soc2_control_evidence_policy.sh --bundle-file /tmp/soc2-control-evidence.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_soc2_control_evidence_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_soc2_control_evidence_deep_lane.sh --output-json soc2-control-evidence-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_soc2_control_evidence_replay_matrix.py --fixture fixtures/compliance_soc2/control_evidence_replay_cases.json --output-json soc2-control-evidence-report.json`
- Regression policy:
  - tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`).

## DSAR Legal-Hold Evidence Contract (Issue #746)
GDPR data-subject workflows require deterministic legal-hold precedence evidence before export/erasure approvals.

- Stable shell wrappers:
  - `scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh`
  - `scripts/compliance/check_dsar_legal_hold_policy.sh`
- Shared Python implementation:
  - `scripts/compliance/dsar_legal_hold_contract.py`
- Evidence bundle generator:
  - `bash scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh --output-file /tmp/dsar-legal-hold.json --request-id dsar-erasure-001 --subject-did did:kamn:subject-001 --request-type ERASURE --legal-hold-active true --retention-expired true --evidence-complete true --approval-recorded true --tamper-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_dsar_legal_hold_policy.sh --bundle-file /tmp/dsar-legal-hold.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_dsar_legal_hold_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_dsar_legal_hold_deep_lane.sh --output-json dsar-legal-hold-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_dsar_legal_hold_matrix.py --fixture fixtures/compliance_dsar/legal_hold_precedence_cases.json --output-json dsar-legal-hold-report.json`
- Regression policy:
  - legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`).

## Federated DID Handshake Evidence Contract (Issue #752)
Federated DID trust handshakes require deterministic replay, downgrade, and quorum evidence before cross-network approval.

- Evidence bundle generator:
  - `bash scripts/did/generate_federated_did_handshake_evidence_bundle.sh --output-file /tmp/federated-did-handshake.json --handshake-id federated-go-001 --subject-did kamn:did:agent:federated-worker-1 --local-network kolme-mainnet-a --remote-network kolme-mainnet-b --resolver-cache-hit true --resolver-version resolver-v1 --signature-policy PASS --nonce-monotonic true --downgrade-detected false --partition-sequence-monotonic true --required-quorum 2 --received-quorum 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/did/check_federated_did_handshake_policy.sh --bundle-file /tmp/federated-did-handshake.json`
- PR fast contract lane:
  - `bash scripts/did/run_federated_did_handshake_contract_lane.sh`
- Stable shell wrappers:
  - `scripts/did/generate_federated_did_handshake_evidence_bundle.sh`
  - `scripts/did/check_federated_did_handshake_policy.sh`
- Shared Python implementation:
  - `scripts/did/federated_did_handshake_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/did/run_federated_did_handshake_deep_lane.sh --output-json federated-did-handshake-report.json`
- Partition replay matrix runner:
  - `python3 scripts/did/run_federated_did_handshake_matrix.py --fixture fixtures/federated_did_handshake/partition_replay_cases.json --output-json federated-did-handshake-report.json`
- Deep-lane summary policy checker:
  - `bash scripts/did/check_federated_did_handshake_deep_policy.sh --report-file federated-did-handshake-report.json`
- Deep-lane shared Python policy implementation:
  - `scripts/did/federated_did_handshake_deep_policy_contract.py`
- Deep-lane policy matrix runner:
  - `python3 scripts/did/run_federated_did_handshake_deep_policy_matrix.py --fixture fixtures/federated_did_handshake/deep_lane_policy_cases.json --output-json federated-did-handshake-deep-policy-report.json`
- Runtime trust-store handshake evaluator:
  - `cargo test -p kamn-core --test federated_did_handshake_runtime`
- Regression policy:
  - replay/downgrade attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`).
  - runtime trust-store misses and quorum shortfalls must remain fail-closed with deterministic reason codes (`Regression: #1002`).
  - stale/tampered federated handshake deep-lane summary artifacts must remain `NO-GO` (`Regression: #1003`).

## Federated Delegation Settlement Evidence Contract (Issue #754)
Cross-network task delegation requires deterministic envelope and settlement reference evidence before cross-network approvals.

- Evidence bundle generator:
  - `bash scripts/task/generate_federated_delegation_settlement_evidence_bundle.sh --output-file /tmp/federated-delegation-settlement.json --delegation-id delegation-go-001 --task-id task-go-001 --delegator-did kamn:did:agent:delegator-go-001 --delegatee-did kamn:did:agent:delegatee-go-001 --source-network kolme-mainnet-a --destination-network kolme-mainnet-b --settlement-reference-id settlement-ref-go-001 --expected-settlement-reference-id settlement-ref-go-001 --settlement-receipt-finality FINAL --nonce-monotonic true --replay-detected false --partition-sequence-monotonic true --required-attestors 2 --received-attestors 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/task/check_federated_delegation_settlement_policy.sh --bundle-file /tmp/federated-delegation-settlement.json`
- PR fast contract lane:
  - `bash scripts/task/run_federated_delegation_settlement_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json federated-delegation-settlement-report.json`
- Partition replay matrix runner:
  - `python3 scripts/task/run_federated_delegation_settlement_matrix.py --fixture fixtures/federated_task_delegation/partition_replay_cases.json --output-json federated-delegation-settlement-report.json`
- Regression policy:
  - settlement reference drift, replay attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`).

## Kolme Version Compatibility Replay Evidence Contract (Issues #780, #1401, #1402)
Kolme upgrade approvals require deterministic KAMN/Kolme version compatibility validation and replay artifact evidence.

- Version compatibility validator:
  - `python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json /tmp/kolme-version-report.json`
- Fork compatibility evidence generator:
  - `python3 scripts/kolme/generate_fork_compatibility_evidence.py --upstream-release-tag v0.15.2 --fork-release-tag v0.15.2 --fork-repo njfio/kolme_fork --fork-ref refs/heads/main --ci-fast-gate PASS --output-json /tmp/kolme-fork-compatibility-report.json`
- Fork compatibility policy checker:
  - `python3 scripts/kolme/check_fork_compatibility_policy.py --report-file /tmp/kolme-fork-compatibility-report.json --expected-upstream-release-tag v0.15.2 --expected-fork-release-tag v0.15.2 --expected-fork-repo njfio/kolme_fork --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-fork-compatibility-policy-report.json`
- Replay matrix runner:
  - `python3 scripts/kolme/run_version_compatibility_replay.py --fixture fixtures/kolme_compatibility/version_compatibility_cases.json --output-json /tmp/kolme-version-replay-report.json`
- Runtime commit replay policy checker:
  - `python3 scripts/kolme/check_runtime_commit_replay_policy.py --operation-id op-go-001 --idempotency-key kolme-runtime-commit:op-go-001:state:agent:1:12 --receipt-provider kolme-local --expected-receipt-provider kolme-local --receipt-commit-id kolme-commit:op-go-001:agent:1:12 --expected-receipt-commit-id kolme-commit:op-go-001:agent:1:12 --nonce-monotonic true --replay-detected false --payload-hash-match true --receipt-finality FINAL --ci-fast-gate PASS --output-json /tmp/kolme-runtime-commit-replay-policy.json`
  - deterministic recovery taxonomy markers:
    - `recovery_reason_taxonomy_version=kamn.kolme.runtime-commit-recovery-reason-taxonomy.v1`
    - `recovery_reason_codes_csv=recovery_nonce_not_monotonic,recovery_payload_hash_mismatch,recovery_receipt_not_final,recovery_replay_detected`
    - `recovery_reason_codes_value=none|recovery_nonce_not_monotonic,recovery_payload_hash_mismatch,recovery_receipt_not_final,recovery_replay_detected`
  - deterministic retransmission evidence markers:
    - `retransmission_evidence_contract_version=v1`
    - `nonce_idempotency_contract_version=v1`
- Runtime commit replay matrix runner:
  - `python3 scripts/kolme/run_runtime_commit_replay_tamper_matrix.py --fixture fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json --output-json /tmp/kolme-runtime-commit-replay-report.json`
- Runtime commit adapter replay/finality fast lane:
  - `bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh`
  - `cargo test -p kamn-kolme --test runtime_commit_module_boundary_contracts`
  - `cargo test -p kamn-core --test kolme_runtime_commit_import_boundary`
  - adapter reason-code checks include:
    - `receipt_provider_mismatch`
    - `receipt_not_final`
- Runtime commit submit/finality evidence policy checker:
  - `python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/kolme-local-runtime-commit-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-runtime-commit-live-policy.json`
  - deterministic submit/finality taxonomy markers:
    - `submit_finality_reason_taxonomy_version=kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1`
    - `submit_finality_reason_codes_csv=submit_finality_reason_mismatch_for_finality_enabled_run,submit_finality_reason_mismatch_for_submit_only_run`
    - `submit_finality_reason_codes_value=none|submit_finality_reason_mismatch_for_finality_enabled_run|submit_finality_reason_mismatch_for_submit_only_run`
  - deterministic provider-failure taxonomy markers:
    - `provider_failure_reason_taxonomy_version=kamn.kolme.local-runtime-commit-provider-failure-reason-taxonomy.v1`
    - `provider_failure_reason_codes_csv=provider_client_contract_mismatch,provider_contract_enforcement_mode_mismatch,provider_live_contract_marker_mismatch,provider_live_contract_marker_missing,provider_in_memory_reference_detected,provider_hint_in_memory_provider_reference_detected,provider_submit_profile_contract_mismatch,provider_command_marker_mismatch,provider_command_marker_missing,provider_signing_profile_marker_mismatch,provider_signing_profile_marker_missing,provider_signing_profile_simulated_detected,provider_signer_adapter_contract_mismatch,provider_signing_curve_contract_mismatch,provider_signing_profile_contract_version_mismatch,live_command_in_memory_provider_reference_detected`
    - `provider_failure_reason_codes_value=none|provider_client_contract_mismatch,provider_contract_enforcement_mode_mismatch,provider_live_contract_marker_mismatch,provider_live_contract_marker_missing,provider_in_memory_reference_detected,provider_hint_in_memory_provider_reference_detected,provider_submit_profile_contract_mismatch,provider_command_marker_mismatch,provider_command_marker_missing,provider_signing_profile_marker_mismatch,provider_signing_profile_marker_missing,provider_signing_profile_simulated_detected,provider_signer_adapter_contract_mismatch,provider_signing_curve_contract_mismatch,provider_signing_profile_contract_version_mismatch,live_command_in_memory_provider_reference_detected`
  - deterministic submission/finality lineage-failure markers:
    - `request_payload_evidence_artifact_path_lineage_mismatch`
    - `submit_evidence_artifact_path_lineage_mismatch`
    - `finality_evidence_artifact_path_lineage_mismatch`
- Signed-message + commit evidence demo policy checker:
  - `python3 scripts/kolme/check_local_signed_to_kolme_demo_policy.py --report-file /tmp/kolme-local-signed-to-kolme-demo-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-signed-to-kolme-demo-policy.json`
  - deterministic mismatch marker:
    - `signed_message_commit_evidence_mismatch`
  - deterministic taxonomy/normalization markers:
    - `demo_evidence_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo.reason-taxonomy.v1`
    - `demo_evidence_normalization_version=kamn.kolme.local-signed-to-kolme-demo.evidence-normalization.v1`
  - deterministic native signer markers:
    - `runtime_signing_profile_contract_version=v1`
    - `runtime_signing_profile=kolme-fork-secp256k1-v1`
    - `native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1`
    - `native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch`
    - `native_signer_reason_codes_value=none|runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch`
- PR fast contract lane:
  - `bash scripts/kolme/run_version_compatibility_contract_lane.sh`
  - `bash scripts/kolme/run_runtime_commit_replay_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json kolme-version-compatibility-report.json`
- Regression policy:
  - incompatible upgrade signature (`kamn 1.2.x` + `kolme 0.14.x`) remains blocked (`Regression: #775`).
  - fork release-tag drift remains blocked (`Regression: #1401`).
  - fork policy checker rejects malformed schema, tuple mismatch, and missing required reason codes (`Regression: #1402`).
  - runtime commit replay/tamper mismatches and non-final receipts force `NO-GO` (`Regression: #827`).
  - runtime commit replay recovery/nonce-idempotency taxonomy drift forces `NO-GO` (`Regression: #4422`).
  - adapter transport/provider mismatch and non-final receipt reason-code checks remain fail-closed (`Regression: #980`).
  - submit/finality success-reason mismatches force `NO-GO` (`submit_finality_reason_mismatch_for_finality_enabled_run`, `submit_finality_reason_mismatch_for_submit_only_run`) (`Regression: #4420`).
  - submission/finality artifact lineage cross-link drift forces `NO-GO` (`request_payload_evidence_artifact_path_lineage_mismatch`, `submit_evidence_artifact_path_lineage_mismatch`, `finality_evidence_artifact_path_lineage_mismatch`) (`Regression: #4372`).
  - provider-failure taxonomy drift or non-deterministic provider-failure reason projection forces `NO-GO` (`Regression: #4378`).
  - signed-message checkpoint failures accepted alongside commit evidence success force `NO-GO` (`signed_message_commit_evidence_mismatch`) (`Regression: #4497`).
  - signed-to-Kolme taxonomy/normalized-evidence drift forces `NO-GO` (`reason_taxonomy_overall_mismatch`, `normalized_evidence_status_mismatch:local_kamn_runtime_integration_run`) (`Regression: #4498`).
  - signed-to-Kolme simulated signing profile acceptance or native signer profile marker omission forces `NO-GO` (`runtime_commit_simulated_signing_profile_detected`, `runtime_commit_native_signing_profile_marker_missing`) (`Regression: #4373`).
  - signed-to-Kolme runtime signing profile drift and native signer taxonomy drift force `NO-GO` (`runtime_signing_profile_mismatch`) (`Regression: #4380`).

## Failover + Sync Drill Evidence Contract (Issues #787, #788)
Runtime failover and sync readiness requires deterministic lane routing, budget guards, and scheduled-cadence enforcement before release approval.

- Lane selector:
  - `bash scripts/runtime/select_failover_sync_drill_lane.sh --event-name pull_request`
- PR fast contract lane:
  - `bash scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh --output-json /tmp/failover-sync-preflight-report.json`
- Scheduled deep lane entrypoint:
  - `KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled bash scripts/runtime/run_failover_sync_drill_deep_lane.sh --output-json /tmp/failover-sync-deep-report.json`
- Suite/artifact summary entrypoint:
  - `bash scripts/runtime/run_failover_sync_drill_suite.sh --event-name schedule --output-json /tmp/failover-sync-suite-report.json`
- Regression policy:
  - preflight runtime budget overruns force lane failure (`Regression: #788`).
  - unscheduled deep-lane execution force-fails via scheduled-only cadence guard (`Regression: #788`).

## Peer Adapter Reason Projection and Multi-Process Validation Hooks (Issue #4320)
Peer adapter retry/timeout evidence must project deterministic reason classes and remain repeatable across process-isolated multi-process validation lanes.

- Projection contract selectors:
  - `cargo test -p kamn-core --test p2p_peer_adapter_reason_projection`
- Multi-process validation hooks:
  - `bash scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh --mode run --lane-profile smoke --ci-fast-gate PASS`
  - `bash scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh --report-file /tmp/libp2p-convergence-process-isolated-live-summary.json --expected-final-decision GO --ci-fast-gate PASS`
  - `bash scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh --mode run --lane-profile deep --ci-fast-gate FAIL`
- Deterministic taxonomy and projection markers:
  - `peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1`
  - `peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer`
  - `peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout`
  - `peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted`
  - `peer_adapter_multi_process_validation_local_heavy_status=required`
- Regression policy:
  - peer adapter reason projection drift and multi-process hook contract drift force `NO-GO` (`Regression: #4320`).

## Live-Network Pilot Launch and Rollback Evidence Gates (Issue #830)
Pilot launch gates require deterministic smoke and scheduled/manual deep-lane evidence before release approval.

- PR-fast smoke evidence:
  - `bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json`
- Scheduled/manual deep evidence:
  - `bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name schedule --output-json /tmp/live-network-pilot-report.json`
- Deep summary policy checker:
  - `bash scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh --summary-file /tmp/live-network-pilot-report.json`
- Contract lane:
  - `bash scripts/runtime/run_live_network_pilot_deep_contract_lane.sh`
- Partition/reconnect lane selector:
  - `bash scripts/runtime/select_live_network_partition_reconnect_lane.sh --event-name pull_request`
- Partition/reconnect matrix smoke lane:
  - `bash scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh --event-name pull_request --output-json /tmp/live-network-partition-reconnect-smoke-report.json`
- Partition/reconnect matrix deep lane:
  - `bash scripts/runtime/run_live_network_partition_reconnect_deep_lane.sh --event-name schedule --output-json /tmp/live-network-partition-reconnect-deep-report.json`
- Partition/reconnect matrix policy checker:
  - `bash scripts/runtime/check_live_network_partition_reconnect_policy.sh --report-file /tmp/live-network-partition-reconnect-smoke-report.json`
- Partition/reconnect matrix contract lane:
  - `bash scripts/runtime/run_live_network_partition_reconnect_contract_lane.sh --event-name pull_request --output-json /tmp/live-network-partition-reconnect-contract-report.json`
- Partition/reconnect matrix fixture:
  - `fixtures/runtime/live_network_partition_reconnect_matrix_cases.json`
- Regression policy:
  - missing smoke/deep pilot evidence or non-`GO` pilot decisions force launch `NO-GO` and trigger rollback review (`Regression: #830`).
  - stale/tampered partition/reconnect matrix artifacts and replay anomalies force `NO-GO` (`Regression: #982`).

## Validator/Watchdog Proof Consensus Evidence Contract (Issue #996)
Validator/watchdog proof-consensus rollout requires deterministic anomaly evidence and deep-lane budget/cadence controls before release approval.

- Evidence bundle generator:
  - `bash scripts/runtime/generate_watchdog_proof_consensus_evidence_bundle.sh --output-file /tmp/watchdog-proof-consensus-go.json --message-id urn:uuid:watchdog-proof-go-996 --artifact-id artifact-watchdog-go-996 --consensus-status ConsensusValid --required-quorum 2 --valid-attestation-count 2 --invalid-attestation-count 0 --replay-attestation-count 0 --cadence fast --runtime-seconds 4 --max-seconds 90 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/runtime/check_watchdog_proof_consensus_policy.sh --bundle-file /tmp/watchdog-proof-consensus-go.json`
- PR fast contract lane:
  - `bash scripts/runtime/run_watchdog_proof_consensus_contract_lane.sh --output-file /tmp/watchdog-proof-consensus-contract.json`
- Scheduled/manual deep lane entrypoint:
  - `KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE=scheduled bash scripts/runtime/run_watchdog_proof_consensus_deep_lane.sh --event-name schedule --output-json /tmp/watchdog-proof-consensus-deep-summary.json`
- Runtime/cadence controls:
  - `KAMN_WATCHDOG_PROOF_CONSENSUS_MAX_SECONDS`
  - `KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE`
  - `KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_MAX_SECONDS`
- Regression policy:
  - proof-consensus deep-lane budget overruns and unscheduled cadence execution force `NO-GO` (`Regression: #996`).

## Governance Simulation and Human-Veto Evidence Contract (Issue #748)
Governance activation requires deterministic simulation, veto, timelock, and approval evidence before GO decisions.

- Stable shell wrappers:
  - `scripts/governance/generate_governance_simulation_evidence_bundle.sh`
  - `scripts/governance/check_governance_simulation_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_simulation_contract.py`
- Shared Python contract-lane implementation:
  - `scripts/governance/governance_simulation_contract_lane_contract.py` (uses `framework.contract_lane_helpers`)
- Evidence bundle generator:
  - `bash scripts/governance/generate_governance_simulation_evidence_bundle.sh --output-file /tmp/governance-simulation.json --proposal-id gov-proposal-activation-001 --simulation-hash sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --simulation-complete true --veto-window-open false --veto-recorded false --timelock-expired true --required-approvals 2 --received-approvals 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/governance/check_governance_simulation_policy.sh --bundle-file /tmp/governance-simulation.json`
- PR fast contract lane:
  - `bash scripts/governance/run_governance_simulation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_governance_simulation_deep_lane.sh --output-json governance-simulation-report.json`
- Replay matrix runner:
  - `python3 scripts/governance/run_governance_simulation_matrix.py --fixture fixtures/governance_simulation/veto_timelock_cases.json --output-json governance-simulation-report.json`
- Regression policy:
  - simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`).

## Governance Stake/Slash Risk Threshold Contract (Issue #750)
Governance activation requires deterministic stake/slash risk thresholds to block unsafe economic-impact scenarios.

- Stable shell wrappers:
  - `scripts/governance/generate_stake_slash_risk_evidence_bundle.sh`
  - `scripts/governance/check_stake_slash_risk_policy.sh`
- Shared Python implementation:
  - `scripts/governance/stake_slash_risk_contract.py`
- Shared Python contract-lane implementation:
  - `scripts/governance/stake_slash_risk_contract_lane_contract.py` (uses `framework.contract_lane_helpers`)
- Evidence bundle generator:
  - `bash scripts/governance/generate_stake_slash_risk_evidence_bundle.sh --output-file /tmp/stake-slash-risk.json --proposal-id gov-risk-001 --simulation-hash sha256:1111111111111111111111111111111111111111111111111111111111111111 --stake-at-risk-bps 120 --max-stake-at-risk-bps 300 --slash-probability-bps 40 --max-slash-probability-bps 150 --validator-churn-bps 60 --max-validator-churn-bps 180 --quorum-safety-margin-bps 220 --min-quorum-safety-margin-bps 150 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/governance/check_stake_slash_risk_policy.sh --bundle-file /tmp/stake-slash-risk.json`
- PR fast contract lane:
  - `bash scripts/governance/run_stake_slash_risk_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_stake_slash_risk_deep_lane.sh --output-json governance-stake-slash-report.json`
- Replay matrix runner:
  - `python3 scripts/governance/run_stake_slash_risk_matrix.py --fixture fixtures/governance_stake_slash/risk_threshold_cases.json --output-json governance-stake-slash-report.json`
- Regression policy:
  - unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`).

## Reputation Dispute Evidence Contract (Issue #738)
Reputation dispute decisions require deterministic evidence bundles so trust-score corrections remain auditable and tamper-evident.

- Shared Python contract-lane implementation:
  - `scripts/reputation/reputation_dispute_contract_lane_contract.py` (uses `framework.contract_lane_helpers`)
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_dispute_evidence_bundle.sh --output-file /tmp/reputation-dispute.json --dispute-id dispute-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --dispute-reason-code QUALITY --evidence-uri s3://kamn-audit/reputation/dispute-001.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --evidence-hash-verified PASS --original-trust-score 640 --proposed-trust-score 560 --max-adjustment-points 120 --policy-window-open true --approval-recorded true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_dispute_policy.sh --bundle-file /tmp/reputation-dispute.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_dispute_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_reputation_dispute_deep_lane.sh --output-json reputation-dispute-report.json`
- Replay matrix runner:
  - `python3 scripts/reputation/run_reputation_dispute_matrix.py --fixture fixtures/reputation_dispute/replay_cases.json --output-json reputation-dispute-report.json`
- Regression policy:
  - tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`).

## Token Launch Handoff Evidence Contract (Issue #714)
Token launch readiness requires deterministic supply/allocation and approval evidence before activation.

- Evidence bundle generator:
  - `bash scripts/token/generate_token_launch_handoff_evidence_bundle.sh --output-file /tmp/token-launch-handoff.json --token-symbol KAMN --configured-total-supply 1000000000 --expected-total-supply 1000000000 --configured-allocation-sum 1000000000 --expected-allocation-sum 1000000000 --allocation-bucket-count 5 --expected-bucket-count 5 --genesis-hash sha256:token-launch-handoff-go-2026-02-09 --required-approvals 2 --received-approvals 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/token/check_token_launch_handoff_policy.sh --bundle-file /tmp/token-launch-handoff.json`
- PR fast contract lane:
  - `bash scripts/token/run_token_launch_handoff_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/token/run_token_launch_handoff_deep_lane.sh --output-json token-launch-handoff-report.json`
- Regression policy:
  - supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`).

## Treasury Disbursement Approval Evidence Contract (Issue #716)
Treasury disbursement execution requires deterministic approval-threshold evidence and policy-window validation.

- Evidence bundle generator:
  - `bash scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh --output-file /tmp/treasury-disbursement.json --disbursement-id disbursement-go-001 --treasury-account-id treasury-main-001 --destination-account-id ops-wallet-001 --asset-symbol KAMN --disbursement-amount 250000 --daily-limit-amount 500000 --required-approvals 2 --received-approvals 2 --approval-quorum-hash sha256:approval-go-001 --policy-window-open true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/treasury/check_treasury_disbursement_policy.sh --bundle-file /tmp/treasury-disbursement.json`
- PR fast contract lane:
  - `bash scripts/treasury/run_treasury_disbursement_contract_lane.sh`
- Shared Python implementation (contract lane):
  - `scripts/treasury/treasury_disbursement_contract_lane_contract.py`
- Regression policy:
  - insufficient approvals, approval-window closure, and daily-limit overruns force `NO-GO` (`Regression: #716`).
  - shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1278`).

## Mainnet Cutover Manifest Validation Contract (Issue #707)
Mainnet cutover requires deterministic triadic checkpoint manifests with explicit approval and dependency evidence.

- Schema contract:
  - `fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json`
- Validator:
  - `python3 scripts/cutover/validate_mainnet_cutover_manifest.py --manifest fixtures/mainnet_cutover/mainnet_cutover_manifest.valid.json --output-json /tmp/mainnet-cutover-validation-report.json`
- PR fast contract lane:
  - `bash scripts/cutover/run_mainnet_cutover_contract_lane.sh`
- Regression policy:
  - unresolved/non-prior dependencies and insufficient approvals force `NO-GO` (`Regression: #705`).

## Cutover Rollback Evidence Contract (Issue #708)
Rollback readiness and trigger execution must emit deterministic evidence before cutover approval.

- Evidence bundle generator:
  - `bash scripts/cutover/generate_cutover_rollback_evidence_bundle.sh --output-file /tmp/cutover-rollback.json --cutover-manifest-id cutover-mainnet-2026-02-09 --rollback-trigger-status CLEAR --checkpoint-state READY --failed-checkpoint-id '' --rollback-target-hash state-hash-abc --post-rollback-hash state-hash-abc --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/cutover/check_cutover_rollback_evidence_policy.sh --bundle-file /tmp/cutover-rollback.json`
- PR fast contract lane:
  - `bash scripts/cutover/run_cutover_rollback_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/cutover/run_cutover_rollback_deep_lane.sh --output-json cutover-rollback-report.json`
- Regression policy:
  - missing failed-checkpoint evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`).

## Launch Canary Critical-Path Contract (Issue #710)
Launch approval requires deterministic critical-path probe evidence covering message/task/escrow behavior.

- Probe fixture matrix:
  - `fixtures/launch_canary/critical_path_probe_cases.json`
- Matrix runner:
  - `python3 scripts/canary/run_launch_canary_matrix.py --fixture fixtures/launch_canary/critical_path_probe_cases.json --output-json /tmp/launch-canary-report.json`
- PR fast contract lane:
  - `bash scripts/canary/run_launch_canary_contract_lane.sh`
- Shared Python implementation (contract lane):
  - `scripts/canary/launch_canary_contract_lane_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/canary/run_launch_canary_deep_lane.sh --output-json launch-canary-report.json`
- Regression policy:
  - missing probe evidence and failing critical-path probes force `NO-GO` (`Regression: #710`).
  - shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1286`).

## Post-Cutover SLO Gate Evidence Contract (Issue #711)
Post-cutover launch gates require deterministic SLO evidence export with stale/partial evidence rejection.

- Evidence bundle generator:
  - `bash scripts/canary/generate_post_cutover_slo_evidence_bundle.sh --output-file /tmp/post-cutover-slo.json --window-minutes 15 --p95-latency-ms 140 --max-p95-latency-ms 200 --error-rate-bps 18 --max-error-rate-bps 25 --delivery-success-bps 9992 --min-delivery-success-bps 9950 --snapshot-age-seconds 30 --max-snapshot-age-seconds 120 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/canary/check_post_cutover_slo_policy.sh --bundle-file /tmp/post-cutover-slo.json`
- PR fast contract lane:
  - `bash scripts/canary/run_post_cutover_slo_contract_lane.sh`
- Shared Python implementation (contract lane):
  - `scripts/canary/post_cutover_slo_contract_lane_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/canary/run_post_cutover_slo_deep_lane.sh --output-json post-cutover-slo-report.json`
- Deterministic alert-governance markers:
  - `alert_rule_promotion_gate_status=verified`
  - `burn_rate_parity_status=verified`
  - `ci_local_promotion_budget_boundary_status=verified`
  - `alert_governance_reason_taxonomy_version=kamn.runtime.alert-governance-reason-taxonomy.v1`
  - `alert_governance_reason_codes_csv=alert_rule_promotion_stalled,burn_rate_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded`
- Runtime budget controls:
  - `KAMN_POST_CUTOVER_SLO_MAX_SECONDS`
  - `KAMN_POST_CUTOVER_SLO_CI_LOCAL_PROMOTION_MAX_SECONDS`
  - `KAMN_POST_CUTOVER_SLO_DEEP_MAX_SECONDS`
  - `KAMN_POST_CUTOVER_SLO_DEEP_LOCAL_ONLY`
- Regression policy:
  - stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`).
  - shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1282`).

## Local Validation
Run from repository root:

```bash
bash scripts/canary/test_run_launch_canary_matrix.sh
bash scripts/canary/test_run_launch_canary_contract_lane.sh
bash scripts/canary/test_generate_post_cutover_slo_evidence_bundle.sh
bash scripts/canary/test_run_post_cutover_slo_contract_lane.sh
bash scripts/cutover/test_validate_mainnet_cutover_manifest.sh
bash scripts/cutover/test_run_mainnet_cutover_contract_lane.sh
bash scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh
bash scripts/cutover/test_run_cutover_rollback_contract_lane.sh
bash scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh
bash scripts/escrow/test_run_settlement_reconciliation_contract_lane.sh
bash scripts/escrow/test_run_settlement_reconciliation_race_matrix.sh
bash scripts/compliance/test_generate_soc2_control_evidence_bundle.sh
bash scripts/compliance/test_run_soc2_control_evidence_contract_lane.sh
bash scripts/compliance/test_run_soc2_control_evidence_replay_matrix.sh
bash scripts/compliance/test_run_soc2_control_evidence_deep_lane.sh
bash scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh
bash scripts/compliance/test_run_dsar_legal_hold_contract_lane.sh
bash scripts/compliance/test_run_dsar_legal_hold_matrix.sh
bash scripts/compliance/test_run_dsar_legal_hold_deep_lane.sh
bash scripts/did/test_generate_federated_did_handshake_evidence_bundle.sh
bash scripts/did/test_run_federated_did_handshake_contract_lane.sh
bash scripts/did/test_run_federated_did_handshake_matrix.sh
bash scripts/did/test_run_federated_did_handshake_deep_lane.sh
bash scripts/did/test_check_federated_did_handshake_deep_policy.sh
bash scripts/did/test_run_federated_did_handshake_deep_policy_matrix.sh
bash scripts/task/test_generate_federated_delegation_settlement_evidence_bundle.sh
bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh
bash scripts/task/test_run_federated_delegation_settlement_matrix.sh
bash scripts/task/test_run_federated_delegation_settlement_deep_lane.sh
bash scripts/kolme/test_validate_version_compatibility.sh
bash scripts/kolme/test_run_version_compatibility_contract_lane.sh
bash scripts/governance/test_generate_governance_simulation_evidence_bundle.sh
bash scripts/governance/test_run_governance_simulation_contract_lane.sh
bash scripts/governance/test_run_governance_simulation_matrix.sh
bash scripts/governance/test_run_governance_simulation_deep_lane.sh
bash scripts/governance/test_generate_stake_slash_risk_evidence_bundle.sh
bash scripts/governance/test_run_stake_slash_risk_contract_lane.sh
bash scripts/governance/test_run_stake_slash_risk_matrix.sh
bash scripts/governance/test_run_stake_slash_risk_deep_lane.sh
bash scripts/reputation/test_generate_reputation_dispute_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_dispute_contract_lane.sh
bash scripts/reputation/test_run_reputation_dispute_matrix.sh
bash scripts/reputation/test_run_reputation_dispute_deep_lane.sh
bash scripts/token/test_generate_token_launch_handoff_evidence_bundle.sh
bash scripts/token/test_run_token_launch_handoff_contract_lane.sh
bash scripts/token/test_run_token_launch_handoff_deep_lane.sh
bash scripts/treasury/test_generate_treasury_disbursement_evidence_bundle.sh
bash scripts/treasury/test_run_treasury_disbursement_contract_lane.sh
bash scripts/guard/test_run_durable_guard_recovery_contract_lane.sh
bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh
bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh
bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh
bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh
cargo test -p kamn-core --test mainnet_cutover_runbook_docs
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo test -p kamn-core --test token_config_docs
cargo test -p kamn-core --test audit_export_interfaces_docs
cargo test -p kamn-core --test durable_guard_recovery_matrix
cargo test -p kamn-core --test durable_guard_snapshot_store
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
