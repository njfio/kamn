# R27.41 TLS Governance and Anti-Flake Reliability Contracts

- Milestone: `R27.41 TLS-governance completion and anti-flake merge-gate reliability contracts`
- Scope:
  - Finish deterministic TLS governance evidence and fail-closed release gating.
  - Enforce anti-flake merge-gate reliability with bounded CI smoke and explicit local-heavy drills.

## Child Work
- Epic: `#4473`
- Stories:
  - `#4474` deterministic TLS certificate-policy and evidence convergence
  - `#4475` anti-flake merge-gate reliability and rerun-policy convergence
- Tasks:
  - `#4476` TLS certificate-policy checker deterministic failure taxonomy
  - `#4477` TLS evidence bundle completeness/freshness convergence in release gate checks
  - `#4478` anti-flake classifier and deterministic rerun-policy checker contracts
  - `#4479` merge-gate reliability evidence convergence and CI smoke/local-heavy boundary governance

## Exit Criteria
- Deterministic fail-closed reasons across TLS and anti-flake release/merge gates.
- CI fast-gate remains bounded and smoke-oriented.
- Local-heavy reliability drills remain explicit opt-in and excluded from fast-gate.
- Specs, plans, tasks, tests, and docs are synchronized per issue.
