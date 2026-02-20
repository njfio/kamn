# Issue #5402 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5402` daemon test decomposition markers.
- run that exact test before docs updates to capture RED evidence.

2. Extract live-postgres fixtures/helpers:
- create `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs`.
- move live-postgres constants/models/helper projection/extraction functions from `daemon_tests.rs` into the new submodule.
- wire with `mod live_postgres_fixtures;` and `use live_postgres_fixtures::*;` while preserving existing root test function names.

3. Update docs markers:
- add a `#5402` section in `docs/ops/configuration.md` describing phase-1 decomposition markers and command path stability.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include daemon-tests decomposition phase-1 hardening.

5. Verify line-count objective:
- capture post-change `wc -l` evidence for `daemon_tests.rs` and ensure <= 4300 lines.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs` (new)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: extraction breaks helper visibility across tests.
  - Mitigation: keep test function definitions in root module and import extracted helpers via `use live_postgres_fixtures::*;`.
- Risk: accidental test-path changes.
  - Mitigation: avoid moving `#[test]` function definitions; validate unchanged exact command paths.
- Risk: partial decomposition without measurable effect.
  - Mitigation: enforce post-change `daemon_tests.rs` line-count target in conformance checks.

## Interfaces / Contracts
- No production API changes.
- New internal test-module contract: live-postgres fixture/topology/hash helpers are sourced from a dedicated daemon-tests submodule while root test command paths remain stable.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
