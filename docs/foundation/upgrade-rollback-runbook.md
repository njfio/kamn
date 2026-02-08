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

## Watchdog Incident Response Flow
1. Confirm incident attestation severity and fingerprint.
2. Capture incident evidence payload before mitigation.
3. Execute containment actions from the approved response tier.
4. Run rollback procedure when severity is `critical`.
5. Record closure summary with deterministic incident fields.

Capture incident evidence payload with expected/observed state hash, quorum sample, and censorship delivery ratios before rollback action.

## Fast and Cost-Effective Watchdog Validation Lane
Run from repository root:

```bash
cargo test -p kamn-core --test runtime_watchdog_attestation_docs
cargo test -p kamn-core --test watchdog_node_docs
cargo test -p kamn-core --test upgrade_rollback_runbook_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test upgrade_rollback_runbook_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
