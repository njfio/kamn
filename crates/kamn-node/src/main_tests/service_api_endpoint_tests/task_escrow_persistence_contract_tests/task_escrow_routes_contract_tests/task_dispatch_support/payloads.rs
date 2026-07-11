pub(super) fn canonical_dispatch_body(provider: &str, task_type: &str, key: &str) -> String {
    serde_json::json!({
        "provider_did": provider,
        "transaction_id": format!("transaction-{key}"),
        "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "idempotency_key": key,
        "task_type": task_type,
        "description": "canonical dispatch task",
    })
    .to_string()
}
