# Issue #5006 Tasks

- Issue: #5006
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M3 red conformance tests for blind-index determinism,
      owner scoping, metadata filtering, and fail-closed invalid-input behavior
      in child task `#5019`.
- [x] T2 (Green): implement M3 blind-index and metadata-search contracts and exports
      in child task `#5019`.
- [x] T3 (Refactor): tighten normalization, filtering, and deterministic ordering
      helper behavior in child task `#5019`.
- [x] T4 (Regression): run `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m3_blind_index_search`, and
      `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for
      child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story
      issue with linked child evidence.
