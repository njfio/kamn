# Plan — Issue #4199

## Approach
- Extend `scripts/runtime/test_run_go_no_go_gate_lane.sh` with targeted tamper fixtures:
  - manifest variant missing `local_full_runtime_convergence`
  - manifest variant with tampered success marker for `local_full_runtime_convergence`
- Assert deterministic fail-closed reason codes for both cases.
- Update `docs/planning/kolme-devnet-ops.md` with convergence integrity/fail-closed marker guidance.
- Extend docs-contract tests in `crates/kamn-core/tests/kolme_devnet_ops_docs.rs` for new markers.

## Affected Modules
- `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- `docs/planning/kolme-devnet-ops.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks and Mitigations
- Risk: Added tests could be flaky if they depend on external runtime state.
  - Mitigation: keep tamper tests manifest-file based and deterministic in dry-run mode.
- Risk: Reason-code naming drift across future convergence implementation.
  - Mitigation: lock exact reason-code assertions in regression tests now.

## Interfaces/Contracts
- No new CLI interfaces.
- Test contract expansion only; fail-closed reason-code expectations become explicit.

## ADR
- Not required (test/docs-only scope).
