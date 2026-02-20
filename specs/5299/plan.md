# Issue #5299 Plan

## Objective
Wire existing M8/M10 Phase-6 scheduler runtime contracts into `kamn-node` daemon execution and surface deterministic runtime markers in report output.

## Approach
1. Add a daemon-phase helper that executes a bounded Phase-6 scheduler runtime cycle against deterministic fixtures.
2. Project Phase-6 runtime result markers (reason taxonomy, reason code, counters) into daemon execution output.
3. Extend report builder/rendering structures with optional Phase-6 runtime fields.
4. Add daemon/runtime tests for applied/deferred/fail-closed paths and report marker assertions.
5. Add ops-doc markers and tracker updates.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/report_builder.rs`
- `crates/kamn-node/src/report_render.rs`
- `crates/kamn-node/src/main_tests/daemon_tests.rs`
- `crates/kamn-node/src/main_tests/report_tests.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`

## Risks and Mitigations
- Risk: introducing nondeterministic runtime fixtures in daemon execution.
  - Mitigation: use fixed owner DID, fixed partition IDs, and stable clock anchors.
- Risk: report-schema drift causing broad test breakage.
  - Mitigation: add optional fields and explicit regression coverage for output markers.
- Risk: fail-closed behavior masked by logging path.
  - Mitigation: assert fail-closed reason markers in targeted regression tests.

## Interfaces / Contracts
- Reuses existing `kamn_core` Phase-6 contracts (`DataLayerM10Phase6SchedulerRuntime` and related types/constants).
- Extends daemon/report projection contracts with additive optional Phase-6 fields.

## ADR
- Not required (no dependency, protocol, or schema changes).
