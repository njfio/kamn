# Bridge Quorum Runtime Contracts (Issues #364 / #367 / #370 / #373)

This document captures deterministic bridge quorum runtime contracts for listener-confirmed inbound events and approver-gated outbound actions.

## Scope Delivered
- Added bridge quorum docs contract baseline for runtime listener and approver workflows.
- Added deterministic guard text for replay, malformed payload, and under-quorum rejection paths.
- Added low-cost validation lane commands for bridge quorum docs assertions.
- Added listener quorum runtime model references:
  - `ListenerAttestation`
  - `ListenerQuorumInput`
  - `ListenerQuorumEvaluator`
  - `ListenerQuorumDecision`
  - `ListenerQuorumError`
  - `evaluate_daemon_listener_quorum`

## Listener Quorum Workflow Rules
- Inbound bridge decisions require canonical listener attestation normalization before threshold evaluation.
- Listener confirmation sets are deduplicated and sorted deterministically for stable outcomes.
- Duplicate listener attestation replay is rejected.
- Replayed or out-of-order listener event sequences are rejected.
- Listener quorum outcomes emit deterministic accept/reject decision evidence in runtime reporting.

## Approver Quorum Workflow Rules
- Outbound bridge actions require canonical approver attestation set evaluation against configured quorum thresholds.
- Outbound under-quorum approval sets are rejected.
- Malformed approver attestation payload is rejected.
- Outbound authorization decisions emit deterministic typed rejection reasons when quorum requirements are unmet.

## Test Coverage Mapping
- Unit: N/A (docs-focused scope).
- Functional: listener and approver workflow rule assertions.
- Integration: command-to-contract mapping assertions for bridge quorum validation lanes.
- Regression:
  - duplicate listener attestation replay rejection (`Regression: #371`)
  - listener event sequence replay rejection (`Regression: #371`)
  - malformed approver payload rejection (`Regression: #372`)
  - outbound under-quorum rejection (`Regression: #372`)

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test bridge_quorum_runtime_docs
cargo test -p kamn-core --test runtime_network_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```
