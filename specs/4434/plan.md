# Plan: Issue #4434

Status: Completed
Issue: #4434

## Approach

1. Add RED tests for live-validation mismatch/tamper and partial-evidence acceptance paths.
2. Implement deterministic live-gate reason taxonomy/version/csv marker emission in go/no-go
   generator/checker and lane wrappers.
3. Enforce explicit CI smoke/local-heavy boundary governance for live go/no-go lane execution.
4. Update CI strategy and release checklist docs with live boundary/taxonomy matrix markers.
5. Add docs-contract assertions to fail closed on live governance marker drift.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/run_gonogo_evidence_deep_lane.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `docs/ci/strategy.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks / Mitigations

- Risk: existing incident gate marker surface regresses while adding live gate markers.
  - Mitigation: preserve existing incident assertions and add additive live marker assertions.
- Risk: live milestone bundle schema drift breaks checker parity.
  - Mitigation: update generator/checker symmetrically and verify tamper mismatch paths.
- Risk: CI scope creep from deep lane execution.
  - Mitigation: enforce explicit local-heavy opt-in and bounded CI smoke max-seconds checks.

## Interfaces / Contracts

- Live gate marker surface (deterministic):
  - `live_gonogo_boundary_reason_taxonomy_status`
  - `live_gonogo_boundary_reason_taxonomy_version`
  - `live_gonogo_boundary_reason_codes_csv`
- Boundary controls:
  - CI smoke: `--max-seconds` bounded upper limit
  - Local-heavy: explicit opt-in env with bounded `--max-seconds`
- Live milestone bundle contract:
  - deterministic taxonomy/version/csv fields aligned across generator and checker.

## ADR

No ADR required (no new dependency, no protocol or schema migration outside existing go/no-go
contract surface).
