# Plan: Issue #6244 - Restore CI Fast-Gate and E2E Live Lanes

## Approach

1. Reproduce failures locally with scoped commands matching failing CI steps.
2. Fix strict clippy regressions by removing unnecessary ownership conversions.
3. Fix workspace license policy failures by restoring required crate metadata.
4. Re-run targeted CI-equivalent checks for clippy and license policy.
5. Reproduce and fix E2E CLI smoke live scenario failures, then re-run lane-equivalent command.

## Affected Modules

- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-agent-lib/src/{auth.rs,envelope.rs,identity.rs}`
- `crates/kamn-bridges/Cargo.toml`
- `crates/kamn-crypto/Cargo.toml`
- `crates/kamn-data-layer/Cargo.toml`
- E2E lane-related runtime/harness modules (to be narrowed after local reproduction)

## Risks and Mitigations

- Risk: clippy fixes alter behavior in parsing paths.
  - Mitigation: only remove redundant allocations; keep parsing and tests unchanged.
- Risk: license metadata change could drift from workspace policy expectation.
  - Mitigation: align with existing workspace crate metadata conventions.
- Risk: E2E smoke failure may involve external runtime timing and non-determinism.
  - Mitigation: run CI-equivalent command path locally and patch only contract-breaking change.

## Verification

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p kamn-core --test workspace_license_policy_contract`
- E2E CLI smoke lane-equivalent command from workflow step for run `22508619101`
