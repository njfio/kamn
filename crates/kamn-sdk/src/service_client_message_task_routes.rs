use super::super::{
    expect_status, json_string_array_field, json_string_field, normalize_route_segment, SdkError,
    ServiceChannelMessages, ServiceChannelReceipt, ServiceMessageDelivery, ServiceMessageReceipt,
    ServiceMessageStatus, ServiceRequestAuth, ServiceTaskReceipt, ServiceTaskStatus,
    ServiceTaskTransitionReceipt,
};
use super::receipt_fields;
use super::ServiceApiClient;
use serde_json::Value;

impl ServiceApiClient {
    /// Sends a signed message payload through the service API.
    pub fn send_message(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageReceipt, SdkError> {
        let response = self.request("POST", "/v1/messages/send", payload, Some(auth))?;
        expect_status(response.status, 202)?;
        Ok(ServiceMessageReceipt {
            message_id: json_string_field(response.body.as_str(), "message_id")?,
            status: json_string_field(response.body.as_str(), "status")?,
            runtime_mode: json_string_field(response.body.as_str(), "runtime_mode")?,
        })
    }

    /// Queries a message status by identifier.
    pub fn get_message(
        &self,
        message_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageStatus, SdkError> {
        let message_id = normalize_route_segment("message_id", message_id)?;
        let route = format!("/v1/messages/{message_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceMessageStatus {
            message_id: json_string_field(response.body.as_str(), "message_id")?,
            status: json_string_field(response.body.as_str(), "status")?,
        })
    }

    pub(crate) fn get_message_delivery(
        &self,
        message_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageDelivery, SdkError> {
        let message_id = normalize_route_segment("message_id", message_id)?;
        let route = format!("/v1/messages/{message_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        parse_message_delivery(response.body.as_str())
    }

    /// Creates a channel payload through the service API.
    pub fn create_channel(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceChannelReceipt, SdkError> {
        let response = self.request("POST", "/v1/channels/create", payload, Some(auth))?;
        expect_status(response.status, 201)?;
        Ok(ServiceChannelReceipt {
            channel_id: json_string_field(response.body.as_str(), "channel_id")?,
            status: json_string_field(response.body.as_str(), "status")?,
        })
    }

    /// Lists channel messages through `GET /v1/channels/{id}/messages`.
    pub fn list_channel_messages(
        &self,
        channel_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceChannelMessages, SdkError> {
        let channel_id = normalize_route_segment("channel_id", channel_id)?;
        let route = format!("/v1/channels/{channel_id}/messages");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceChannelMessages {
            channel_id: json_string_field(response.body.as_str(), "channel_id")?,
            messages: json_string_array_field(response.body.as_str(), "messages")?,
        })
    }

    /// Creates a task payload through the service API.
    pub fn create_task(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskReceipt, SdkError> {
        let response = self.request("POST", "/v1/tasks/create", payload, Some(auth))?;
        expect_status(response.status, 201)?;
        let (receipt_id, receipt_digest) = receipt_fields(response.body.as_str())?;
        Ok(ServiceTaskReceipt {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
            receipt_id,
            receipt_digest,
            action: json_string_field(response.body.as_str(), "action")?,
        })
    }

    /// Queries task lifecycle state by identifier.
    pub fn get_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceTaskStatus {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Accepts one task through `POST /v1/tasks/{id}/accept`.
    pub fn accept_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskTransitionReceipt, SdkError> {
        self.accept_task_with_payload(task_id, "{}", auth)
    }

    /// Accepts one task with a canonical transition payload.
    pub fn accept_task_with_payload(
        &self,
        task_id: &str,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskTransitionReceipt, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}/accept");
        let response = self.request("POST", route.as_str(), payload, Some(auth))?;
        expect_status(response.status, 200)?;
        let (receipt_id, receipt_digest) = receipt_fields(response.body.as_str())?;
        Ok(ServiceTaskTransitionReceipt {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
            receipt_id,
            receipt_digest,
            action: json_string_field(response.body.as_str(), "action")?,
        })
    }

    /// Completes one task through `POST /v1/tasks/{id}/complete`.
    pub fn complete_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskTransitionReceipt, SdkError> {
        self.complete_task_with_payload(task_id, "{}", auth)
    }

    /// Completes one task with a canonical evidence payload.
    pub fn complete_task_with_payload(
        &self,
        task_id: &str,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskTransitionReceipt, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}/complete");
        let response = self.request("POST", route.as_str(), payload, Some(auth))?;
        expect_status(response.status, 200)?;
        let (receipt_id, receipt_digest) = receipt_fields(response.body.as_str())?;
        Ok(ServiceTaskTransitionReceipt {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
            receipt_id,
            receipt_digest,
            action: json_string_field(response.body.as_str(), "action")?,
        })
    }
}

fn parse_message_delivery(payload: &str) -> Result<ServiceMessageDelivery, SdkError> {
    let root = serde_json::from_str::<Value>(payload)
        .map_err(|_| SdkError::TransportFailure("service response payload was not valid json"))?;
    Ok(ServiceMessageDelivery {
        message_id: required_message_field(
            &root,
            "message_id",
            "service message response missing required message_id",
        )?,
        sender_did: required_message_field(
            &root,
            "sender_did",
            "service message response missing required sender_did",
        )?,
        recipient_did: required_message_field(
            &root,
            "recipient_did",
            "service message response missing required recipient_did",
        )?,
        body: required_message_field(
            &root,
            "body",
            "service message response missing required body",
        )?,
    })
}

fn required_message_field(
    root: &Value,
    key: &str,
    missing_message: &'static str,
) -> Result<String, SdkError> {
    let Some(value) = root.get(key) else {
        return Err(SdkError::TransportFailure(missing_message));
    };
    value
        .as_str()
        .map(str::to_owned)
        .ok_or(SdkError::TransportFailure(
            "service message response field was malformed",
        ))
}
