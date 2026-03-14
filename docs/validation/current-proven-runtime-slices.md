# Current Proven Runtime Slices

This index summarizes the runtime behavior that current `main` proves today through executable runbooks and regression contracts.

## What Is Currently Proven
- Service-api vertical slice: `docs/validation/working-vertical-slice.md`
  - proves one service-api vertical slice with two identities, encrypted delivery evidence, one task lifecycle transition, and audit export output
- TCP signed-relay vertical slice: `docs/validation/sdk-tcp-vertical-slice.md`
  - proves one SDK TCP signed-relay vertical slice with signed handshake acceptance, one successful relay, and fail-closed replay or forged-handshake rejection
- durable cross-node relay slice: `docs/validation/durable-cross-node-relay-slice.md`
  - proves durable sender spool enqueue, fail-closed pending spool preservation, later successful relay projection, and recipient-visible delivered state across fresh boot
- restart persistence slice: `docs/validation/restart-persistence-slice.md`
  - proves restart persistence across message state, task and escrow state, directory state, and relayed or delivered status continuity
- escrow settlement slice: `docs/validation/escrow-settlement-slice.md`
  - proves service-api escrow lifecycle persistence through fund, release, and restart-visible released state
- bridge finality slice: `docs/validation/bridge-finality-slice.md`
  - proves deterministic receipt-finality normalization and persisted forwarded bridge state

## What Remains Unproven
- broad production readiness
- consensus or multi-node finality
- live chain-backed bridge finality or external settlement
- global fault tolerance under arbitrary partitions

## How To Use This Index
Start here when evaluating what KAMN actually demonstrates on current `main`. Then follow the linked runbooks to the exact test commands and operator evidence for each slice.
