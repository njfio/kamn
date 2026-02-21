# R49 Ignored-Test Periodic Re-Evaluation

## Summary
Periodic R49 re-evaluation was executed for the current ignored-test inventory to confirm that every ignored test remains an intentional deep-lane opt-in decision with explicit rationale and active tracking.

## Evidence Commands
- `bash scripts/ci/check_ignored_test_inventory_drift.sh --baseline-file fixtures/ci/ignored_test_inventory_baseline.json --metadata-file fixtures/ci/ignored_test_inventory_metadata.json --promotion-criteria-file fixtures/ci/ignored_test_promotion_criteria.json --output-json /tmp/ignored-test-inventory-drift-report-r49.json`
- `rg -n "#\\[\\s*ignore" crates -g '*.rs'`

## Deterministic Markers
- `ignored_test_disposition_schema_version=kamn.review.ignored-test-disposition.v1`
- `ignored_test_periodic_review_cycle=R49`
- `ignored_test_inventory_count=12`
- `ignored_test_inventory_evidence_command=bash scripts/ci/check_ignored_test_inventory_drift.sh`
- `ignored_test_inventory_drift_status=pass`
- `ignored_test_inventory_reason_codes=none`
- `ignored_test_disposition_decision_set=retain|promote|deprecate`
- `ignored_test_disposition_default_decision=retain`
- `ignored_test_periodic_review_next_due_cycle=R54`

## Inventory Dispositions
| Source File | Test Name | Decision | Rationale | Tracking |
|---|---|---|---|---|
| `crates/kamn-core/src/channel_models.rs` | `performance_channel_snapshot_deep_lane_stress` | retain | deep-lane stress runtime unsuitable for fast PR lane; bounded opt-in remains policy-compliant | `#2843` |
| `crates/kamn-core/src/message_lifecycle.rs` | `performance_message_lifecycle_snapshot_deep_lane_stress` | retain | deep-lane stress runtime unsuitable for fast PR lane; bounded opt-in remains policy-compliant | `#2843` |
| `crates/kamn-core/src/runtime_tests_network_fault.rs` | `performance_network_fault_simulation_chaos_lane_stress` | retain | chaos-lane profile is local-heavy and intentionally scheduled; retain ignore for deterministic fast-gate stability | `#2843` |
| `crates/kamn-core/src/runtime_tests_snapshot_store.rs` | `performance_file_snapshot_store_recovery_deep_lane_large_payload` | retain | large-payload recovery stress remains deep-lane-only until bounded runtime evidence is promoted | `#2843` |
| `crates/kamn-core/src/task_operations.rs` | `performance_task_operation_snapshot_store_deep_lane_stress` | retain | deep-lane task snapshot stress remains intentionally excluded from fast lane | `#2843` |
| `crates/kamn-core/tests/concurrency_state_mutation.rs` | `performance_concurrency_state_mutation_deep_lane_stress` | retain | concurrency stress lane remains local-heavy and scheduled; retain ignore pending promotion criteria completion | `#2843` |
| `crates/kamn-core/tests/durable_guard_recovery_matrix.rs` | `performance_durable_guard_recovery_matrix_deep_lane` | retain | matrix runtime profile remains deep-lane-only; no regression evidence requiring promotion | `#2843` |
| `crates/kamn-core/tests/durable_guard_snapshot_store.rs` | `performance_bundle_store_deep_lane_stress` | retain | deep bundle-store stress remains opt-in for deterministic fast-gate bounds | `#2843` |
| `crates/kamn-core/tests/signer_backend.rs` | `performance_signer_emulator_bulk_signing_deep_lane` | retain | signer bulk-signing performance lane remains scheduled deep-lane workload | `#2843` |
| `crates/kamn-core/tests/zk_witness_fuzz_smoke.rs` | `performance_zk_witness_mutation_deep_lane_stress` | retain | deep mutation stress remains scheduled to avoid fast-lane runtime inflation | `#2843` |
| `crates/kamn-sdk/tests/live_transport_agent.rs` | `performance_live_transport_multi_client_deep_lane` | retain | multi-client live transport deep lane remains local-heavy opt-in | `#2843` |
| `crates/kamn-sdk/tests/tcp_failover_matrix.rs` | `performance_tcp_failover_reconnect_matrix_deep_lane` | retain | failover reconnect matrix stress is intentionally scheduled as deep lane | `#2843` |

## Outcome
All 12 ignored tests remain aligned with baseline and metadata inventories, with no drift or missing rationale metadata. No promote/deprecate actions are required in this R49 cycle.
