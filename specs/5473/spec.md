# Issue #5473 Spec - Live-Postgres Selector Row Runtime Telemetry Marker

- Status: Implemented
- Issue: #5473
- Parent: #3812
- Milestone: R50.2 Live-postgres selector row telemetry integration

## Problem Statement
Daemon runtime completion telemetry omits the canonical live-postgres multi-host selector-row bundle, limiting runtime observability and reducing direct drift diagnostics between runtime markers and selector contracts.

## Scope
In scope:
- Emit `multi_host_execution_bundle_selector_rows_csv` in daemon runtime completion telemetry.
- Derive marker value from production selector-row source in runtime orchestration.
- Add/extend daemon contract tests to validate marker presence and coherence.

Out of scope:
- NodeBootstrapReport schema additions.
- New multi-host execution behavior changes.

## Acceptance Criteria
- AC-1: Daemon runtime completion telemetry includes `multi_host_execution_bundle_selector_rows_csv` marker.
- AC-2: Telemetry marker value is derived from production selector-row source and aligns with selector prefix/row-count markers.
- AC-3: Contract tests detect selector-row telemetry drift and pass when runtime telemetry remains coherent.

## Conformance Cases
- C-01 (Functional, AC-1): daemon complete log event contains `multi_host_execution_bundle_selector_rows_csv` marker.
- C-02 (Unit, AC-2): selector-row CSV marker parses to rows whose count equals `multi_host_execution_bundle_row_count` and each row uses selector prefix.
- C-03 (Conformance, AC-3): targeted daemon contract tests fail before marker implementation and pass after implementation.

## Success Metrics / Observable Signals
- Runtime telemetry carries full selector-row bundle context.
- Drift between selector rows and row-count/prefix becomes directly test-detectable.
- Existing phase6/convergence/multi-host marker regression remains green.
