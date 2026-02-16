# R27.30 Milestone: Async API Runtime, Networked Peer Transport, and Durable Block-Pipeline Governance

- Milestone: `R27.30 Async API runtime, networked peer transport, and durable block-pipeline governance`
- GitHub milestone number: `64`
- Scope: close P1/P2 hardening work for async runtime API behavior, peer transport integrity, and durable block commit governance under deterministic fail-closed contracts.

## Linked Hierarchy
- Epic: `#4308`
- Stories: `#4309`, `#4310`
- Tasks/Subtasks: tracked under the story hierarchy and mapped to per-issue specs in `specs/<issue-id>/`.

## Definition of Done
- Every linked issue has accepted spec artifacts (`spec.md`, `plan.md`, `tasks.md`).
- ACs and conformance cases are mapped to tests with passing evidence.
- Runtime/docs updates are included in the same PR when behavior or ownership changes.

## Verification Baseline
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- Targeted issue conformance selectors in each issue spec
