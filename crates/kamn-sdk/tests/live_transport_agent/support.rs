pub(crate) use kamn_sdk::{
    service_signature_for_fields, AgentDid, AgentMetadata, AgentQuery, KamnAgent, KamnTransport,
    LiveTransportConfig, LiveTransportKamnClient, Message, MessageId, SdkError, TransportMode,
};
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::io::{ErrorKind, Read, Write};
pub(crate) use std::net::{TcpListener, TcpStream};
pub(crate) use std::sync::{Mutex, OnceLock};
pub(crate) use std::thread;
pub(crate) use std::time::{Duration, Instant};

#[path = "support/auth_support.rs"]
mod auth_support;
#[path = "support/contract_server_support.rs"]
mod contract_server_support;
#[path = "support/env_support.rs"]
mod env_support;
#[path = "support/request_parse_support.rs"]
mod request_parse_support;

pub(crate) use auth_support::validate_auth;
pub(crate) use contract_server_support::run_live_transport_contract_server;
pub(crate) use env_support::{
    deterministic_message_id, did, ensure_live_test_env, metadata, reserve_loopback_addr,
    wait_for_server_ready, with_env_lock, CHAIN_ID, CHAIN_VERSION, DEFAULT_LIVE_REQUESTER_DID,
    LIVE_REQUESTER_DID_ENV, REQUEST_AUTH_SCOPE_HEADER,
};
pub(crate) use request_parse_support::{parse_http_request, write_http_response};
