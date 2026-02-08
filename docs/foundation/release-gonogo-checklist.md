# Release Go/No-Go Checklist and Dry-Run Workflow (Issues #172, #173)

This checklist defines deterministic release gates and auditable evidence requirements before approving a protocol or runtime upgrade.
For semantic versioning policy and compatibility rules, see `docs/foundation/versioning-compatibility-matrix.md`.

## Preflight Gates
- Migration plan reviewed and signed.
- Compatibility matrix validated.
- Deployment topology preflight passed (`scripts/deploy/preflight_topology.sh`).
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

## Machine-Readable Evidence Bundle Contract (Issue #644)
Go/no-go decisions are captured as machine-readable JSON so release policy checks are auditable and deterministic.

- Generator:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo.json --release-candidate v1.0.0-rc.1 --schema-target-version 1.0.0 --runtime-image-digest sha256:abc123 --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2`
- Policy checker:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo.json`
- Fast contract lane:
  - `bash scripts/deploy/run_gonogo_evidence_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_gonogo_evidence_deep_lane.sh`

## Local Validation
Run from repository root:

```bash
bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh
bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
