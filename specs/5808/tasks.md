# Tasks: Issue #5808 - S-02 Live Scenario Activation

- Issue: #5808
- Spec: `specs/5808/spec.md`
- Plan: `specs/5808/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (RED/Conformance): add failing `S-02` fail-closed tests in SDK-direct, CLI-scripted, and MCP-agent driver modules.
- [x] T2 (GREEN/Implementation): wire `S-02` into per-driver live-bound mapping and implement dedicated probe/runner helpers.
- [x] T3 (Regression): run targeted and full `kamn-e2e-harness` test lanes to verify no behavior regression.
- [x] T4 (Closeout): update milestone index and issue lifecycle markers to completed state.

## Tier Mapping
- Unit: helper validation paths for S-02 probe parsing and field checks.
- Functional: `execute("S-02")` fail-closed conformance behavior per driver.
- Conformance: spec case C-01..C-05 coverage and lifecycle artifact parity.
- Integration: full `cargo test -p kamn-e2e-harness` lane.
- Regression: existing `S-01/S-04/S-06` and disabled-live behavior remain green.
- Performance: N/A (no performance-sensitive algorithm changes).
