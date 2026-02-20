# Issue #5342 Plan

## Implementation Approach
1. Add docs-contract red gate:
- add a new docs-contract test that requires scenario-matrix/stability markers for issue `#5342`.
- run that test before docs updates to capture RED evidence.

2. Add daemon scenario-matrix coverage:
- add a deterministic env matrix test for unset, preferred, fallback, and trimmed env states.
- add a live-gated integration test that runs applied/deferred daemon scenarios repeatedly and asserts stable reason codes.

3. Extend ops marker contracts:
- add a `#5342` subsection in `docs/ops/configuration.md` for matrix/stability contracts and exact validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include this matrix-stability increment.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: live DB unavailable in some environments.
  - Mitigation: live scenario remains env-gated; matrix gate test remains deterministic without DB.
- Risk: env mutation races across daemon tests.
  - Mitigation: continue using shared env lock and guard helpers.
- Risk: marker drift over time.
  - Mitigation: add dedicated docs-contract enforcement.

## Interfaces / Contracts
- No production API changes.
- New contract: scenario matrix and repeated-run stability markers for live-postgres daemon validation slice must remain documented and test-covered.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
