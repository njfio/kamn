//! Canonical service API authorization scope taxonomy.

use std::fmt::{Display, Formatter};

/// Canonical service API authorization scope values used across runtime surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ServiceApiScope {
    /// Write scope for message creation/send operations.
    MessagesWrite,
    /// Read scope for message lookup operations.
    MessagesRead,
    /// Write scope for channel creation operations.
    ChannelsWrite,
    /// Read scope for channel message listing operations.
    ChannelsRead,
    /// Write scope for task lifecycle mutation operations.
    TasksWrite,
    /// Read scope for task query operations.
    TasksRead,
    /// Write scope for escrow lifecycle mutation operations.
    EscrowWrite,
    /// Write scope for content lifecycle mutation operations.
    ContentWrite,
    /// Read scope for content lifecycle query operations.
    ContentRead,
    /// Read scope for agent profile/query operations.
    AgentsRead,
    /// Read scope for event stream/websocket operations.
    EventsRead,
    /// Fallback scope for protected but unmapped routes.
    ProtectedUnknown,
}

impl ServiceApiScope {
    /// Render the canonical string form.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MessagesWrite => "messages:write",
            Self::MessagesRead => "messages:read",
            Self::ChannelsWrite => "channels:write",
            Self::ChannelsRead => "channels:read",
            Self::TasksWrite => "tasks:write",
            Self::TasksRead => "tasks:read",
            Self::EscrowWrite => "escrow:write",
            Self::ContentWrite => "content:write",
            Self::ContentRead => "content:read",
            Self::AgentsRead => "agents:read",
            Self::EventsRead => "events:read",
            Self::ProtectedUnknown => "protected:unknown",
        }
    }

    /// Parse an inbound scope header value using canonical string matching.
    pub fn parse(raw: &str) -> Result<Self, ServiceApiScopeError> {
        let normalized = raw.trim();
        if normalized.is_empty() {
            return Err(ServiceApiScopeError::Empty);
        }
        let scope = match normalized {
            "messages:write" => Self::MessagesWrite,
            "messages:read" => Self::MessagesRead,
            "channels:write" => Self::ChannelsWrite,
            "channels:read" => Self::ChannelsRead,
            "tasks:write" => Self::TasksWrite,
            "tasks:read" => Self::TasksRead,
            "escrow:write" => Self::EscrowWrite,
            "content:write" => Self::ContentWrite,
            "content:read" => Self::ContentRead,
            "agents:read" => Self::AgentsRead,
            "events:read" => Self::EventsRead,
            "protected:unknown" => Self::ProtectedUnknown,
            _ => return Err(ServiceApiScopeError::Unknown),
        };
        Ok(scope)
    }
}

impl Display for ServiceApiScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parse failures for [`ServiceApiScope`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceApiScopeError {
    /// Scope value was empty after trim.
    Empty,
    /// Scope value did not match any canonical scope.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::{ServiceApiScope, ServiceApiScopeError};

    #[test]
    fn unit_service_api_scope_parse_accepts_canonical_values() {
        assert_eq!(
            ServiceApiScope::parse("messages:write").expect("scope"),
            ServiceApiScope::MessagesWrite
        );
        assert_eq!(
            ServiceApiScope::parse("messages:read").expect("scope"),
            ServiceApiScope::MessagesRead
        );
        assert_eq!(
            ServiceApiScope::parse("channels:write").expect("scope"),
            ServiceApiScope::ChannelsWrite
        );
        assert_eq!(
            ServiceApiScope::parse("channels:read").expect("scope"),
            ServiceApiScope::ChannelsRead
        );
        assert_eq!(
            ServiceApiScope::parse("tasks:write").expect("scope"),
            ServiceApiScope::TasksWrite
        );
        assert_eq!(
            ServiceApiScope::parse("tasks:read").expect("scope"),
            ServiceApiScope::TasksRead
        );
        assert_eq!(
            ServiceApiScope::parse("escrow:write").expect("scope"),
            ServiceApiScope::EscrowWrite
        );
        assert_eq!(
            ServiceApiScope::parse("content:write").expect("scope"),
            ServiceApiScope::ContentWrite
        );
        assert_eq!(
            ServiceApiScope::parse("content:read").expect("scope"),
            ServiceApiScope::ContentRead
        );
        assert_eq!(
            ServiceApiScope::parse("agents:read").expect("scope"),
            ServiceApiScope::AgentsRead
        );
        assert_eq!(
            ServiceApiScope::parse("events:read").expect("scope"),
            ServiceApiScope::EventsRead
        );
        assert_eq!(
            ServiceApiScope::parse("protected:unknown").expect("scope"),
            ServiceApiScope::ProtectedUnknown
        );
    }

    #[test]
    fn regression_service_api_scope_parse_rejects_empty_or_unknown_values() {
        // Regression: #5831
        assert_eq!(
            ServiceApiScope::parse("   "),
            Err(ServiceApiScopeError::Empty)
        );
        assert_eq!(
            ServiceApiScope::parse("content:admin"),
            Err(ServiceApiScopeError::Unknown)
        );
    }

    #[test]
    fn unit_service_api_scope_parse_trims_whitespace_and_renders_canonical_value() {
        let parsed = ServiceApiScope::parse("  tasks:write  ").expect("scope");
        assert_eq!(parsed, ServiceApiScope::TasksWrite);
        assert_eq!(parsed.as_str(), "tasks:write");
        assert_eq!(parsed.to_string(), "tasks:write");
    }
}
