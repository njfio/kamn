# Tasks: Issue #5812 - Live S-02 Matrix Execution Evidence

- Issue: #5812
- Spec: `specs/5812/spec.md`
- Plan: `specs/5812/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing docs-contract assertions for new `S-02` live evidence artifact markers and milestone issue linkage.
- [x] T2 (GREEN/Docs): author `docs/research/e2e-live-testing-prd-r55-live-s02-execution-evidence.md` with deterministic marker set and command/result evidence.
- [x] T3 (Integration): execute live harness matrix (`sdk-direct`, `cli-scripted`, `mcp-tau`) with scenarios `S-01,S-02,S-04,S-06` and capture outputs under `.tmp/5812-live/`.
- [x] T4 (Lifecycle): update milestone index delivery slice + issue references and finalize spec/task statuses.
- [x] T5 (Regression): run docs-contract lane, full harness tests, fmt, and clippy gates.
- [x] T6 (Guardrail): preserve non-regression spec-volume cap by removing one legacy archived top-level spec pointer (`specs/3910`).

## Tier Mapping
- Unit: N/A (docs/lifecycle scope).
- Functional: docs marker presence assertions.
- Conformance: lifecycle + milestone linkage checks.
- Integration: live harness matrix runs with real local API runtime.
- Regression: harness crate regression + format/lint gates.
- Performance: N/A (no hotspot/runtime algorithm changes).
