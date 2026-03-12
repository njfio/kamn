use kamn_data_layer::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10Phase6CompliancePort,
    DataLayerM10Phase6CompliancePortError, DataLayerM10Phase6CryptoShredInput,
    DataLayerM10Phase6RetentionDueCandidate,
};

use crate::{
    DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest, DataLayerM8OwnerScopeQuery,
};

use super::error_mapping::{
    map_m8_execution_error_to_phase6_port, map_phase6_owner_scope_error_to_phase6_port,
    map_phase6_port_error_to_projection_port_error,
};
use crate::data_layer_m10_partition_archival::shared::authorize_owner_scope;

pub(crate) struct M8Phase6CompliancePortAdapter<'a> {
    compliance_registry: &'a mut DataLayerM8ComplianceRegistry,
}

impl<'a> M8Phase6CompliancePortAdapter<'a> {
    pub(crate) fn new(compliance_registry: &'a mut DataLayerM8ComplianceRegistry) -> Self {
        Self {
            compliance_registry,
        }
    }
}

impl DataLayerM10Phase6CompliancePort for M8Phase6CompliancePortAdapter<'_> {
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10Phase6CompliancePortError> {
        let owner_did = authorize_owner_scope(requester_owner_did, owner_did)
            .map_err(map_phase6_owner_scope_error_to_phase6_port)?;
        Ok(owner_did.as_str().to_owned())
    }

    fn retention_due_for_owner(
        &self,
        owner_did: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM10Phase6RetentionDueCandidate>, DataLayerM10Phase6CompliancePortError>
    {
        self.compliance_registry
            .retention_due_for_owner(
                DataLayerM8OwnerScopeQuery {
                    requester_owner_did: owner_did.to_owned(),
                    owner_did: owner_did.to_owned(),
                },
                now_epoch_seconds,
            )
            .map_err(map_m8_execution_error_to_phase6_port)
            .map(|candidates| {
                candidates
                    .into_iter()
                    .map(|candidate| DataLayerM10Phase6RetentionDueCandidate {
                        message_id: candidate.message_id,
                    })
                    .collect()
            })
    }

    fn crypto_shred(
        &mut self,
        input: DataLayerM10Phase6CryptoShredInput,
    ) -> Result<(), DataLayerM10Phase6CompliancePortError> {
        self.compliance_registry
            .crypto_shred(DataLayerM8CryptoShredRequest {
                requester_owner_did: input.requester_owner_did,
                owner_did: input.owner_did,
                message_id: input.message_id,
                shredded_at_epoch_seconds: input.shredded_at_epoch_seconds,
            })
            .map(|_| ())
            .map_err(map_m8_execution_error_to_phase6_port)
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10Phase6CompliancePortError>
    {
        self.compliance_registry
            .message_for_owner(owner_did, message_id)
            .map_err(map_m8_execution_error_to_phase6_port)
            .map(|message| DataLayerM10ComplianceProjectionMessageState {
                message_id: message.message_id.clone(),
                legal_hold_active: message.legal_hold_active,
                shredded_at_epoch_seconds: message.shredded_at_epoch_seconds,
            })
    }
}

pub(crate) struct Phase6ProjectionPortBridge<'a, T: DataLayerM10Phase6CompliancePort> {
    phase6_port: &'a T,
}

impl<'a, T: DataLayerM10Phase6CompliancePort> Phase6ProjectionPortBridge<'a, T> {
    pub(crate) fn new(phase6_port: &'a T) -> Self {
        Self { phase6_port }
    }
}

impl<T: DataLayerM10Phase6CompliancePort> DataLayerM10ComplianceProjectionPort
    for Phase6ProjectionPortBridge<'_, T>
{
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10ComplianceProjectionPortError> {
        self.phase6_port
            .authorize_owner_scope(requester_owner_did, owner_did)
            .map_err(map_phase6_port_error_to_projection_port_error)
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<
        DataLayerM10ComplianceProjectionMessageState,
        DataLayerM10ComplianceProjectionPortError,
    > {
        self.phase6_port
            .message_for_owner(owner_did, message_id)
            .map_err(map_phase6_port_error_to_projection_port_error)
    }
}
