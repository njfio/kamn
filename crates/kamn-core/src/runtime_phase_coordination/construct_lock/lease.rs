#[derive(Debug, Clone, PartialEq, Eq)]
/// Construct lock lease.
pub struct ConstructLockLease {
    owner_id: String,
    fencing_token: u64,
}

impl ConstructLockLease {
    pub(crate) fn new(owner_id: String, fencing_token: u64) -> Self {
        Self {
            owner_id,
            fencing_token,
        }
    }

    /// Handles owner id.
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    /// Handles fencing token.
    pub fn fencing_token(&self) -> u64 {
        self.fencing_token
    }
}
