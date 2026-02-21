# R50.6 Live-postgres Selector Fingerprint Bootstrap Report Integration

## Scope
- Surface selector-bundle fingerprint marker in daemon bootstrap report JSON/text output.
- Keep runtime completion telemetry marker unchanged.
- Extend runtime report contract tests for marker parity/coherence.

## Linked Issues
- #5481

## Exit Criteria
- Bootstrap report includes selector-bundle fingerprint field in JSON/text renderings.
- Runtime contract tests validate marker coherence with canonical selector rows.
- Issue #5481 merged with green CI.
