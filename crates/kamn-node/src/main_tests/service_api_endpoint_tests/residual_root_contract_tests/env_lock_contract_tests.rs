use super::*;

#[test]
fn regression_service_api_env_lock_recovers_from_signer_lock_poison() {
    let _ = std::panic::catch_unwind(|| {
        let _lock = lock_signer_env_guard();
        panic!("intentional signer env lock poison");
    });
    let _env = acquire_service_api_test_env();
}
