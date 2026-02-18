# Tasks — Issue #4819

- [x] T1 (Red): add failing contract test `scripts/framework/test_common_shell_library.sh` and capture pre-implementation failure (`scripts/lib/common.sh: No such file or directory`).
- [x] T2 (Green): implement `scripts/lib/common.sh` and migrate pilot scripts/dispatcher to source shared helpers.
- [x] T3 (Refactor): remove duplicated `extract_value`/`assert_eq`/local `ROOT_DIR` from pilot scripts.
- [x] T4 (Verify): run focused regression suite and capture evidence.

## Verification Evidence

- `bash scripts/framework/test_common_shell_library.sh`
- `bash scripts/deploy/test_generate_dr_evidence_bundle.sh`
- `bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh`
- `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh`
