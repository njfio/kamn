# Upgrade Rollback and Post-Upgrade Verification Runbook (Issues #170, #171)

This runbook defines deterministic rollback triggers, rollback execution steps, and post-upgrade verification checks for KAMN operators.
For semantic versioning policy and compatibility rules, see `docs/foundation/versioning-compatibility-matrix.md`.

## Rollback Triggers
- State migration checksum mismatch.
- Quorum health degraded below threshold.
- Critical post-upgrade verification failure.
- Invalid runtime wiring detected for processor, listener, or approver roles.

## Rollback Procedure
1. Freeze upgrade pipeline and block new proposals.
2. Confirm rollback trigger evidence.
3. Restore last known-good state snapshot.
4. Rehydrate node roles with pinned release image.
5. Re-run migration consistency checks.
6. Resume controlled traffic.

## Post-Upgrade Verification Checklist
- App-state schema version matches expected target.
- Processor, Listener, and Approver roles report healthy wiring.
- No stale state-hash acceptance detected.
- Invariant smoke harness completes with no critical failures.
- Governance and operator permission paths remain authorized.

## Failure Simulation Scenarios
- Schema mismatch rollback drill:
  - Inject incompatible migration metadata and verify trigger fires before write-path enablement.
- Partial node upgrade divergence drill:
  - Upgrade processor only, keep listeners/approvers pinned, and verify rollback restores quorum consistency.
- Quorum degradation during upgrade drill:
  - Remove one approver during staged rollout and validate rollback gate blocks finalization.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test upgrade_rollback_runbook_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
