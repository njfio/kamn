use super::super::*;

pub(super) async fn handle_post_route(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Option<Response> {
    if context.parsed_request.path == ROUTE_MESSAGES_RELAY {
        return Some(relay_message(state, context).await);
    }
    if context.parsed_request.path == ROUTE_MESSAGES_SEND {
        return Some(send_message(state, context).await);
    }
    None
}

async fn relay_message(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let relay_payload = match parse_relay_ingest_payload(context.parsed_request.body.as_str()) {
        Ok(payload) => payload,
        Err(error) => return bad_request(error),
    };
    let result = state.message_store.lock().await.upsert_relayed_message(
        relay_payload.message_id.as_str(),
        relay_payload.sender_did.as_deref(),
        relay_payload.recipient_did.as_str(),
        relay_payload.body.as_str(),
    );
    match result {
        Ok(payload) => contract_json(202, &payload),
        Err(error) => persistence_error("service api relay persistence failed", error),
    }
}

async fn send_message(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
) -> Response {
    let channel_id = channel_id_from_context(context);
    let recipient_did = match recipient_did_from_context(context) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let sender_did = sender_did_from_context(context);
    let result =
        create_message_for_context(state, context, sender_did, &recipient_did, &channel_id).await;
    match result {
        Ok(payload) => publish_message_side_effects(
            state,
            context,
            &payload,
            sender_did,
            recipient_did.as_deref(),
            channel_id.as_deref(),
        ),
        Err(error) => persistence_error("service api message persistence failed", error),
    }
}

fn channel_id_from_context(context: &ServiceApiRequestContext) -> Option<String> {
    extract_channel_id_from_payload(context.parsed_request.body.as_str())
}

fn recipient_did_from_context(
    context: &ServiceApiRequestContext,
) -> Result<Option<String>, Response> {
    extract_canonical_recipient_did_from_payload(context.parsed_request.body.as_str())
        .map_err(bad_request)
}

fn sender_did_from_context(context: &ServiceApiRequestContext) -> Option<&str> {
    super::auth::header_value(
        &context.parsed_request.headers,
        REQUEST_AUTH_SENDER_DID_HEADER,
    )
}

async fn create_message_result(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
    channel_id: Option<&str>,
) -> Result<ServiceApiMessageCreateBody, String> {
    state.message_store.lock().await.create_message(
        context.parsed_request.body.as_str(),
        state.snapshot.runtime_mode.as_str(),
        channel_id,
        sender_did,
        recipient_did,
    )
}

async fn create_message_for_context(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    sender_did: Option<&str>,
    recipient_did: &Option<String>,
    channel_id: &Option<String>,
) -> Result<ServiceApiMessageCreateBody, String> {
    create_message_result(
        state,
        context,
        sender_did,
        recipient_did.as_deref(),
        channel_id.as_deref(),
    )
    .await
}

fn publish_message_side_effects(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    payload: &ServiceApiMessageCreateBody,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
    channel_id: Option<&str>,
) -> Response {
    if let Err(response) =
        append_recipient_relay_entry(state, context, payload, sender_did, recipient_did)
    {
        return response;
    }
    publish_message_event(state, payload, sender_did, recipient_did, channel_id);
    success_message_response(payload)
}

fn append_recipient_relay_entry(
    state: &Arc<ServiceApiRuntimeState>,
    context: &ServiceApiRequestContext,
    payload: &ServiceApiMessageCreateBody,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
) -> Result<(), Response> {
    let Some(recipient_did_value) = recipient_did else {
        return Ok(());
    };
    let relay_entry = relay_spool_entry(context, payload, sender_did, recipient_did_value);
    super::append_service_api_relay_spool_entry(state.relay_spool_file.as_deref(), &relay_entry)
        .map_err(relay_spool_error_response)
}

fn relay_spool_entry(
    context: &ServiceApiRequestContext,
    payload: &ServiceApiMessageCreateBody,
    sender_did: Option<&str>,
    recipient_did: &str,
) -> ServiceApiRelaySpoolEntry {
    ServiceApiRelaySpoolEntry {
        message_id: payload.message_id.clone(),
        sender_did: sender_did.map(str::to_owned),
        recipient_did: recipient_did.to_owned(),
        body: context.parsed_request.body.clone(),
        queued_at_unix: queued_at_unix(),
    }
}

fn queued_at_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn relay_spool_error_response(error: impl std::fmt::Display) -> Response {
    super::payload::json_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        REASON_CODE_STATE_PERSISTENCE_FAILED,
        format!("service api relay spool append failed: {error}").as_str(),
    )
}

fn publish_message_event(
    state: &Arc<ServiceApiRuntimeState>,
    payload: &ServiceApiMessageCreateBody,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
    channel_id: Option<&str>,
) {
    state.websocket_events.publish_message_created_event(
        payload,
        sender_did,
        recipient_did,
        channel_id,
    );
}

fn success_message_response(payload: &ServiceApiMessageCreateBody) -> Response {
    contract_json(202, payload)
}
