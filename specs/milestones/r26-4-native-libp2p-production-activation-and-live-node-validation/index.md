# Milestone: R26.4 Native libp2p Production Activation and Live-Node Validation

- Milestone ID: r26-4-native-libp2p-production-activation-and-live-node-validation
- GitHub Milestone: #32
- Milestone Title: R26.4 Native libp2p production activation and live-node validation
- Scope Status: In progress

## Objective
Close R26.4 by making native libp2p activation, rollback control, interoperability validation, and go/no-go governance deterministic and auditable.

## Issue Map
- Epic #3876: R26.4 activate native libp2p runtime path and live-node validation closure
  - Story #3877: activate native libp2p runtime profiles with deterministic rollback controls
    - Task #3878: implement native runtime profile selector guardrails and fail-closed validation
      - Subtask #3879: add native-fallback profile compatibility validation checks
      - Subtask #3880: add invalid-profile fail-closed reason taxonomy regression checks
    - Task #3881: deliver native transport cutover-rollback policy lane with CI governance
      - Subtask #3882: implement native cutover-rollback evidence bundle lane
      - Subtask #3883: add policy checker and CI exclusion tests for native cutover lane
  - Story #3884: prove live-node native libp2p and kolme interoperability readiness
    - Task #3885: implement local-heavy native libp2p plus kolme interoperability matrix lane
      - Subtask #3886: add triadic native libp2p plus kolme interoperability scenario runner
      - Subtask #3887: add interoperability artifact schema and marker policy checks
    - Task #3889: enforce activation go-no-go budget and documentation parity contracts
      - Subtask #3891: add activation readiness and budget marker checks to go-no-go policy
      - Subtask #3893: add docs-contract and milestone-summary parity checks for activation closure

## Contract Signals
- Native profile activation and rollback paths fail closed with deterministic reason taxonomy.
- Interoperability matrix emits deterministic schema and marker evidence for promotion decisions.
- Go/no-go checks enforce readiness markers, budget ceilings, and documentation parity contracts.

## Verification Surface
- cargo test -p kamn-node
- runtime native lane checks
- CI policy/exclusion checks for native lanes
- deploy go/no-go checks
