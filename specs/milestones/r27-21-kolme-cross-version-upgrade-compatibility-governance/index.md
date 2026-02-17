# R27.21 Kolme Cross-version Upgrade Compatibility Governance

## Milestone Summary

R27.21 closes deterministic KAMN-Kolme compatibility governance, upgrade-rehearsal lineage
integrity, and low-cost CI smoke controls that keep heavy rehearsal lanes opt-in.

## Scope

In scope:
- deterministic compatibility marker matrix and fail-closed reason-taxonomy checks,
- upgrade-rehearsal evidence lineage validation across checker, policy, and closure docs,
- low-cost CI smoke drift checks for compatibility markers and local-heavy lane exclusion.

Out of scope:
- always-on heavy upgrade rehearsal execution in PR fast-gate,
- release-orchestration architecture redesign.

## Active Chain

- Epic: `#4173`
- Story chain:
  - `#4174` deterministic KAMN-Kolme compatibility marker contracts
  - `#4175` upgrade rehearsal lineage and promotion gate compatibility integrity
- Task chain:
  - `#4176` compatibility matrix checks for runtime schema/failure-taxonomy markers
  - `#4177` mismatch reason-taxonomy enforcement and runbook parity
  - `#4178` upgrade rehearsal evidence lineage validation
  - `#4179` low-cost CI smoke governance for upgrade compatibility drift
- Subtasks under `#4179`:
  - `#4186` CI smoke checker for compatibility-rehearsal drift + heavy-lane exclusion
  - `#4187` docs + drift-contract closure synchronization

## Governance Markers

- Compatibility smoke checker decisions stay deterministic with explicit reason-taxonomy outputs.
- Heavy rehearsal/run-mode commands remain excluded from fast-gate and ci-tools fast mode.
- Upgrade compatibility and rehearsal markers remain fail-closed for drift or tamper.
