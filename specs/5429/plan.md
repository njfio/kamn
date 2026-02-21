# Issue #5429 Plan — Production Report Telemetry Extension

## Approach
1. Identify source telemetry values already available in daemon runtime projection/report pipeline.
2. Add fields to report model and report builder mapping.
3. Extend text/json renderers with stable field names.
4. Add/adjust tests first (RED) for missing fields, then implement (GREEN).

## Affected Modules
- `crates/kamn-node/src/main.rs` (report struct fields)
- `crates/kamn-node/src/report_builder.rs`
- `crates/kamn-node/src/report_render.rs`
- `crates/kamn-node/src/main_tests/report_tests.rs` and/or related daemon report tests
- `specs/5429/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: report-schema drift between text/json surfaces.
  - Mitigation: assert both renderers expose matching key set in tests.
- Risk: optional-value nullability mismatch.
  - Mitigation: use deterministic Option handling and explicit expected values in tests.

## Interfaces / Contracts
- New field names must be stable and follow existing `daemon_*` telemetry naming pattern.
- JSON/text parity for all new fields is mandatory.

## Validation Strategy
- RED: report tests expecting new fields fail before implementation.
- GREEN: field wiring + renderer updates make tests pass.
- VERIFY: targeted tests + fmt + clippy.
