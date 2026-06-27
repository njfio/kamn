pub(crate) use kamn_sdk::{
    service_signature_for_fields, AgentDid, SdkError, ServiceApiClient, ServiceRequestAuth,
};
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
pub(crate) use std::net::{TcpListener, TcpStream};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::{Child, Command, Stdio};
pub(crate) use std::sync::{Mutex, OnceLock};
pub(crate) use std::thread;
pub(crate) use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
pub(crate) use std::{env, fs};

pub(crate) const CHAIN_ID: &str = "kolme-localnet";
pub(crate) const CHAIN_VERSION: &str = "v0";
pub(crate) const REQUEST_AUTH_SCOPE_HEADER: &str = "x-kamn-authz-scope";
pub(crate) const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
pub(crate) const SERVICE_TLS_CA_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_CA_FILE";
pub(crate) const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

#[path = "support/auth_support.rs"]
mod auth_support;
#[path = "support/contract_server_support.rs"]
mod contract_server_support;
#[path = "support/env_support.rs"]
mod env_support;
#[path = "support/request_parse_support.rs"]
mod request_parse_support;
#[path = "support/tls_server_support.rs"]
mod tls_server_support;
#[path = "support/websocket_support.rs"]
mod websocket_support;

pub(crate) use auth_support::{auth_with_scope, validate_auth};
pub(crate) use contract_server_support::{
    run_bound_service_contract_server, run_service_contract_server,
    run_service_contract_server_with_websocket_payload, wait_for_server_ready,
};
pub(crate) use env_support::{
    bind_loopback_listener, ensure_test_service_auth_private_key, reserve_loopback_addr,
    tls_env_lock, unique_temp_dir, EnvVarGuard,
};
pub(crate) use request_parse_support::{parse_http_request, write_http_response};
pub(crate) use tls_server_support::spawn_https_single_request_server;
pub(crate) use websocket_support::{
    deterministic_tag, write_websocket_upgrade_response, DEFAULT_WEBSOCKET_EVENT_PAYLOAD,
};
