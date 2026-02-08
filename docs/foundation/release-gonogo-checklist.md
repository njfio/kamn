# Release Go/No-Go Checklist and Dry-Run Workflow (Issues #172, #173)

This checklist defines deterministic release gates and auditable evidence requirements before approving a protocol or runtime upgrade.
For semantic versioning policy and compatibility rules, see `docs/foundation/versioning-compatibility-matrix.md`.

## Preflight Gates
- Migration plan reviewed and signed.
- Compatibility matrix validated.
- CI fast gate and deferred deep lane both green.
- Rollback runbook version pinned.
- Release candidate artifact digest verified.

## Deterministic Dry-Run Workflow
1. Create release candidate tag.
2. Rehearse migration on staging snapshot.
3. Execute bounded smoke and invariant suites.
4. Capture and sign dry-run evidence bundle.
5. Validate rollback precheck against last known-good snapshot.

## Go/No-Go Evidence Template
- Release candidate:
- Schema target version:
- Runtime image digest:
- Dry-run timestamp:
- CI evidence links:
- Rollback trigger status:
- Rollback precheck result: PASS
- Final decision: GO | NO-GO
- Approver signatures:

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
