use super::*;

#[path = "contract_server_support/agent_content_route_support.rs"]
mod agent_content_route_support;
#[path = "contract_server_support/bridge_route_support.rs"]
mod bridge_route_support;
#[path = "contract_server_support/message_task_route_support.rs"]
mod message_task_route_support;
#[path = "contract_server_support/public_route_support.rs"]
mod public_route_support;
#[path = "contract_server_support/route_id_support.rs"]
mod route_id_support;
#[path = "contract_server_support/server_runtime_support.rs"]
mod server_runtime_support;

pub(crate) use route_id_support::strip_suffix_id;
pub(crate) use server_runtime_support::{
    run_bound_service_contract_server, run_service_contract_server,
    run_service_contract_server_with_websocket_payload, wait_for_server_ready,
};
