# Issue #5340 Plan

## Implementation Approach
1. Add docs-contract red gate:
- introduce a new docs-contract test requiring gate/deferred markers for issue `#5340`.
- execute that exact test before docs updates to capture RED evidence.

2. Harden daemon live-postgres test helpers:
- add explicit reason-code constants and a helper that resolves env-gate decision + URL.
- add deterministic env-unset regression and env-precedence unit tests under existing env lock discipline.

3. Add deferred-path live slice:
- add an env-gated integration test that performs live adapter connect+migrations and executes daemon with shutdown signal configuration.
- assert deferred Phase-6 reason marker in report output.

4. Extend docs and review narrative:
- add gate/deferred marker contract lines and exact commands in `docs/ops/configuration.md`.
- update `docs/review/gaps-and-issues-r45.md` next-frontier wording to reflect this hardening increment while preserving broader follow-up scope.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: env mutation races in daemon tests.
  - Mitigation: reuse shared lock + env guards already used by daemon tests.
- Risk: live DB not configured in local/CI contexts.
  - Mitigation: keep integration tests env-gated with deterministic unset-path tests independent from live DB availability.
- Risk: docs markers drift in future edits.
  - Mitigation: enforce marker presence with dedicated docs-contract test.

## Interfaces / Contracts
- No production API changes.
- Test/docs contract adds deterministic env-gate reason and deferred-path marker requirements for the live-postgres daemon validation slice.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
