# Plan — Issue #4828

## Approach

- Execute the smallest deterministic implementation slice that satisfies all ACs.
- Add/extend red->green regression coverage before broad migration changes.
- Preserve CI smoke/runtime budget boundaries and deterministic reason-taxonomy outputs.

## Affected Modules

- To be finalized during implementation from concrete file-level impact.

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/manifests.
  Mitigation: phased rollout with deterministic regression suites and compatibility checks.
- Risk: CI cost increase.
  Mitigation: enforce bounded smoke limits and local-heavy opt-in boundaries.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Emit stable key=value outputs and reason taxonomy/version markers on policy paths.

## ADR

- Required only if this issue introduces protocol/dependency/architecture decisions.
