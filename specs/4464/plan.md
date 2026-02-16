# Plan: Issue #4464

Status: Completed
Issue: #4464

## Approach

1. Add RED boundary tests for incident go/no-go lane convergence gaps and governance markers.
2. Implement deterministic CI smoke boundary + local-heavy opt-in enforcement in deploy lane scripts.
3. Emit boundary taxonomy/version/reason markers from contract lane execution.
4. Add CI strategy docs incident boundary matrix and fail-closed reason references.
5. Add docs-contract assertions in `ci_strategy_docs` for drift protection.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/run_gonogo_evidence_deep_lane.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks / Mitigations

- Risk: boundary enforcement breaks existing run-mode aggregation paths.
  - Mitigation: reuse existing go/no-go run-mode opt-in env and add explicit tests.
- Risk: reason-code taxonomy drift between docs and scripts.
  - Mitigation: enforce docs contract tests for taxonomy/version/reason markers.
- Risk: accidentally broadening CI scope.
  - Mitigation: keep default CI smoke path bounded and deep lane opt-in gated.

## Interfaces / Contracts

- Contract lane bound:
  - `--max-seconds` (CI smoke, deterministic upper bound)
- Deep lane bound:
  - explicit `KAMN_GONOGO_GATE_LOCAL_OPT_IN=1` requirement
  - optional `--max-seconds` bounded local-heavy runtime
- Deterministic marker surfaces:
  - `incident_gonogo_boundary_reason_taxonomy_status`
  - `incident_gonogo_boundary_reason_taxonomy_version`
  - `incident_gonogo_boundary_reason_codes_csv`
  - `ci_smoke_lane_cost_profile`
  - `local_heavy_lane_execution_mode`

## ADR

No ADR required (no new dependency, protocol, or schema migration).
