# Tasks — Issue #4808

- [x] T1 (Red): Child tasks introduced failing migration tests before implementation.
  - `#4813` and `#4814` subtasks followed RED-first contract tests.
- [x] T2 (Green): Deliver story objectives via child tasks.
  - `#4813` completed (wave/matrix parameterization).
  - `#4814` completed (shared harness + JSON helper rollout).
- [x] T3 (Refactor): Replace duplicate script boilerplate with shared runners/helpers.
  - Wrapper/matrix family consolidation landed in `#4813`.
  - Harness + JSON helper migrations landed in `#4814`.
- [x] T4 (Verify): Validate conformance and full regression.
  - Story conformance mapped through merged child evidence and `bash scripts/ci/test_ci_tools.sh` passes.
