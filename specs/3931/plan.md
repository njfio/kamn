# Issue #3931 Plan

- Issue: #3931
- Status: Implemented

## Approach
1. Add a red contract test that asserts required fuzz package files, corpus seed files, and CI strategy markers exist.
2. Implement `fuzz/` package with two libFuzzer targets:
   - message envelope parsing/validation
   - DID parsing
3. Add deterministic seed corpus fixtures and replay metadata for both targets.
4. Add CI strategy contract markers for cargo-fuzz CI-smoke/local-heavy command boundaries.
5. Run targeted contract tests, fmt/clippy, and shell guardrails.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Fuzz package layout drift can silently break target discoverability.
  - Documentation may drift from command surface unless pinned by tests.
- Mitigations:
  - Add fail-closed contract tests on file paths and marker strings.
  - Keep heavy fuzz lanes documented as local-heavy only.

## Interface Contract
- Adds fuzz harness package and documentation contracts only.
- No production API/wire-format/runtime behavior changes.

## ADR
- Not required (test and governance surface addition).
