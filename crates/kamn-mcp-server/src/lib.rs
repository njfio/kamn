#![warn(missing_docs)]
//! MCP tool-server scaffold for KAMN.

mod authority;
/// MCP configuration scaffold.
pub mod config;
/// MCP tool dispatch and backend adapters.
pub mod dispatch;
mod json_helpers;
/// MCP stdio framing + JSON-RPC protocol handling.
pub mod protocol;
mod registration;
/// MCP tool registry scaffold.
pub mod tools;

pub use dispatch::{dispatch_tool_request_json, invalid_request_response_json, McpToolBackend};
pub use json_helpers::json_optional_bool_field;
pub use protocol::process_stdio_input;
