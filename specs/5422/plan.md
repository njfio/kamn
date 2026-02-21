# Issue #5422 Plan — Multi-Host + Batched Coherence Delivery

## Approach
1. Add explicit bundle-map markers in docs/spec surfaces as the governance anchor.
2. Introduce distributed-lane fixture/orchestration definitions in daemon live-postgres test helpers.
3. Implement one representative multi-host lane per bundle, expanding incrementally while preserving selector compatibility.
4. Add fail-closed guards for missing multi-host prerequisites and deterministic reason codes.

## Affected Modules (expected)
- `crates/kamn-node/src/main_tests/daemon_tests/*.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`
- potentially `scripts/runtime/*` for distributed-lane execution wrappers/policies

## Risks / Mitigations
- Risk: distributed lane increases CI/runtime cost.
  - Mitigation: separate fast/deep profiles with explicit budget guardrails.
- Risk: coherence bundle map drifts from implementation.
  - Mitigation: fail-closed docs/contract tests for bundle markers.
- Risk: selector instability after lane expansion.
  - Mitigation: keep include-based module routing under existing `main_tests::daemon_tests` namespace.

## Interfaces / Contracts
- Keep daemon test selector prefix stable.
- Publish and enforce six bundle IDs (`B-01..B-06`) in docs/spec contracts.
- Fail-closed reason-taxonomy markers for distributed-lane prerequisites and execution decisions.

## Validation Strategy
- Red: add failing bundle-map/docs contract assertions first.
- Green: implement bundle markers + distributed-lane scaffolding until targeted conformance passes.
- Verify: targeted integration/conformance/regression checks plus fmt/clippy.
