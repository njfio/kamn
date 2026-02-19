# Issue #3940 Plan

- Issue: #3940
- Status: In Progress
- Spec: `specs/3940/spec.md`

## Implementation Approach
1. Add a regression test that demonstrates the current production-source extractor bug with top-level `#[cfg(test)]` attributes.
2. Replace the extractor with a test-item skipping parser that removes `#[cfg(test)]`-guarded items while preserving remaining production lines.
3. Extend panic-path regression file coverage to API/observability runtime modules.
4. Update runtime watchdog docs with #3940 expect-callsite retirement guard mapping.
5. Run mapped test/lint/format checks.

## Affected Modules
- `crates/kamn-node/src/cli_tests.rs`
- `docs/foundation/runtime-watchdog-attestation.md`

## Risks and Mitigations
- Risk: source parser could skip too much or too little around cfg(test) blocks.
  - Mitigation: add direct regression fixture for top-level cfg(test) import plus production lines.
- Risk: broader file coverage introduces noisy failures.
  - Mitigation: include only startup/API/observability modules and keep deterministic assertions.

## Contracts and Interfaces
- Panic-path guard contract: listed runtime sources must have no production `expect(` / `panic!(` / `unreachable!(`.
- Source extraction contract: `#[cfg(test)]`-scoped items are excluded; production items remain scan-visible.

## Verification Strategy
- RED: run new extractor regression before parser fix.
- GREEN: update parser and panic-path coverage list.
- REGRESSION: run startup panic-path test, docs contract test, fmt, and strict clippy.
