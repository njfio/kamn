# Issue #5230 Plan

- Issue: #5230
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Add typed DID conversion seams to wave-C runtime/proof/reputation boundaries while preserving existing public payload shapes.
2. Standardize invalid DID errors to structured deterministic markers (`field`, `reason_code`, `detail`) across scoped modules.
3. Update representative cross-module flows and tests to assert deterministic invalid-DID contracts without changing behavior on valid inputs.
4. Keep shell surface unchanged and verify ratio guardrail remains green.

## Affected Modules
- `runtime_peer_coordination.rs`: peer/runtime identity boundary validation.
- `runtime_phase_coordination.rs`: phase orchestration identity boundary validation.
- `group_channel_crypto.rs`: sender/member DID identity conversion seams.
- `message_proof_anchoring.rs`: proof submitter/anchor DID boundary validation.
- `reputation_signals.rs`: candidate/source DID conversion seams and deterministic invalid DID errors.
- `reputation_state.rs`: state persistence DID boundary validation.
- `instruction_verify.rs`: instruction signer/agent DID validation seams.
- `agent_upgrade_workflow.rs`: proposer/reviewer/executor DID boundaries.
- `upgrade_orchestration.rs`: orchestration actor DID boundaries.

## Risks and Mitigations
- Risk: broad enum shape changes can break many test assertions.
  - Mitigation: add RED tests first for deterministic invalid-DID markers and migrate module-by-module.
- Risk: API surface guardrails (public item delta) fail after adding new methods/types.
  - Mitigation: keep wrappers private where possible and avoid unnecessary new public helpers.
- Risk: behavior drift in runtime/proof orchestration.
  - Mitigation: rerun targeted integration suites after each module cluster.

## Interfaces / Contracts
- Invalid DID errors include:
  - `field`
  - `reason_code`
  - `detail`
- Conversion seams use typed wrappers via `TryFrom<&Raw>` where external string contracts must remain unchanged.
