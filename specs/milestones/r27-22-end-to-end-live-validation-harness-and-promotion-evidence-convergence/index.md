# R27.22 End-to-end Live Validation Harness and Promotion Evidence Convergence

## Milestone Summary

R27.22 closes deterministic full-stack live validation harness governance, promotion-evidence
convergence integrity, and low-cost CI smoke boundaries that keep heavy run-mode lanes opt-in.

## Scope

In scope:
- deterministic marker taxonomy and fail-closed policy contracts for full-stack harness lanes,
- promotion-evidence convergence checks across harness, gate, and release artifacts,
- low-cost CI smoke drift checks for marker parity and local-heavy exclusions.

Out of scope:
- always-on heavy live-node execution in PR fast-gate,
- external release-orchestration redesign.

## Active Chain

- Epic: `#4188`
- Story chain:
  - `#4189` deterministic full-stack harness marker taxonomy across runtime/transport/consensus
  - `#4190` promotion-evidence convergence across full-stack live harness lanes
- Task chain under `#4190`:
  - `#4193` promotion-evidence convergence checker
  - `#4194` CI smoke governance for marker drift and heavy-lane exclusion
- Subtasks under `#4194`:
  - `#4201` CI smoke checker implementation and enforcement
  - `#4202` docs + drift-contract closure synchronization

## Governance Markers

- CI smoke checker decisions stay deterministic with explicit reason-taxonomy outputs.
- Heavy run-mode/local-heavy commands remain excluded from fast-gate and ci-tools fast mode.
- Promotion evidence remains fail-closed for missing/tampered linked artifact markers.
