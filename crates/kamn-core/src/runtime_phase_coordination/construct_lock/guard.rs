use super::{ConstructLockError, ConstructLockLease};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Construct lock guard.
pub struct ConstructLockGuard {
    lease_ttl_ticks: u64,
    current_lease: Option<ConstructLockLease>,
}

impl ConstructLockGuard {
    /// Handles new.
    pub fn new(lease_ttl_ticks: u64) -> Result<Self, ConstructLockError> {
        if lease_ttl_ticks == 0 {
            return Err(ConstructLockError::InvalidLeaseTtl);
        }
        Ok(Self { lease_ttl_ticks, current_lease: None })
    }

    /// Handles lease ttl ticks.
    pub fn lease_ttl_ticks(&self) -> u64 { self.lease_ttl_ticks }

    /// Handles acquire for.
    pub fn acquire_for(&mut self, owner_id: &str) -> Result<ConstructLockLease, ConstructLockError> {
        validate_owner_id(owner_id)?;
        if let Some(lease) = &self.current_lease {
            return reuse_or_reject_lease(lease, owner_id);
        }
        let lease = ConstructLockLease::new(owner_id.to_owned(), 1);
        self.current_lease = Some(lease.clone());
        Ok(lease)
    }

    /// Handles renew.
    pub fn renew(&mut self, owner_id: &str, fencing_token: u64) -> Result<ConstructLockLease, ConstructLockError> {
        validate_owner_id(owner_id)?;
        let current_lease = self.current_lease.as_ref().ok_or(ConstructLockError::NoActiveLease)?;
        validate_current_lease(current_lease, owner_id, fencing_token)?;
        let renewed = ConstructLockLease::new(current_lease.owner_id().to_owned(), current_lease.fencing_token() + 1);
        self.current_lease = Some(renewed.clone());
        Ok(renewed)
    }

    /// Handles release.
    pub fn release(&mut self, owner_id: &str, fencing_token: u64) -> Result<(), ConstructLockError> {
        validate_owner_id(owner_id)?;
        let current_lease = self.current_lease.as_ref().ok_or(ConstructLockError::NoActiveLease)?;
        validate_current_lease(current_lease, owner_id, fencing_token)?;
        self.current_lease = None;
        Ok(())
    }

    /// Handles transfer.
    pub fn transfer(&mut self, owner_id: &str, next_owner_id: &str, fencing_token: u64) -> Result<ConstructLockLease, ConstructLockError> {
        validate_owner_id(owner_id)?;
        validate_owner_id(next_owner_id)?;
        let current_lease = self.current_lease.as_ref().ok_or(ConstructLockError::NoActiveLease)?;
        validate_current_lease(current_lease, owner_id, fencing_token)?;
        if current_lease.owner_id() == next_owner_id {
            return Err(ConstructLockError::LeaseAlreadyHeld { owner: current_lease.owner_id().to_owned() });
        }
        let transferred = ConstructLockLease::new(next_owner_id.to_owned(), current_lease.fencing_token() + 1);
        self.current_lease = Some(transferred.clone());
        Ok(transferred)
    }

    /// Handles validate execution lease.
    pub fn validate_execution_lease(&self, owner_id: &str, fencing_token: u64) -> Result<(), ConstructLockError> {
        validate_owner_id(owner_id)?;
        let current_lease = self.current_lease.as_ref().ok_or(ConstructLockError::NoLeaseForExecution)?;
        validate_current_lease(current_lease, owner_id, fencing_token)
    }
}

/// Handles execute processor daemon tick.
pub fn execute_processor_daemon_tick(
    lock_guard: &ConstructLockGuard,
    owner_id: &str,
    fencing_token: u64,
    executed_ticks: u64,
) -> Result<u64, ConstructLockError> {
    lock_guard.validate_execution_lease(owner_id, fencing_token)?;
    Ok(executed_ticks + 1)
}

fn validate_owner_id(owner_id: &str) -> Result<(), ConstructLockError> {
    if owner_id.trim().is_empty() { return Err(ConstructLockError::InvalidOwnerId); }
    Ok(())
}

fn reuse_or_reject_lease(lease: &ConstructLockLease, owner_id: &str) -> Result<ConstructLockLease, ConstructLockError> {
    if lease.owner_id() != owner_id {
        return Err(ConstructLockError::LeaseAlreadyHeld { owner: lease.owner_id().to_owned() });
    }
    Ok(lease.clone())
}

fn validate_current_lease(
    current_lease: &ConstructLockLease,
    owner_id: &str,
    fencing_token: u64,
) -> Result<(), ConstructLockError> {
    if current_lease.owner_id() != owner_id {
        return Err(ConstructLockError::LeaseOwnerMismatch {
            expected: current_lease.owner_id().to_owned(),
            found: owner_id.to_owned(),
        });
    }
    if current_lease.fencing_token() != fencing_token {
        return Err(ConstructLockError::StaleFencingToken {
            expected: current_lease.fencing_token(),
            found: fencing_token,
        });
    }
    Ok(())
}
