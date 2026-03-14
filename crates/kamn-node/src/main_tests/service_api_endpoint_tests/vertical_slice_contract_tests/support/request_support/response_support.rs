use super::super::super::super::*;
use crate::service_api_endpoint::{ServiceApiMessageCreateBody, ServiceApiSnapshot};

pub(crate) fn parse_created_message(response: &str) -> ServiceApiMessageCreateBody {
    assert_status(response, "HTTP/1.1 202 Accepted");
    parse_payload(response, "send payload should deserialize")
}

pub(crate) fn parse_created_payload<T>(response: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    assert_status(response, "HTTP/1.1 201 Created");
    parse_payload(response, "created payload should deserialize")
}

pub(crate) fn parse_ok_payload<T>(response: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    assert_status(response, "HTTP/1.1 200 OK");
    parse_payload(response, "ok payload should deserialize")
}

pub(crate) fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!("service-api:{}:{}", snapshot.chain_id.as_str(), snapshot.chain_version.as_str())
}

fn assert_status(response: &str, expected: &str) {
    assert!(response.contains(expected));
}

fn parse_payload<T>(response: &str, error_message: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    parse_service_api_payload(extract_http_response_body(response)).expect(error_message)
}
