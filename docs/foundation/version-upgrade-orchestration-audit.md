# Version Upgrade Orchestration and Governance Audit Views (Issues #192 / #193)

This document captures the first implementation slice for governed chain-version upgrade orchestration.

## Scope Delivered
- Added `crates/kamn-core/src/upgrade_orchestration.rs` with:
  - `VersionUpgradeOrchestrator` for:
    - `propose_upgrade(...)`
    - `approve_upgrade(...)`
    - `mark_governance_status(...)`
    - `activate_upgrade(...)`
    - `rollback_upgrade(...)`
  - governance-aware proposal models:
    - `UpgradeProposalRecord`
    - `UpgradeProposalState`
  - audit projection surfaces:
    - `UpgradeAuditEvent`
    - `UpgradeAuditEventKind`
    - `VersionUpgradeAuditView`
  - typed errors via `UpgradeOrchestrationError`.
- Added integration and regression tests in `crates/kamn-core/tests/upgrade_orchestration.rs`.

## Upgrade Gating Rules
- Upgrade proposals require:
  - valid proposer DID.
  - non-empty proposal id.
  - valid semantic version target (`vX.Y.Z`).
  - target version strictly greater than current version.
  - positive required quorum.
- Activation requires:
  - governance status `Approved`.
  - sufficient unique validator approvals to satisfy required quorum.
  - positive activation timestamp.

## Governance Audit View Rules
- All proposal/approval/governance-status/activation/rollback actions emit audit events.
- Audit view is deterministic and includes:
  - current chain version
  - ordered event history with event kind, actor DID, and timestamp
- Rollback is allowed only for previously activated proposals.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test upgrade_orchestration --test upgrade_orchestration_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
