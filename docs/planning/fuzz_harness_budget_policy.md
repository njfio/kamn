# Fuzz Harness Budget Policy

This policy defines deterministic, bounded coverage-guided parser fuzz targets for DID and
message-envelope validation.

## Scope

Covered by `crates/kamn-core/tests/input_mutation_coverage_guided.rs`:

- DID parser coverage-guided seed frontier (`AgentDid::parse`)
- message-envelope validator coverage-guided seed frontier (`CanonicalMessageEnvelope::validate`)
- minimal replay prefix minimization (`minimal_failing_seed_prefix`)

## Contract Markers

- `coverage_guided_schema=kamn.runtime.input-mutation-coverage-guided-contract-report.v1`
- `replay_schema=kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1`
- `replay_artifact_key=input_mutation_coverage_guided_replay:v1`
- `seed_model=deterministic_mixed_seed_frontier`
- `minimizer=minimal_failing_seed_prefix`
- `ci_fast_gate_scope=bounded_contract_local_deep_only`

## Seed and Budget Profile

- deterministic frontier scan cap: `1024` seeds (contract lane)
- deep stress scan cap: `16384` seeds (ignored/deep lane)
- default contract runtime budget: `KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS=120`
- default deep runtime budget: `KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS=180`
- deep lane opt-in gate: `KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY=true`

## Evidence Commands

- contract lane:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json`
- envelope-only bounded local run:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --target envelope --output-json /tmp/input-mutation-coverage-guided-envelope-report.json`
- DID-only bounded local run:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --target did --output-json /tmp/input-mutation-coverage-guided-did-report.json`
- deep/local-only lane:
  - `bash scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh`

## Policy

- The contract lane is bounded and may be invoked from merge-critical runtime mutation checks.
- The deep lane is local-only by default and must stay excluded from `ci-fast-gate`.
- Policy enforcement remains fail-closed by validating:
  - workflow exclusion of `run_input_mutation_coverage_guided_deep_lane.sh`
  - runtime mutation lane marker `runtime_input_mutation_coverage_guided_deep=skipped_local_only`

## Regression

- Regression: #2693
