# R27.24 Async API Concurrency and Admission-Backpressure Governance

## Milestone Summary

R27.24 closes async API concurrency, overload admission/backpressure lineage, and CI smoke governance boundaries so fast-gate remains low-cost while heavy lanes stay explicit local opt-in.

## Scope

In scope:
- async API concurrency budget contracts and deterministic queue/in-flight markers,
- admission/backpressure taxonomy and runbook parity governance,
- admission/backpressure evidence convergence checker contracts,
- low-cost CI smoke drift checker and heavy load-lane exclusion enforcement.

Out of scope:
- API architecture redesign,
- autoscaling or global traffic-policy changes.

## Active Chain

- Epic: `#4218`
- Stories:
  - `#4219` deterministic async API concurrency/queue-budget contracts
  - `#4220` admission-backpressure evidence lineage and promotion gate integrity
- Task chain under `#4220`:
  - `#4223` admission-backpressure evidence convergence checker
  - `#4224` CI smoke governance for marker drift and heavy-lane exclusion

## Governance Markers

- CI smoke boundary remains `low` cost with deterministic `GO/NO-GO` drift decisions.
- Heavy load-lane commands remain excluded from `ci-fast-gate` and ci-tools fast mode.
