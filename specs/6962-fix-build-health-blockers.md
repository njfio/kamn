# 6962-fix-build-health-blockers

## Objective
Repair the current `kamn-core` build-health blockers by fixing the malformed M7 timeseries telemetry test module and removing the two production `.unwrap()` calls from M1 Merkle batch assembly without weakening lint policy or changing Merkle semantics.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/data_layer_m1/batch.rs`
  - `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs`
  - current workspace lint policy in `Cargo.toml`
- Outputs:
  - syntax-correct M7 telemetry test module
  - M1 batch assembly without production `.unwrap()` calls
  - focused regression tests for the repaired M1 batch behavior
  - updated spec evidence for the build repair

## Boundaries/Non-goals
- Do not weaken `unwrap_used = "deny"`.
- Do not refactor unrelated data-layer modules.
- Do not perform broad warning cleanup.
- Do not change Merkle proof semantics or telemetry projection behavior beyond restoring intended current behavior.

## Failure modes
- M7 telemetry test module still fails to parse.
- M1 batch assembly changes Merkle root or proof behavior.
- Removing `.unwrap()` introduces silent fallback instead of structured failure.
- `cargo test --no-run` remains blocked by this scope.
- `cargo clippy` still flags production unwrap usage in `data_layer_m1/batch.rs`.

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs` parses correctly.
- [ ] `crates/kamn-core/src/data_layer_m1/batch.rs` contains no production `.unwrap()` calls.
- [ ] M1 batch assembly still returns the same Merkle root/proof behavior under focused tests.
- [ ] `cargo test --no-run` succeeds for the repaired scope or any unrelated blocker is recorded explicitly.
- [ ] `cargo clippy` no longer fails on the M1 batch unwrap usage.

## Files to touch
- `crates/kamn-core/src/data_layer_m1/batch.rs`
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs`
- optional focused regression tests under existing `kamn-core` test targets
- `specs/6962-fix-build-health-blockers.md`

## Error semantics
- Preserve `DataLayerM1Error::EmptyBatch` for empty input.
- Any new helper must continue returning typed errors; no silent defaults.
- Test failures must remain hard-fail and explicit.

## Test plan
- Red tests proving the M7 test module is malformed and the M1 batch source still contains `.unwrap()`.
- Green focused tests for M1 batch assembly/proof behavior.
- `cargo test -p kamn-core --test build_health_blockers_contract -- --nocapture`.
- `cargo test --no-run` for the affected crate/workspace scope.
- `cargo clippy` or equivalent targeted check proving the unwrap usage is gone.
