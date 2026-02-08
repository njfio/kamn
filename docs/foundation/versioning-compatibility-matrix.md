# Semantic Versioning and Compatibility Matrix (Issues #174, #175)

This document defines the first versioning policy slice for chain protocol, app-state schema, and SDK families, including compatibility expectations for upgrades and downgrades.

## Semantic Versioning Policy
- Chain protocol version follows MAJOR.MINOR.PATCH.
- App-state schema version follows MAJOR.MINOR.PATCH.
- SDK versions (Rust, Python, TypeScript) follow MAJOR.MINOR.PATCH.
- MAJOR increments indicate breaking protocol or schema changes.
- MINOR increments indicate backward-compatible feature additions.
- PATCH increments indicate backward-compatible fixes and hardening.

## Compatibility Matrix
| Chain Protocol | App-State Schema | Node Binary | SDK Family | Upgrade Expectation | Downgrade Expectation |
|---|---|---|---|---|---|
| Same major version | Same major version | Same major version | Same or previous minor | Same major version upgrade: supported with migration plan. | Downgrade within same major may be allowed only with rollback checklist evidence. |
| Next major version | Next major version | Next major version | Mixed major not allowed | Cross-major upgrade: requires governance approval and staged rollout. | Downgrade across major versions: blocked. |

## Support and Deprecation Windows
- Current minor (N) and previous minor (N-1) are supported.
- Anything older than N-1 is deprecated and no-go for new rollouts.
- Security fixes are backported only for supported minors.
- Deprecated minors require explicit governance waiver for continued operation.

## Decision Rules
- Incompatible downgrade decision: NO-GO.
- Any cross-major schema shift requires dry-run evidence before GO.
- Mixed-major node binaries are never GO in production rollout windows.

## Workflow References
- Referenced by governance workflow: docs/foundation/release-gonogo-checklist.md
- Referenced by migration/rollback workflow: docs/foundation/upgrade-rollback-runbook.md

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test versioning_compatibility_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
