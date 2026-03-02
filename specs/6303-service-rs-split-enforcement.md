# Spec: Issue #6303 - kamn-sdk service.rs extraction threshold + first split tranche

## Objective

Introduce deterministic CI extraction-threshold governance for `crates/kamn-sdk/src/service.rs`
and execute an initial no-behavior-change split tranche that moves a coherent helper block out of
the monolith.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-sdk/src/service.rs` current line count and helper layout.
  - extraction-threshold fixture and optional tracked exception metadata.
  - CI fast-gate workflow wiring and shell-surface policy contract tests.
- Outputs:
  - new checker output for service extraction thresholds with `GO|WARN|NO-GO`.
  - fast-gate workflow step and artifact for service extraction-threshold report.
  - reduced `service.rs` size via helper extraction into dedicated module file.

## Boundaries/Non-goals

- In scope:
  - service.rs line-count threshold checker (script + wrapper + fixtures + tests).
  - fast-gate wiring + wiring contract updates.
  - first split tranche moving response/json helper logic into dedicated module.
- Out of scope:
  - full decomposition of all service client responsibilities.
  - `kamn-node/src/main.rs` threshold policy changes.
  - transport protocol or API contract changes.

## Failure Modes

- FM-1: checker does not fail closed for missing/invalid threshold/exception metadata.
- FM-2: workflow is not wired to execute/report service extraction threshold checks.
- FM-3: split tranche introduces behavioral regressions in service response parsing.
- FM-4: docs/contracts drift and CI strategy parity checks fail.

## Acceptance Criteria

- AC-1: new checker exists for `kamn-sdk/src/service.rs` and emits deterministic
  `policy_decision=GO|WARN|NO-GO` plus reason-code markers.
- AC-2: fast-gate workflow runs checker and uploads report artifact.
- AC-3: threshold fixture and tracked exception metadata are present with valid schemas.
- AC-4: one helper tranche is extracted from `service.rs` into a dedicated module with tests green.
- AC-5: strategy/wiring contract tests include the new checker markers and remain green.

## Files To Touch

- `scripts/ci/check_kamn_sdk_service_rs_extraction_threshold.py`
- `scripts/ci/check_kamn_sdk_service_rs_extraction_threshold.sh`
- `scripts/ci/test_check_kamn_sdk_service_rs_extraction_threshold.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/test_ci_strategy_contract.sh`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `scripts/lib/exec_registry.json`
- `.github/workflows/ci-fast-gate.yml`
- `fixtures/ci/kamn_sdk_service_rs_extraction_thresholds.json`
- `.ci/kamn_sdk_service_rs_extraction_threshold_exception.json`
- `docs/ci/strategy.md`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_response.rs`

## Error Semantics

- Checker follows existing extraction-threshold fail-closed model:
  - invalid/missing source or threshold metadata produces `NO-GO`.
  - fail-threshold overflow without valid exception produces `NO-GO`.
  - warn-threshold overflow produces `WARN`.
- Split tranche must preserve existing `SdkError` mapping semantics for response parsing and JSON
  field extraction helpers.

## Test Plan

- RED:
  - add service.rs extraction-threshold checker contract test using existing common harness.
  - add workflow/strategy contract markers for service checker wiring.
- GREEN:
  - implement checker + wrapper + fixtures + workflow wiring.
  - extract response/json helper tranche from `service.rs` to module.
- REFACTOR:
  - keep checker aligned with shared extraction-threshold reason taxonomy style.
  - ensure module extraction keeps names/self-documenting boundaries clear.
- Verification:
  - `bash scripts/ci/test_check_kamn_sdk_service_rs_extraction_threshold.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `cargo test -p kamn-sdk`
