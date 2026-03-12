use crate::EscrowStatus;
use std::fmt;

/// Escrow state projection aligned to PRD M4 lifecycle markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM4EscrowState {
    /// Escrow draft created but not funded.
    Created,
    /// Escrow funding confirmed.
    Funded,
    /// Escrow active for normal operations.
    Active,
    /// Escrow in dispute state.
    Disputed,
    /// Escrow settled by release.
    Released,
    /// Escrow settled by refund.
    Refunded,
    /// Escrow expired without settlement.
    Expired,
}

impl DataLayerM4EscrowState {
    pub(crate) fn as_marker(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Funded => "funded",
            Self::Active => "active",
            Self::Disputed => "disputed",
            Self::Released => "released",
            Self::Refunded => "refunded",
            Self::Expired => "expired",
        }
    }
}

/// Interoperability errors for bridging M4 escrow contracts with legacy `escrow.rs` types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4EscrowInteropError {
    /// Legacy status cannot be represented without semantic loss in M4 state model.
    UnsupportedLegacyStatus(EscrowStatus),
}

impl TryFrom<EscrowStatus> for DataLayerM4EscrowState {
    type Error = DataLayerM4EscrowInteropError;

    fn try_from(value: EscrowStatus) -> Result<Self, Self::Error> {
        if let Some(state) = resolved_terminal_state(&value) {
            return Ok(state);
        }
        match value {
            EscrowStatus::Funded => Ok(Self::Funded),
            EscrowStatus::PartiallyReleased { .. } => Ok(Self::Active),
            EscrowStatus::Released => Ok(Self::Released),
            EscrowStatus::Refunded => Ok(Self::Refunded),
            EscrowStatus::Disputed => Ok(Self::Disputed),
            EscrowStatus::Resolved { .. } => {
                Err(DataLayerM4EscrowInteropError::UnsupportedLegacyStatus(value))
            }
        }
    }
}

fn resolved_terminal_state(status: &EscrowStatus) -> Option<DataLayerM4EscrowState> {
    match status {
        EscrowStatus::Resolved {
            released_total,
            refunded_total,
        } if *released_total > 0 && *refunded_total == 0 => Some(DataLayerM4EscrowState::Released),
        EscrowStatus::Resolved {
            released_total,
            refunded_total,
        } if *refunded_total > 0 && *released_total == 0 => Some(DataLayerM4EscrowState::Refunded),
        _ => None,
    }
}

impl fmt::Display for DataLayerM4EscrowInteropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedLegacyStatus(status) => {
                write!(f, "legacy escrow status cannot be represented in M4: {status:?}")
            }
        }
    }
}

impl std::error::Error for DataLayerM4EscrowInteropError {}
