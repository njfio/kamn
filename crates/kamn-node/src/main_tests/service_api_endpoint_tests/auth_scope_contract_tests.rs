use super::*;

use support::{join_service_api_server, service_api_request_state_hash, start_service_api_server};

#[path = "auth_scope_contract_tests/auth_binding_contract_tests.rs"]
mod auth_binding_contract_tests;
#[path = "auth_scope_contract_tests/legacy_signature_contract_tests.rs"]
mod legacy_signature_contract_tests;
#[path = "auth_scope_contract_tests/route_scope_policy_contract_tests.rs"]
mod route_scope_policy_contract_tests;
#[path = "auth_scope_contract_tests/support.rs"]
mod support;
