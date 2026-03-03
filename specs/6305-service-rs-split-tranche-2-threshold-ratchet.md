# Spec: Issue #6305 - service.rs split tranche 2 + extraction threshold ratchet

## Objective

Reduce `crates/kamn-sdk/src/service.rs` root-module size with a no-behavior-change split tranche and
ratchet extraction thresholds so CI enforces continued decomposition.

## Inputs/Outputs

- Inputs:
  - current `crates/kamn-sdk/src/service.rs` layout (production code + inline `#[cfg(test)]` module).
  - existing extraction-threshold checker and threshold fixture.
  - current `kamn-sdk` unit/integration tests.
- Outputs:
  - extracted test module file for `service.rs` test-only helpers and tests.
  - reduced root `service.rs` line count under new warn threshold.
  - tightened threshold fixture values with checker returning `policy_decision=GO`.
  - deterministic budget contract test that fails when root `service.rs` regresses past tranche cap.

## Boundaries/Non-goals

- In scope:
  - `service.rs` root size reduction via test-module extraction only.
  - threshold ratchet for `kamn-sdk service.rs` extraction checker.
  - budget contract test for `service.rs` line count.
- Out of scope:
  - route-handler decomposition or HTTP behavior changes.
  - changes to public `ServiceApiClient` method signatures.
  - changes to `kamn-node` extraction thresholds.

## Failure Modes

- FM-1: root `service.rs` remains above tightened threshold after extraction.
- FM-2: extracted tests are not wired, silently reducing executed coverage.
- FM-3: threshold ratchet causes false-positive checker failures due mismatched contract values.
- FM-4: extraction introduces behavior drift in request/response/auth code paths.

## Acceptance Criteria

- AC-1: `crates/kamn-sdk/src/service.rs` line count is `<= 1700` (boolean pass/fail).
- AC-2: all tests previously defined in inline `service.rs` `#[cfg(test)]` module compile and run from
  extracted module file (boolean pass/fail via `cargo test -p kamn-sdk`).
- AC-3: `fixtures/ci/kamn_sdk_service_rs_extraction_thresholds.json` enforces warn/fail thresholds that
  keep current `service.rs` in `GO` state (boolean pass/fail via checker output).
- AC-4: a dedicated contract test fails closed when `service.rs` exceeds the tranche max line budget
  (boolean pass/fail).
- AC-5: no `ServiceApiClient` runtime behavior changes are introduced (boolean pass/fail via existing
  `kamn-sdk` test suite).

## Files To Touch

- `specs/6305-service-rs-split-tranche-2-threshold-ratchet.md`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_tests.rs`
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `fixtures/ci/kamn_sdk_service_rs_extraction_thresholds.json`

## Error Semantics

- No changes to SDK runtime error mapping (`SdkError`) for transport/auth/validation paths.
- Extraction-threshold checker semantics stay fail-closed:
  - threshold overflow above fail threshold without exception remains `NO-GO`.
  - warn-zone remains `WARN`.
  - below warn remains `GO`.

## Test Plan

- RED:
  - add `service.rs` line-budget contract test with `MAX_LINES=1700` (expected to fail at current
    1863 LOC).
- GREEN:
  - extract inline `#[cfg(test)]` module from `service.rs` into `service_tests.rs` and wire with
    `#[cfg(test)] #[path = "service_tests.rs"] mod tests;`.
  - run `cargo test -p kamn-sdk` to confirm behavior parity.
- REFACTOR:
  - clean imports and test-module boundaries after extraction.
  - keep root `service.rs` focused on production code.
- INTEGRATION:
  - ratchet threshold fixture values and run checker in live mode against repository source.
  - verify checker decision and reason codes match expected policy state.

## Phase 6 Integration Evidence (to fill at close)

- `bash scripts/ci/check_kamn_sdk_service_rs_extraction_threshold.sh --output-json /tmp/kamn-sdk-service-rs-extraction-threshold-report.json`
- `cargo test -p kamn-sdk`

Observed results:
- checker reported `policy_decision=GO`, `line_count=1340`, `warn_line_count=1700`,
  `fail_line_count=1800`, `reason_codes=none`.
- `bash scripts/ci/test_check_kamn_sdk_service_rs_extraction_threshold.sh` passed.
- `cargo test -p kamn-sdk` passed.
