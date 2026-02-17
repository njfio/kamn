# Plan — #4260 Deterministic Finality Evidence Verifier + Promotion Reason Mapping

Status: Implemented

## Approach

1. Add policy-side reason-code resolver for stable promotion decision mapping.
2. Add `check-evidence-convergence` command validating:
   - lane report schema/status/final decision,
   - policy schema/status/final decision/reason mapping,
   - policy source report linkage integrity.
3. Emit convergence report schema/taxonomy markers and fail closed on deterministic reasons.
4. Wire convergence checker into contract lane and expose markers in output/json.

## Affected Surfaces

- `scripts/runtime/libp2p_convergence_process_isolated_live_contract.py`
- `scripts/runtime/check_libp2p_convergence_process_isolated_live_evidence_convergence.sh`
- `scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh`

## Risks and Mitigations

- Risk: inconsistent reason-code ordering across reruns.
  Mitigation: use ordered deterministic resolver and explicit reason taxonomy CSV constants.
