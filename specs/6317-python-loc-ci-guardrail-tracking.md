# Spec: Issue #6317 - track Python LOC in shell-rust CI guardrail output

## Objective

Extend the fast-gate shell/rust ratio guardrail output surface to include Python LOC telemetry
(metrics only), so CI artifacts expose Python growth alongside existing shell/rust metrics.

## Inputs/Outputs

- Inputs:
  - `scripts/ci/check_shell_rust_ratio_guardrail.py` current shell/rust-only metric collection.
  - `crates/kamn-core/tests/ci_shell_rust_ratio_guardrail_contract.rs` current contract coverage.
  - existing fast-gate invocation of the guardrail wrapper in `.github/workflows/ci-fast-gate.yml`.
- Outputs:
  - guardrail stdout markers include `python_line_total` and `tracked_python_file_count`.
  - guardrail JSON metrics include `python_line_total` and `tracked_python_file_count`.
  - contract tests assert Python markers exist on pass/warn/fail/validation paths.

## Boundaries/Non-goals

- In scope:
  - metrics enrichment in guardrail checker and test contracts.
  - no change to wrapper topology or workflow invocation shape.
- Out of scope:
  - introducing python-to-rust threshold enforcement.
  - changing final decision policy beyond existing shell/rust ratio behavior.
  - unrelated CI telemetry tool refactors.

## Failure modes

- FM-1: Python metrics absent from stdout or JSON payload despite successful guardrail run.
- FM-2: guardrail contract lane does not catch missing Python telemetry fields.
- FM-3: existing shell/rust decision semantics regress (pass/warn/fail drift).
- FM-4: malformed threshold/error paths omit Python telemetry placeholders, causing schema drift.

## Acceptance criteria (testable booleans)

- AC-1: guardrail emits `python_line_total=<value>` and `tracked_python_file_count=<value>` markers.
- AC-2: guardrail JSON `metrics` object includes numeric Python totals/count on success paths.
- AC-3: fail-closed/error marker paths include Python placeholders as `unknown` where metrics are unavailable.
- AC-4: existing shell/rust ratio final decision behavior remains unchanged for identical thresholds.
- AC-5: `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract` passes.

## Files to touch

- `specs/6317-python-loc-ci-guardrail-tracking.md`
- `scripts/ci/check_shell_rust_ratio_guardrail.py`
- `crates/kamn-core/tests/ci_shell_rust_ratio_guardrail_contract.rs`

## Error semantics

- Preserve current reason taxonomy and reason codes.
- Preserve fail-closed behavior for invalid arguments, missing threshold keys, parse failures,
  output-write failures, and invalid rust totals.
- For fail paths with unknown computed metrics, Python telemetry markers emit `unknown` consistently
  with existing shell/rust unknown marker behavior.

## Test plan

- RED:
  - add contract assertions requiring Python telemetry markers/JSON fields in guardrail outputs.
  - run `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract` and confirm failure.
- GREEN:
  - implement Python metric collection in guardrail script.
  - include Python metrics in success and failure marker/report payloads.
- REFACTOR:
  - centralize guardrail metric unknown defaults to avoid duplication drift.
- INTEGRATION:
  - run guardrail contract lane and fast-gate wiring contract lane for non-regression.

