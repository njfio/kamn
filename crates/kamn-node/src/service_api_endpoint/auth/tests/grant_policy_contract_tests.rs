use super::super::*;

#[test]
fn unit_transaction_authorization_rejects_reserved_task_path_ids() {
    for (method, path) in [
        ("GET", "/v1/tasks/create"),
        ("POST", "/v1/tasks/create/accept"),
        ("POST", "/v1/tasks/create/complete"),
    ] {
        assert!(
            resolve_target(method, path).is_none(),
            "reserved route: {path}"
        );
    }
}

#[test]
fn unit_transaction_authorization_rejects_reserved_escrow_path_ids() {
    assert!(resolve_target("POST", "/v1/escrow/fund/release").is_none());
}

fn resolve_target(method: &str, path: &str) -> Option<TransactionAuthorizationTarget> {
    let request = ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body: String::new(),
        headers: BTreeMap::from([(
            REQUEST_AUTH_SENDER_DID_HEADER.to_owned(),
            "kamn:did:agent:grant-policy".to_owned(),
        )]),
    };
    resolve_transaction_authorization_target(&request).expect("target resolution should succeed")
}
