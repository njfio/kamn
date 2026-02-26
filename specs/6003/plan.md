# Plan: Issue #6003

## Approach
1. Add explicit full-supervisor lane liveness checks in runtime orchestration while daemon execution is in flight.
2. Run daemon execution under a monitor loop that checks lane thread completion (`is_finished`) and aborts with deterministic lane-specific errors if a lane exits early.
3. Reserve one request slot for full-supervisor startup probes when constructing lane configs so monitoring evaluates runtime liveness rather than probe-induced immediate shutdown.
4. Add red-first tests for early service-api lane exit and early observability lane exit, then validate nominal full-mode behavior remains green.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/full_supervisor_and_shutdown_tests.rs`

## Risks / Mitigations
- Risk: monitor loop races with daemon completion boundary and creates false positives.
  Mitigation: check daemon completion first, then lane liveness; only fail when lane exits before daemon completion.
- Risk: startup health probe consumes lane request budget and prematurely completes lanes.
  Mitigation: in full-mode lane config reserve one request slot beyond contract input for probe+runtime lifecycle.
- Risk: contract tests depending on strict lane max-request behavior regress.
  Mitigation: keep CLI-level contract input checks unchanged; only adjust internal full-supervisor lane config.

## Interfaces / Contracts
- Full-mode fail-closed reason codes add lane-liveness variants:
  - `full_supervisor_service_api_lane_liveness_failed`
  - `full_supervisor_observability_lane_liveness_failed`
- No wire-format changes.
- No behavior changes for api-only or observability-only runtime modes.
