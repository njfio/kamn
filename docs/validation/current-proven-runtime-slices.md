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
- live task-lifecycle slice: `docs/validation/live-task-lifecycle-slice.md`
  - proves one bounded live task-lifecycle execution lane through `sdk-direct` S-04
- live task-lifecycle MCP parity slice: `docs/validation/live-task-lifecycle-mcp-parity-slice.md`
  - proves one bounded live task-lifecycle execution lane through MCP-agent S-04 parity
- escrow settlement slice: `docs/validation/escrow-settlement-slice.md`
  - proves service-api escrow lifecycle persistence through fund, release, and restart-visible released state
- bridge finality slice: `docs/validation/bridge-finality-slice.md`
  - proves deterministic receipt-finality normalization and persisted forwarded bridge state
- solana receipt finality slice: `docs/validation/solana-receipt-finality-slice.md`
  - proves bounded Solana receipt-finality normalization on the public core surface
- solana devnet bridge smoke slice: `docs/validation/solana-devnet-bridge-smoke-slice.md`
  - proves a bounded Solana devnet-addressed bridge smoke path on the public bridge surface
- live solana devnet proof slice: `docs/validation/live-solana-devnet-proof-slice.md`
  - proves bounded live Solana devnet JSON-RPC evidence normalized through the public receipt surface
- live solana bridge dispatch slice: `docs/validation/live-solana-bridge-dispatch-slice.md`
  - proves a bounded live Solana-backed bridge evidence lane on the service-api path
- live solana bridge websocket slice: `docs/validation/live-solana-bridge-websocket-slice.md`
  - proves the live Solana-backed bridge evidence lane reaches the websocket event stream
- live escrow settlement slice: `docs/validation/live-escrow-settlement-slice.md`
  - proves one bounded live escrow settlement execution lane through external-execution `sdk-direct` S-05
- live escrow CLI parity slice: `docs/validation/live-escrow-cli-parity-slice.md`
  - proves one bounded live escrow settlement execution lane through CLI-scripted S-05 parity
- live escrow MCP parity slice: `docs/validation/live-escrow-mcp-parity-slice.md`
  - proves one bounded live escrow settlement execution lane through MCP-agent S-05 parity

## What Remains Unproven
- broad production readiness
- consensus or multi-node finality
- live chain-backed bridge finality or external settlement
- live Solana settlement over the KAMN bridge path
- broad multi-driver live economic-settlement parity
- global fault tolerance under arbitrary partitions

## How To Use This Index
Start here when evaluating what KAMN actually demonstrates on current `main`. Then follow the linked runbooks to the exact test commands and operator evidence for each slice.
