# Plan — #4268

Status: Reviewed

## Approach

- Extend websocket policy checker to emit deterministic promotion-decision reason mapping markers.
- Add websocket evidence convergence subcommand validating report/policy linkage and mapping parity.
- Integrate convergence checker into websocket contract lane output.
- Add regression tests for missing-link and tamper rejection.
- Update planning/release docs and docs-contract tests for new convergence markers.

## Risks and Mitigations

- Risk: marker drift between checker outputs and docs.
  - Mitigation: explicit docs-contract assertions in Rust tests and shell lane tests.
- Risk: convergence behavior diverges from policy reason mapping semantics.
  - Mitigation: shared deterministic mapping function reused by policy and convergence subcommands.

## Interfaces and Contracts

- `scripts/runtime/service_api_websocket_live_contract.py check-evidence-convergence`
- `scripts/runtime/check_service_api_websocket_live_evidence_convergence.sh`
- Websocket lane output markers:
  - `service_api_websocket_evidence_convergence_status`
  - `promotion_decision_reason_mapping_status`
  - convergence taxonomy/version + reason codes csv
