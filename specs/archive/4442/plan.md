# Plan: Issue #4442

Status: Completed
Issue: #4442

## Approach

1. Implement additive live-go/no-go taxonomy marker emission in generator/checker and lane scripts.
2. Add deterministic CI smoke/local-heavy boundary reason codes for live-go/no-go enforcement.
3. Update docs and docs-contract tests for live-go/no-go boundary/taxonomy governance.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/run_gonogo_evidence_deep_lane.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `docs/ci/strategy.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks / Mitigations

- Risk: introducing live markers regresses existing incident marker coverage.
  - Mitigation: additive markers only; retain existing incident assertions.
- Risk: checker/generator parity drift for live milestone bundle fields.
  - Mitigation: enforce parity via tamper regression tests.

## Interfaces / Contracts

- Live taxonomy marker surface:
  - `live_gonogo_boundary_reason_taxonomy_status`
  - `live_gonogo_boundary_reason_taxonomy_version`
  - `live_gonogo_boundary_reason_codes_csv`
- Deterministic fail-closed boundary reason codes for CI smoke and local-heavy paths.

## ADR

No ADR required.
