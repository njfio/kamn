# Issue #5481 Spec - Selector-Bundle Fingerprint Bootstrap Report Parity

- Status: Accepted
- Issue: #5481
- Parent: #3812
- Milestone: R50.6 Live-postgres selector fingerprint bootstrap report integration

## Problem Statement
Daemon completion telemetry now exposes selector-bundle fingerprint markers, but bootstrap report output (JSON/text) omits the same marker, preventing report consumers from validating selector-bundle integrity from report surfaces alone.

## Scope
In scope:
- Add selector-bundle fingerprint field to daemon bootstrap report data model.
- Render the field in JSON and text bootstrap report outputs.
- Extend runtime contract tests to assert report-field parity and value coherence.

Out of scope:
- New protocol/wire format changes.
- Multi-host distributed lane expansion.

## Acceptance Criteria
- AC-1: Bootstrap report JSON output includes `daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint`.
- AC-2: Bootstrap report text output includes `daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint`.
- AC-3: Report marker value matches canonical runtime selector-bundle fingerprint and existing marker tests remain green.

## Conformance Cases
- C-01 (Functional, AC-1): daemon JSON report includes selector-bundle fingerprint marker with canonical value.
- C-02 (Functional, AC-2): daemon text report includes selector-bundle fingerprint marker with canonical value.
- C-03 (Conformance/Regression, AC-3): runtime marker contract test passes while asserting report/telemetry coherence.

## Success Metrics / Observable Signals
- Bootstrap reports carry deterministic selector-bundle fingerprint marker.
- Runtime contract tests enforce marker parity across JSON/text/log surfaces.
- Existing daemon marker contracts remain stable.
