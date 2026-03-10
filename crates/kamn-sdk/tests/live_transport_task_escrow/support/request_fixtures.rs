use crate::support::{expected_request, ExpectedRequest};

pub(crate) fn task_and_escrow_requests() -> Vec<ExpectedRequest> {
    vec![
        create_task_request(),
        get_task_status_request("submitted"),
        accept_task_request(),
        get_task_status_request("accepted"),
        submit_artifact_request(),
        get_artifact_status_request("retained", "none"),
        expire_artifact_request(),
        get_artifact_status_request("expired", "none"),
        tombstone_artifact_request(),
        get_artifact_status_request("tombstoned", "redacted"),
        complete_task_request(),
        get_task_status_request("completed"),
        create_escrow_request(),
        release_escrow_request(),
    ]
}

pub(crate) fn create_task_request() -> ExpectedRequest {
    request_with_sender(
        expected_request(
            "POST",
            "/v1/tasks/create",
            r#"{"creator":"kamn:did:agent:creator-live","task_type":"triage","description":"triage contract"}"#,
        ),
        "kamn:did:agent:creator-live",
        "tasks:write",
        201,
        r#"{"task_id":"task-local-abc","state":"submitted"}"#,
    )
}

pub(crate) fn accept_task_request() -> ExpectedRequest {
    request_with_sender(
        expected_request("POST", "/v1/tasks/task-local-abc/accept", "{}"),
        "kamn:did:agent:assignee-live",
        "tasks:write",
        200,
        r#"{"task_id":"task-local-abc","state":"accepted"}"#,
    )
}

pub(crate) fn complete_task_request() -> ExpectedRequest {
    request_with_sender(
        expected_request("POST", "/v1/tasks/task-local-abc/complete", "{}"),
        "kamn:did:agent:assignee-live",
        "tasks:write",
        200,
        r#"{"task_id":"task-local-abc","state":"completed"}"#,
    )
}

pub(crate) fn get_task_status_request(state: &str) -> ExpectedRequest {
    ExpectedRequest {
        method: "GET",
        path: "/v1/tasks/task-local-abc".to_owned(),
        body: String::new(),
        sender_did: "kamn:did:agent:live-requester".to_owned(),
        scope: "tasks:read",
        response_status: 200,
        response_body: format!(r#"{{"task_id":"task-local-abc","state":"{state}"}}"#),
    }
}

pub(crate) fn submit_artifact_request() -> ExpectedRequest {
    request_with_sender(
        expected_request(
            "POST",
            "/v1/content/register",
            r#"{"task_id":"task-local-abc","artifact_name":"artifact.bin","artifact_bytes_hex":"61727469666163742d6279746573"}"#,
        ),
        "kamn:did:agent:assignee-live",
        "content:write",
        201,
        r#"{"content_id":"content-local-artifact-abc","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}"#,
    )
}

pub(crate) fn create_escrow_request() -> ExpectedRequest {
    request_with_sender(
        expected_request(
            "POST",
            "/v1/escrow/fund",
            r#"{"payer":"kamn:did:agent:payer-live","payee":"kamn:did:agent:payee-live","amount":7}"#,
        ),
        "kamn:did:agent:payer-live",
        "escrow:write",
        200,
        r#"{"escrow_id":"escrow-local-xyz","state":"funded"}"#,
    )
}

pub(crate) fn get_artifact_status_request(
    lifecycle_state: &str,
    redaction_status: &str,
) -> ExpectedRequest {
    ExpectedRequest {
        method: "GET",
        path: "/v1/content/content-local-artifact-abc".to_owned(),
        body: String::new(),
        sender_did: "kamn:did:agent:live-requester".to_owned(),
        scope: "content:read",
        response_status: 200,
        response_body: format!(
            r#"{{"content_id":"content-local-artifact-abc","lifecycle_state":"{lifecycle_state}","redaction_status":"{redaction_status}"}}"#
        ),
    }
}

pub(crate) fn release_escrow_request() -> ExpectedRequest {
    request_with_sender(
        expected_request("POST", "/v1/escrow/escrow-local-xyz/release", "{}"),
        "kamn:did:agent:payer-live",
        "escrow:write",
        200,
        r#"{"escrow_id":"escrow-local-xyz","state":"released"}"#,
    )
}

pub(crate) fn expire_artifact_request() -> ExpectedRequest {
    lifecycle_request(
        "/v1/content/content-local-artifact-abc/expire",
        r#"{"content_id":"content-local-artifact-abc","lifecycle_state":"expired","redaction_status":"none"}"#,
    )
}

pub(crate) fn tombstone_artifact_request() -> ExpectedRequest {
    lifecycle_request(
        "/v1/content/content-local-artifact-abc/tombstone",
        r#"{"content_id":"content-local-artifact-abc","lifecycle_state":"tombstoned","redaction_status":"redacted"}"#,
    )
}

fn lifecycle_request(path: &str, response_body: &str) -> ExpectedRequest {
    ExpectedRequest {
        method: "POST",
        path: path.to_owned(),
        body: "{}".to_owned(),
        sender_did: "kamn:did:agent:live-requester".to_owned(),
        scope: "content:write",
        response_status: 200,
        response_body: response_body.to_owned(),
    }
}

fn request_with_sender(
    request: ExpectedRequest,
    sender_did: &str,
    scope: &'static str,
    response_status: u16,
    response_body: &str,
) -> ExpectedRequest {
    ExpectedRequest {
        sender_did: sender_did.to_owned(),
        scope,
        response_status,
        response_body: response_body.to_owned(),
        ..request
    }
}
