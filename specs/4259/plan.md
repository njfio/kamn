# Plan — #4259 Red Tests for Finality Evidence Convergence

Status: Implemented

## Approach

1. Add a dedicated test script for convergence checker GO/NO-GO paths.
2. Extend libp2p contract-lane test to require convergence markers and tamper rejection.
3. Execute tests before implementation to capture Red evidence.

## Affected Surfaces

- `scripts/runtime/test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh`
- `scripts/runtime/test_validate_libp2p_convergence_process_isolated_live_contract_lane.sh`

## Risks and Mitigations

- Risk: false positives from brittle output matching.
  Mitigation: assert deterministic markers/reason codes only.
