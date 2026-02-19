# Issue #3785 Plan

- Issue: #3785
- Status: Implemented

## Approach
1. Add a red docs-contract assertion in `crates/kamn-core/tests/ci_strategy_docs.rs` for unified API-observability local-heavy CI exclusion command markers.
2. Update `docs/ci/strategy.md` to include unified local-heavy CI exclusion policy command and fail-closed run-mode exclusion rule in heavy-integration contract section.
3. Re-run targeted docs-contract and CI exclusion tests to confirm green.
4. Run lint and shell-surface guardrails and capture closure metrics.

## Risks and Mitigations
- Risk level: low
- Risks:
  - Strategy docs drift from command-surface enforcement script coverage.
  - Local-heavy lane exclusion markers become partial/inconsistent over time.
- Mitigations:
  - Fail-closed docs-contract assertions for explicit command strings.
  - Keep changes constrained to docs and Rust docs-contract tests (no new shell scripts).

## Interface Contract
- Documentation and docs-contract enforcement only.
- No runtime API, protocol, wire-format, or dependency changes.

## ADR
- Not required.
