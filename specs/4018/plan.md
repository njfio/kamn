# Issue #4018 Plan

- Issue: #4018
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Implementation Approach

1. RED first:
- add new policy checker contract tests in `crates/kamn-core/tests/sqlite_crash_restart_local_heavy_policy_contract.rs`.
- add docs assertions for strategy and runbook parity markers.
- run targeted test to confirm failure before checker implementation.

2. GREEN implementation:
- add `scripts/runtime/sqlite_crash_restart_local_heavy_policy_contract.py`.
- add wrapper `scripts/runtime/check_sqlite_crash_restart_local_heavy_policy.sh` and registry entry.
- add shell checker regression script `scripts/runtime/test_check_sqlite_crash_restart_local_heavy_policy.sh`.
- update `docs/ci/strategy.md` and `docs/deploy/kolme_devnet_ops.md` marker sections.
- update docs contract tests in `ci_strategy_docs.rs` and `kolme_devnet_ops_docs.rs`.
- wire checker test into `scripts/ci/test_ci_tools.sh` fast/full command surfaces.

3. VERIFY:
- run targeted rust + shell contract tests.
- run `cargo fmt --check` and scoped `cargo clippy`.

## Affected Modules

- `specs/4018/spec.md`
- `specs/4018/plan.md`
- `specs/4018/tasks.md`
- `scripts/runtime/sqlite_crash_restart_local_heavy_policy_contract.py` (new)
- `scripts/runtime/check_sqlite_crash_restart_local_heavy_policy.sh` (new)
- `scripts/runtime/test_check_sqlite_crash_restart_local_heavy_policy.sh` (new)
- `scripts/lib/exec_registry.json`
- `scripts/ci/test_ci_tools.sh`
- `crates/kamn-core/tests/sqlite_crash_restart_local_heavy_policy_contract.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `docs/ci/strategy.md`
- `docs/deploy/kolme_devnet_ops.md`

## Risks and Mitigations

- Risk: policy reason-code drift across script/docs/tests.
  - Mitigation: single-source constants in checker + exact docs/test parity assertions.
- Risk: checker pass path does not cover profile-specific contracts.
  - Mitigation: enforce profile contract checks for `restart|corruption|combined`.
- Risk: CI tool surface drift.
  - Mitigation: wire checker shell test into both fast/full `scripts/ci/test_ci_tools.sh` blocks.

## Interface / Contract Markers

- policy report schema:
  - `kamn.runtime.sqlite-crash-restart-local-heavy-policy-report.v1`
- policy reason taxonomy:
  - `kamn.runtime.sqlite-crash-restart-local-heavy-policy-reason-taxonomy.v1`
- policy reason codes CSV:
  - `sqlite_crash_restart_policy_required_field_missing,sqlite_crash_restart_policy_marker_mismatch,sqlite_crash_restart_policy_reason_taxonomy_mismatch,sqlite_crash_restart_policy_profile_contract_mismatch,sqlite_crash_restart_policy_runbook_marker_parity_mismatch,sqlite_crash_restart_policy_strategy_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_restart_policy_expected_decision_mismatch,sqlite_crash_restart_policy_violation`

## ADR

- Not required (contract checker + docs/tests only; no dependency/protocol addition).
