# Issue #3876 Spec

- Title: Epic: R26.4 activate native libp2p runtime path and live-node validation closure
- Status: Reviewed
- Priority: P0
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Native libp2p paths are delivered, but production activation and live-node validation still require explicit closure governance: profile activation controls, deterministic rollback evidence, live-node interoperability proofs, and go-no-go budget contracts.

## Scope
In:
- Native runtime profile activation and rollback controls.
- Local-heavy live-node native libp2p plus kolme validation matrix.
- Activation go-no-go and budget/docs parity contracts.

Out:
- Mainnet cutover execution.
- Consensus/protocol redesign.

## Acceptance Criteria
- AC-1:  Native profile activation has deterministic guardrails and rollback evidence.
- AC-2:  Live-node interoperability matrix emits deterministic schema and reason markers.
- AC-3:  Go-no-go budget and docs parity contracts fail closed on drift.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | TBD in implementation task |  Native profile activation has deterministic guardrails and rollback evidence. |
| C-02 | AC-2 | Unit/Functional/Integration/Regression | TBD in implementation task |  Live-node interoperability matrix emits deterministic schema and reason markers. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | TBD in implementation task |  Go-no-go budget and docs parity contracts fail closed on drift. |

## Test Mapping
- To be completed in implementation phase for issue #3876.

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
