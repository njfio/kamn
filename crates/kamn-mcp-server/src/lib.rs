#![warn(missing_docs)]
//! MCP tool-server scaffold for KAMN.

/// MCP configuration scaffold.
pub mod config;
/// MCP tool dispatch and backend adapters.
pub mod dispatch;
/// MCP tool registry scaffold.
pub mod tools;

pub use dispatch::{dispatch_tool_request_json, invalid_request_response_json, McpToolBackend};
