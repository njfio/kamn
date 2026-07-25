use serde_json::Value;

/// Driver surface participating in one settlement-authority parity proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettlementAuthorityDriver {
    /// Direct Rust SDK entrypoint.
    Sdk,
    /// Scriptable CLI entrypoint.
    Cli,
    /// MCP tool entrypoint.
    Mcp,
}

/// One driver's claim about a shared authoritative settlement.
#[derive(Debug, Clone, PartialEq)]
pub struct SettlementAuthorityAttempt {
    /// Driver that produced the response.
    pub driver: SettlementAuthorityDriver,
    /// Escrow identity claimed by the driver invocation.
    pub escrow_id: String,
    /// Idempotency identity claimed by the driver invocation.
    pub idempotency_key: String,
    /// Raw service or entrypoint response containing authoritative settlement.
    pub response: Value,
}

/// Verified three-driver settlement-authority parity evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementAuthorityParityReport {
    /// Shared escrow identity.
    pub escrow_id: String,
    /// Shared idempotency identity.
    pub idempotency_key: String,
    /// Canonical serialized authoritative settlement.
    pub canonical_authority: String,
    /// Number of economic settlement submissions.
    pub settlement_submissions: u64,
}

/// Structured failure from settlement-authority parity verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementAuthorityParityError {
    /// Stable machine-readable error code.
    pub code: &'static str,
    /// Driver that produced the rejected evidence, when applicable.
    pub driver: Option<SettlementAuthorityDriver>,
    /// Field or validation surface that failed.
    pub field: &'static str,
    /// Human-readable failure description.
    pub message: String,
}

impl SettlementAuthorityParityError {
    pub(crate) fn new(
        code: &'static str,
        driver: Option<SettlementAuthorityDriver>,
        field: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            driver,
            field,
            message: message.into(),
        }
    }
}
