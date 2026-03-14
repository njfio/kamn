use super::*;
use crate::service_api_endpoint::{
    parse_service_api_payload, upsert_service_api_relayed_message_from_daemon,
    ServiceApiAgentGetBody, ServiceApiChannelCreateBody, ServiceApiErrorBody, ServiceApiHealthBody,
    ServiceApiMessageCreateBody, ServiceApiRelaySpoolEntry, ServiceApiTaskCreateBody,
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND, SERVICE_API_AUTH_REASON_CODES_CSV,
    SERVICE_API_AUTH_REASON_TAXONOMY_VERSION, SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT,
    SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION, SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV,
    SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION, SERVICE_API_WEBSOCKET_REASON_CODES_CSV,
};
use kamn_core::AgentDid;
use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[path = "service_api_endpoint_tests/auth_scope_contract_tests.rs"]
mod auth_scope_contract_tests;
#[path = "service_api_endpoint_tests/balance_contract_tests.rs"]
mod balance_contract_tests;
#[path = "service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs"]
mod bridge_persistence_restart_contract_tests;
#[path = "service_api_endpoint_tests/channel_agent_directory_contract_tests.rs"]
mod channel_agent_directory_contract_tests;
#[path = "service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs"]
mod content_lifecycle_restart_contract_tests;
#[path = "service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs"]
mod ingress_guard_lifecycle_contract_tests;
#[path = "service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs"]
mod mailbox_relay_delivery_contract_tests;
#[path = "service_api_endpoint_tests/message_persistence_contract_tests.rs"]
mod message_persistence_contract_tests;
#[path = "service_api_endpoint_tests/residual_root_contract_tests.rs"]
mod residual_root_contract_tests;
#[path = "service_api_endpoint_tests/route_render_contract_tests.rs"]
mod route_render_contract_tests;
#[path = "service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs"]
mod task_escrow_persistence_contract_tests;
#[path = "service_api_endpoint_tests/transport_surface_observability_contract_tests.rs"]
mod transport_surface_observability_contract_tests;
#[path = "service_api_endpoint_tests/vertical_slice_contract_tests.rs"]
mod vertical_slice_contract_tests;
#[path = "service_api_endpoint_tests/websocket_contract_tests.rs"]
mod websocket_contract_tests;

#[path = "service_api_endpoint_tests/shared_support.rs"]
mod shared_support;

pub(crate) use shared_support::*;
