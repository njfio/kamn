# Issue #5218 Plan

- Issue: #5218
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Add a Rust wave-2 migration suite (`shell_test_surface_migration_wave2.rs`) that asserts behavior parity for:
   - Makefile command-surface contract checks.
   - Makefile `make -n` execution contract checks.
   - `run_cargo_test_with_quarantine.sh` dry-run + invalid-command guards.
2. Rewire `scripts/ci/test_ci_tools.sh` (fast and full branches) to execute the Rust wave-2 suite and remove calls to deleted shell wrappers.
3. Update docs/contracts that currently reference removed wrapper scripts so they require the Rust wave-2 lane command.
4. Delete the 3 migrated shell wrappers from `scripts/ci`.
5. Add wave-2 entries to superseded script deletion inventory and run stale-reference + deletion-manifest checks in the same change set.
6. Verify targeted conformance commands, then update issue process log and PR evidence.

## Wave-2 Inventory
- `scripts/ci/test_makefile_command_surface_contract.sh`
- `scripts/ci/test_makefile_execution_contract.sh`
- `scripts/ci/test_run_cargo_test_with_quarantine.sh`

## Risks and Mitigations
- Risk: docs/contract tests still reference deleted wrappers.
  - Mitigation: update `docs/ci/strategy.md`, `README.md`, and enforcing shell contracts in same commit.
- Risk: parity drift in moved shell assertions.
  - Mitigation: Rust suite reproduces original assertion semantics and executes shell runner under test for quarantine behavior.
- Risk: deletion inventory drift.
  - Mitigation: add explicit manifest entries and run `test_check_superseded_script_deletion_manifest.sh` plus stale-reference checker before PR.

## Interfaces / Contracts
- New Rust migration lane command:
  - `cargo test -p kamn-core --test shell_test_surface_migration_wave2`
- CI entrypoint contract:
  - `scripts/ci/test_ci_tools.sh` must run wave-1 + wave-2 migration suites.
- Deletion governance:
  - `fixtures/ci/superseded_script_deletion_manifest.json` includes wave-2 wrapper deletions with deterministic reason codes.
