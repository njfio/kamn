# Issue #5364 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5364` topology-scope markers.
- run that exact test before docs updates to capture RED evidence.

2. Add topology-scope contracts in daemon tests:
- add explicit topology schema constants and topology-labeled fingerprint helper(s).
- add functional + integration tests for topology structure and repeated-run stability.

3. Extend docs marker contracts:
- add a `#5364` subsection in `docs/ops/configuration.md` with topology schema/version/ids markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include topology-scope hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: topology assertions overfit to incidental ordering.
  - Mitigation: enforce deterministic sorting and explicit canonical topology id set.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: accidental expansion into real multi-host runtime behavior.
  - Mitigation: keep scope to test/docs contracts only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology-labeled parallel lane fingerprints must preserve canonical schema and remain deterministic across repeated runs.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
