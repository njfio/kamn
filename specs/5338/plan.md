# Issue #5338 Plan

## Implementation Approach
1. Add docs-contract red gate:
- introduce a new test in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` that asserts a dedicated issue-`#5338` marker section exists in `docs/ops/configuration.md`.
- run the exact test before docs updates to capture RED evidence.

2. Implement env-gated validation slice in daemon tests:
- add an integration test to `crates/kamn-node/src/main_tests/daemon_tests.rs` that resolves `KAMN_TEST_POSTGRES_URL`/`DATABASE_URL`.
- when URL is present, connect using `DataLayerPgExecutionAdapter`, apply migrations, then execute daemon runtime and assert deterministic Phase-6 markers in rendered report output.
- when URL is absent, exit early (deterministic env-gated behavior).

3. Add ops marker contract section:
- add a new section in `docs/ops/configuration.md` for PostgreSQL live + daemon runtime slice contracts, including taxonomy markers and exact commands for the two conformance lanes.

4. Follow-up governance marker:
- update `docs/review/gaps-and-issues-r45.md` next-milestone narrative to reference initiated tracking via `#5338`.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: live PostgreSQL lane availability differs across environments.
  - Mitigation: strict env-gating; no live URL means deterministic no-op path.
- Risk: docs marker drift after future command/name changes.
  - Mitigation: enforce marker presence via docs-contract test.
- Risk: cross-test env mutation races in daemon tests.
  - Mitigation: reuse existing daemon/log env lock discipline in test harness.

## Interfaces / Contracts
- No production API changes.
- New test contract: live-postgres + daemon-runtime validation slice command and marker set must remain present and stable in ops docs.

## ADR
- Not required: no new dependency, protocol change, or architectural runtime behavior modification.
