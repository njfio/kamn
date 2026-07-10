use super::super::super::*;
use super::request_support::SignedRequest;

pub(super) fn provision_request_grant(request: &SignedRequest<'_>) {
    const STATE_FILE_ENV: &str = "KAMN_SERVICE_API_STATE_FILE";
    let Ok(state_file) = std::env::var(STATE_FILE_ENV) else {
        panic!("authorized request requires {STATE_FILE_ENV}");
    };
    let actor_did = test_service_api_sender_did(request.caller_did);
    crate::service_api_endpoint::provision_test_transaction_grant(
        state_file,
        actor_did.as_str(),
        request.method,
        request.path,
        request.body,
    )
    .expect("authorization fixture grant should persist");
}
