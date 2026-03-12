mod lookup;
mod mutations;
mod queries;
mod registration;

use super::{
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest,
    DataLayerM8LegalHoldRequest, DataLayerM8MessageRecord, DataLayerM8MessageRecordInput,
    DataLayerM8OwnerScopeQuery, DataLayerM8RetentionDueCandidate,
};

impl DataLayerM8ComplianceRegistry {
    /// Creates an empty M8 compliance registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one owner-scoped message lifecycle record.
    pub fn register_message(
        &mut self,
        input: DataLayerM8MessageRecordInput,
    ) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        registration::register_message(self, input)
    }

    /// Returns retention-due candidates for an owner at `now_epoch_seconds`.
    pub fn retention_due_for_owner(
        &self,
        query: DataLayerM8OwnerScopeQuery,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM8RetentionDueCandidate>, DataLayerM8ComplianceError> {
        queries::retention_due_for_owner(self, query, now_epoch_seconds)
    }

    /// Applies or releases legal-hold status for one message.
    pub fn set_legal_hold(
        &mut self,
        request: DataLayerM8LegalHoldRequest,
    ) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        mutations::set_legal_hold(self, request)
    }

    /// Executes crypto-shredding for one message.
    pub fn crypto_shred(
        &mut self,
        request: DataLayerM8CryptoShredRequest,
    ) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        mutations::crypto_shred(self, request)
    }

    /// Returns one message record by owner + message id.
    pub fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<&DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        lookup::message_for_owner(self, owner_did, message_id)
    }

    pub(crate) fn owner_records_or_error(
        &self,
        owner_did: &str,
    ) -> Result<&[DataLayerM8MessageRecord], DataLayerM8ComplianceError> {
        lookup::owner_records_or_error(self, owner_did)
    }

    pub(crate) fn owner_message_mut(
        &mut self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<&mut DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        lookup::owner_message_mut(self, owner_did, message_id)
    }
}
