# R27.20 Secret material zeroization and signer-rotation governance

## Milestone Summary

R27.20 closes governance gaps around signer secret-material hygiene and custody/rotation
evidence integrity while keeping PR merge-gate coverage low-cost and deterministic.

## Scope

In scope:
- explicit key-material zeroization and fallback-removal contracts with deterministic tests,
- signer custody and rotation evidence parity checks with fail-closed reason-code mapping,
- CI smoke governance for marker-lineage drift with local-heavy execution kept opt-in.

Out of scope:
- always-on local-heavy signer rotation drills in ci-fast-gate,
- multi-environment release orchestration changes.

## Active Chain

- Epic: `#4158`
- Stories:
  - `#4159` enforce private-key zeroization and explicit signer-material requirements
  - `#4160` validate multi-signer rotation evidence and custody policy gate integrity
- Tasks:
  - `#4161` key-material zeroization across env decode and signing lifecycle paths
  - `#4162` remove fallback signer-key paths and enforce explicit deterministic configuration
  - `#4163` multi-signer rotation preflight policy checks and deterministic evidence markers
  - `#4164` low-cost ci smoke governance for custody and signer-rotation marker drift
- Subtasks under `#4164`:
  - `#4171` CI smoke checker for custody/rotation marker lineage and local-heavy exclusions
  - `#4172` docs and drift-contract closure synchronization for zeroization and signer-rotation

## Governance Markers

- Custody and signer-rotation taxonomy markers remain deterministic across checker, CI, and docs.
- Local-heavy rotation/failover commands stay excluded from ci-fast-gate and ci-tools fast mode.
- Drift contracts fail closed on marker omissions, taxonomy mismatch, or boundary-policy leakage.
