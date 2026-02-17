# Milestone Index: R27.31

Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance
GitHub Milestone: https://github.com/njfio/kamn/milestone/65
Status: In Progress

## Objective

Converge deterministic real OS signal handling, graceful daemon drain lifecycle contracts,
streaming observability endpoint schema governance, and runtime module-boundary parity checks with
low-cost CI smoke plus local-heavy deep validation lanes.

## Scope

- Signal-safe daemon shutdown trigger and phase transition contracts.
- Streaming observability endpoint schema and reason-taxonomy contracts.
- Runtime decomposition parity checker contracts and governance.
- CI smoke/local-heavy boundary enforcement for the above.

## Issue Hierarchy

- Epic: #4323
- Stories: #4324, #4325
- Tasks: #4326, #4327, #4328, #4329
- Subtasks: #4330, #4331, #4332, #4333, #4334, #4335, #4336, #4337

## Exit Signals

- OS signal capture paths are deterministic and fail closed under invalid transition conditions.
- Shutdown-phase and observability schema drift are caught by contract tests.
- Runtime decomposition checker detects boundary drift in CI smoke.
- Documentation and runbook markers remain synchronized with checker reason taxonomies.
