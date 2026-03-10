use super::super::*;
use super::support::{assert_preflight_matrix_case, PREFLIGHT_MATRIX};

#[test]
fn integration_kolme_live_signer_preflight_quorum_profile_matrix_paths() {
    // Regression: #3957
    let _lock = lock_signer_env_guard();
    for case in PREFLIGHT_MATRIX {
        assert_preflight_matrix_case(case);
    }
}
