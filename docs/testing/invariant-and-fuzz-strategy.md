# Invariant and Fuzz Strategy

This document defines the low-cost verification strategy for lifecycle
invariants, mutation/fuzz smoke checks, and concurrency race contracts.

## Goals

- Keep PR validation deterministic and reproducible.
- Fail closed on contract drift in evidence schemas and reason codes.
- Preserve bounded fast-gate runtime/cost behavior.

## Command Surface

- Property/invariant lane:
  - `bash scripts/runtime/run_lifecycle_property_contract_lane.sh`
  - `bash scripts/runtime/run_lifecycle_property_contract_lane.sh --output-json /tmp/lifecycle-property-contract-report.json`
- Mutation/fuzz smoke lane:
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh`
  - `bash scripts/runtime/run_input_mutation_contract_lane.sh --output-json /tmp/input-mutation-contract-report.json`
  - local envelope-only bounded run:
    - `bash scripts/runtime/run_input_mutation_contract_lane.sh --target envelope --output-json /tmp/input-mutation-envelope-smoke-report.json`
  - local DID-only bounded run:
    - `bash scripts/runtime/run_input_mutation_contract_lane.sh --target did --output-json /tmp/input-mutation-did-smoke-report.json`
- Coverage-guided parser fuzz lane:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json`
  - local envelope-only bounded run:
    - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --target envelope --output-json /tmp/input-mutation-coverage-guided-envelope-report.json`
  - local DID-only bounded run:
    - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --target did --output-json /tmp/input-mutation-coverage-guided-did-report.json`
  - local-only deep lane:
    - `bash scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh`
