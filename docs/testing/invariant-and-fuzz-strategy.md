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
- Concurrency race lane:
  - `bash scripts/runtime/run_concurrency_state_mutation_contract_lane.sh`
- Combined contract lane:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Combined policy checker:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`

## Determinism Strategy

- Property coverage uses deterministic generated-sequence tests for task,
  escrow/dispute-refund, and peer lifecycle transitions, and emits replay
  metadata artifact key `lifecycle_property_replay:v1`.
- Mutation/fuzz smoke lanes use fixed malformed/tampered classes with stable
  fail-closed reason signatures.
- Concurrency lanes use replay fixtures and deterministic round-based checks to
  guard winner exclusivity and terminal-state safety.

## Evidence and Policy Contracts

- Lifecycle property report schema:
  - `kamn.runtime.lifecycle-property-contract-report.v1`
- Lifecycle property replay artifact key:
  - `lifecycle_property_replay:v1`
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
  - `elapsed_seconds`
  - `max_seconds`
  - `reason_codes`
- Required pass reason code:
  - `none`

## Runtime Budgets

- Property lane runtime budget env:
  - `KAMN_RUNTIME_LIFECYCLE_PROPERTY_MAX_SECONDS` (default `120`)
- Combined lane runtime budget env:
  - `KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS` (default `180`)
- ZK witness mutation routing control:
  - `KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP` (`false` for fast lane by default)

## CI Scope and Cost Posture

- `scripts/ci/select_targets.sh` routes changes to this strategy doc into the
  runtime contract scope so lane/policy contracts are revalidated.
- `scripts/ci/test_select_targets.sh` guards that routing to keep docs changes
  fail-closed without escalating to full-suite runs.
