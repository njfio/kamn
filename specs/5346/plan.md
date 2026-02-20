# Issue #5346 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5346` taxonomy-bridge markers.
- run that exact test before docs updates to capture RED evidence.

2. Add taxonomy-bridge contracts in daemon tests:
- define canonical runtime taxonomy constant and bridge helper assertions in `daemon_tests`.
- add focused functional + integration tests that assert deterministic runtime taxonomy markers for applied/deferred scenarios across repeated runs.

3. Extend docs marker contracts:
- add a `#5346` subsection in `docs/ops/configuration.md` with runtime/matrix taxonomy bridge markers and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include taxonomy-bridge hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: bridge marker duplication across runtime and matrix sections drifts.
  - Mitigation: dedicated docs-contract assertions for exact bridge marker strings.
- Risk: live env gating makes integration checks conditional.
  - Mitigation: preserve env-gated short-circuit behavior while still asserting deterministic contracts when env is present.
- Risk: taxonomy assertion expansion introduces brittle parsing.
  - Mitigation: reuse existing JSON field extraction helpers with minimal helper additions.

## Interfaces / Contracts
- No production API changes.
- New contract: runtime phase6 taxonomy-version markers and matrix taxonomy markers must remain bridge-consistent and deterministic.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
