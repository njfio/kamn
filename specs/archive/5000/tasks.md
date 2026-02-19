# Issue #5000 Tasks

- Issue: #5000
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): reproduce docs-contract failure caused by deleted-wrapper references.
  - Evidence:
    - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` initially failed on 4 deterministic assertions tied to deleted wrapper references.
- [x] T2 (Green): replace deleted wrapper command references with manifest-runner equivalents in docs + docs-contract assertions.
  - Evidence:
    - Updated `docs/ci/strategy.md` and `crates/kamn-core/tests/ci_strategy_docs.rs` to manifest-runner command surface.
- [x] T3 (Regression): rerun docs-contract suite and confirm deterministic pass.
  - Evidence:
    - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` -> `37 passed; 0 failed`.
- [x] T4 (Wave-2 Archive): execute archive migration for implemented-but-unarchived specs.
  - Evidence:
    - Extended `scripts/ci/archive_completed_specs.py` to accept both `- Status: Implemented` and `- Status: `Implemented`` formats.
    - Applied wave-2 command over detected candidates -> `requested_issue_count=44`, `archived_issue_count=44`, `status=ok`.
- [x] T5 (Policy Verify): run archive policy checker/tests and resolve any parity/index drift.
  - Evidence:
    - Extended `scripts/ci/check_spec_archive_policy.sh` to accept both implemented status formats.
    - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-post-wave2.json` -> `status=ok`.
    - `bash scripts/ci/test_check_spec_archive_policy.sh` -> passed.
- [x] T6 (Lifecycle + DoD): set issue/spec status to Implemented and record shell-surface delta markers in PR/issue closure.
  - Evidence:
    - `specs/5000/spec.md`, `specs/5000/plan.md`, `specs/5000/tasks.md` synchronized to Implemented.
