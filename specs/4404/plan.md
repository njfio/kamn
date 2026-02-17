# Plan — #4404

Status: Reviewed

## Approach

- Extend telemetry policy tests to add RED coverage for evidence-link completeness and partial-evidence acceptance drift.
- Harden unified telemetry local-heavy policy checks so run-mode evidence links must be complete, readable, and convergent.
- Ensure run-lane evidence artifacts are preserved so policy links are valid and auditable.
- Add deterministic reason-taxonomy markers for evidence-link failures and keep normalized reason output stable.
- Update CI strategy docs to reflect the new telemetry evidence-convergence governance markers.

## Affected Areas

- `scripts/runtime/unified_api_observability_local_heavy_live_contract.py`
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: stricter evidence checks could fail existing run-mode consumers with incomplete artifacts.
  - Mitigation: preserve and publish run-mode artifacts from lane execution; deterministic reason codes explain failures.
- Risk: taxonomy drift between implementation/tests/docs.
  - Mitigation: keep a single source constant for taxonomy CSV and update tests/docs in same PR.

## Contract Notes

- Keep existing pass-path markers additive and deterministic.
- Preserve fail-closed semantics when CI fast-gate is mismatched or boundary rules drift.