- Concurrency race lane:
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh`
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh --output-json /tmp/concurrency-mutation-contract-report.json`
- Combined contract lane:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Combined policy checker:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`
- Live transport replay/tamper contract lane:
  - `bash scripts/sdk/run_live_transport_replay_tamper_contract_lane.sh --output-report /tmp/live-transport-replay-tamper-contract-report.json`
- Live transport replay/tamper fast lane:
  - `bash scripts/sdk/run_live_transport_replay_tamper_fast_lane.sh --output-report /tmp/live-transport-replay-tamper-fast-report.json`
- Live transport replay/tamper deep lane:
  - `bash scripts/sdk/run_live_transport_replay_tamper_deep_lane.sh --output-report /tmp/live-transport-replay-tamper-deep-report.json`
- Live transport replay/tamper policy checker:
  - `bash scripts/sdk/check_live_transport_replay_tamper_policy.sh --bundle-file /tmp/live-transport-replay-tamper-contract-report.json`

## Determinism Strategy

- Property coverage uses deterministic generated-sequence tests for task,
  escrow/dispute-refund, and peer lifecycle transitions, and emits replay
  metadata artifact key `lifecycle_property_replay:v1`.
- Mutation/fuzz smoke lanes use fixed malformed/tampered classes with stable
  fail-closed reason signatures and emit replay metadata artifact key
  `input_mutation_replay:v1`.
- Coverage-guided parser fuzz lane uses deterministic seed-frontier discovery
  with bounded replay-prefix minimization and emits replay metadata artifact key
  `input_mutation_coverage_guided_replay:v1`.
- Concurrency lanes use replay fixtures and deterministic round-based checks to
  guard winner exclusivity and terminal-state safety, and emit replay metadata
  artifact key `concurrency_mutation_replay:v1`.
- Threaded lifecycle transition regression coverage is anchored in
  `crates/kamn-core/tests/lifecycle_concurrency_contracts.rs`, validating:
  - task terminal-state fail-closed behavior under parallel completion attempts
  - escrow invalid-transition rejection under parallel dispute attempts
  - peer lifecycle invalid-edge rejection under parallel handshake attempts

## Evidence and Policy Contracts

- Lifecycle property report schema:
  - `kamn.runtime.lifecycle-property-contract-report.v1`
- Lifecycle property replay metadata schema:
  - `kamn.runtime.lifecycle-property-replay-metadata.v1`
- Lifecycle property replay artifact key:
  - `lifecycle_property_replay:v1`
- Input mutation report schema:
  - `kamn.runtime.input-mutation-contract-report.v1`
- Input mutation replay metadata schema:
  - `kamn.runtime.input-mutation-replay-metadata.v1`
- Input mutation replay artifact key:
  - `input_mutation_replay:v1`
  - seed corpus keys:
    - `input_mutation_envelope_seed:v1`
    - `input_mutation_did_seed:v1`
- Coverage-guided input mutation report schema:
  - `kamn.runtime.input-mutation-coverage-guided-contract-report.v1`
- Coverage-guided replay metadata schema:
  - `kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1`
- Coverage-guided replay artifact key:
  - `input_mutation_coverage_guided_replay:v1`
  - seed corpus keys:
    - `input_mutation_coverage_guided_envelope_seed:v1`
    - `input_mutation_coverage_guided_did_seed:v1`
  - minimizer marker:
    - `minimal_failing_seed_prefix`
- Concurrency mutation report schema:
  - `kamn.runtime.concurrency-mutation-contract-report.v1`
- Concurrency mutation replay artifact key:
  - `concurrency_mutation_replay:v1`
- Combined lane report schema:
  - `kamn.runtime.invariant-fuzz-concurrency-contract-report.v1`
- Required summary fields:
  - `status`
  - `property_lane_status`
  - `fuzz_lane_status`
  - `concurrency_lane_status`
  - `property_replay_schema_version`
  - `property_replay_artifact_key`
  - `property_replay_test_count`
  - `fuzz_replay_schema_version`
  - `fuzz_replay_artifact_key`
  - `fuzz_replay_test_count`
  - `concurrency_replay_schema_version`
  - `concurrency_replay_artifact_key`
  - `concurrency_replay_test_count`
  - `elapsed_seconds`
  - `max_seconds`
  - `reason_taxonomy_version`
  - `reason_codes_csv`
  - `reason_codes_value`
  - `final_decision`
  - `reason_codes`
- Invariant policy checker deterministic markers:
  - `invariant_policy_reason_taxonomy_version=kamn.runtime.invariant-fuzz-concurrency-policy-reason-taxonomy.v1`
  - `invariant_policy_reason_codes_csv=property_lane_failed,fuzz_lane_failed,concurrency_lane_failed,runtime_budget_exceeded,ci_smoke_local_heavy_boundary_status_mismatch,ci_smoke_lane_cost_profile_mismatch,local_heavy_lane_execution_mode_mismatch,missing_required_report_fields,schema_version_mismatch,status_value_invalid,lane_status_value_invalid,property_replay_schema_version_mismatch,property_replay_artifact_key_mismatch,property_replay_test_count_invalid,fuzz_replay_schema_version_mismatch,fuzz_replay_artifact_key_mismatch,fuzz_replay_test_count_invalid,concurrency_replay_schema_version_mismatch,concurrency_replay_artifact_key_mismatch,concurrency_replay_test_count_invalid,elapsed_seconds_invalid,max_seconds_invalid,reason_codes_payload_invalid,status_contract_mismatch,reason_codes_contract_mismatch,reason_taxonomy_version_mismatch,reason_codes_csv_mismatch,reason_codes_value_mismatch,final_decision_mismatch`
  - `invariant_policy_reason_codes_value=none|<csv>`
  - `invariant_policy_expected_reason_codes_value=none|<csv>`
  - `invariant_policy_observed_reason_codes_value=none|<csv>`
  - `invariant_policy_ci_smoke_local_heavy_boundary_status=verified`
  - `invariant_policy_ci_smoke_lane_cost_profile=low`
  - `invariant_policy_local_heavy_lane_execution_mode=opt_in`
  - `invariant_policy_final_decision=GO|NO-GO`
- Lifecycle property replay metadata contract fields:
  - `executed_cases`
  - `generated_sequence_bounds`
- Input mutation replay metadata contract fields:
  - `target`
  - `seed_corpus_keys`
- Required pass reason code:
  - `none`
- Live transport replay/tamper report schema:
  - `kamn.sdk.live-transport-replay-tamper-evidence.v1`

## Runtime Budgets

- Property lane runtime budget env:
  - `KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS` (default `120`)
- Input mutation lane runtime budget env:
  - `KAMN_RUNTIME_INPUT_MUTATION_MAX_SECONDS` (default `120`)
- Coverage-guided input mutation lane runtime budget env:
  - `KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS` (default `120`)
  - `KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS` (default `180`)
  - `KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY` (`false` by default)
- Concurrency mutation lane runtime budget env:
  - `KAMN_RUNTIME_CONCURRENCY_MUTATION_MAX_SECONDS` (default `120`)
- Combined lane runtime budget env:
  - `KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS` (default `180`)
- ZK witness mutation routing control:
  - `KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP` (`false` for fast lane by default)

## CI Scope and Cost Posture

- `scripts/ci/select_targets.sh` routes changes to this strategy doc into the
  runtime contract scope so lane/policy contracts are revalidated.
- `scripts/ci/test_select_targets.sh` guards that routing to keep docs changes
  fail-closed without escalating to full-suite runs.
- Deep coverage-guided parser fuzz runs remain local-only and are explicitly
  excluded from `ci-fast-gate` (`Regression: #2693`).
