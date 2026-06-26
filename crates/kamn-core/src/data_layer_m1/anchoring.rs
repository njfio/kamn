use super::{
    support::{map_receipt, tagged_digest},
    DataLayerM1AnchorOutcome, DataLayerM1AnchorReceipt, DataLayerM1AnchorResult,
    DataLayerM1AnchorRetryClass, DataLayerM1Error, DataLayerM1MerkleBatch,
};
use crate::kolme_runtime_commit::{
    KolmeCommitReceiptFinality, KolmeRuntimeCommitClient, KolmeRuntimeCommitOutcome,
    KolmeRuntimeCommitRequest,
};
use crate::AgentDid;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct DataLayerM1KolmeAnchoringWorker<C> {
    pub(crate) client: C,
    pub(crate) actor_did: AgentDid,
    pub(crate) state_root_prefix: String,
    pub(crate) next_nonce: u64,
    pub(crate) nonce_by_batch_id: BTreeMap<String, u64>,
    pub(crate) idempotency_by_batch_id: BTreeMap<String, String>,
    pub(crate) receipt_by_batch_id: BTreeMap<String, DataLayerM1AnchorReceipt>,
}

impl<C> DataLayerM1KolmeAnchoringWorker<C> {
    pub fn new(
        client: C,
        actor_did: &str,
        state_root_prefix: &str,
    ) -> Result<Self, DataLayerM1Error> {
        if state_root_prefix.trim().is_empty() {
            return Err(DataLayerM1Error::EmptyField("state_root_prefix"));
        }
        let actor_did = AgentDid::parse(actor_did)
            .map_err(|_| DataLayerM1Error::InvalidActorDid(actor_did.to_owned()))?;
        Ok(Self {
            client,
            actor_did,
            state_root_prefix: state_root_prefix.to_owned(),
            next_nonce: 1,
            nonce_by_batch_id: Default::default(),
            idempotency_by_batch_id: Default::default(),
            receipt_by_batch_id: Default::default(),
        })
    }
}

impl<C> DataLayerM1KolmeAnchoringWorker<C>
where
    C: KolmeRuntimeCommitClient,
{
    pub fn anchor_batch(
        &mut self,
        batch: &DataLayerM1MerkleBatch,
    ) -> Result<DataLayerM1AnchorResult, DataLayerM1Error> {
        if let Some(result) = self.existing_receipt_result(batch)? {
            return Ok(result);
        }
        let nonce = self.assign_or_resolve_nonce(batch.batch_id.as_str());
        let request = self.build_request(batch, nonce)?;
        let idempotency_key = request.idempotency_key().to_owned();
        self.upsert_idempotency(batch.batch_id.as_str(), idempotency_key.as_str())?;
        let outcome = self.client.submit_commit(&request)?;
        self.map_submit_outcome(batch, idempotency_key, outcome)
    }

    fn existing_receipt_result(
        &self,
        batch: &DataLayerM1MerkleBatch,
    ) -> Result<Option<DataLayerM1AnchorResult>, DataLayerM1Error> {
        let Some(receipt) = self.receipt_by_batch_id.get(&batch.batch_id) else {
            return Ok(None);
        };
        let idempotency_key = self
            .idempotency_by_batch_id
            .get(&batch.batch_id)
            .cloned()
            .ok_or(DataLayerM1Error::InvalidAnchoringState(
                "missing idempotency key for accepted receipt",
            ))?;
        Ok(Some(DataLayerM1AnchorResult {
            batch_id: batch.batch_id.clone(),
            idempotency_key,
            retry_class: DataLayerM1AnchorRetryClass::FinalizedNoRetry,
            outcome: DataLayerM1AnchorOutcome::Duplicate(receipt.clone()),
        }))
    }

    fn map_submit_outcome(
        &mut self,
        batch: &DataLayerM1MerkleBatch,
        idempotency_key: String,
        outcome: KolmeRuntimeCommitOutcome,
    ) -> Result<DataLayerM1AnchorResult, DataLayerM1Error> {
        match outcome {
            KolmeRuntimeCommitOutcome::Submitted(receipt) => Ok(self.accepted_result(
                batch,
                idempotency_key,
                receipt,
                DataLayerM1AnchorRetryClass::NewSubmission,
            )),
            KolmeRuntimeCommitOutcome::Duplicate(receipt) => {
                let retry_class = retry_class_from_receipt(&receipt);
                Ok(self.accepted_result(batch, idempotency_key, receipt, retry_class))
            }
            KolmeRuntimeCommitOutcome::Rejected { reason } => Ok(DataLayerM1AnchorResult {
                batch_id: batch.batch_id.clone(),
                idempotency_key,
                retry_class: DataLayerM1AnchorRetryClass::ConflictNoRetry,
                outcome: DataLayerM1AnchorOutcome::Rejected { reason },
            }),
        }
    }

    fn accepted_result(
        &mut self,
        batch: &DataLayerM1MerkleBatch,
        idempotency_key: String,
        receipt: crate::kolme_runtime_commit::KolmeRuntimeCommitReceipt,
        retry_class: DataLayerM1AnchorRetryClass,
    ) -> DataLayerM1AnchorResult {
        let mapped = map_receipt(receipt);
        self.receipt_by_batch_id
            .insert(batch.batch_id.clone(), mapped.clone());
        DataLayerM1AnchorResult {
            batch_id: batch.batch_id.clone(),
            idempotency_key,
            retry_class,
            outcome: build_outcome(mapped, retry_class),
        }
    }

    fn assign_or_resolve_nonce(&mut self, batch_id: &str) -> u64 {
        if let Some(existing) = self.nonce_by_batch_id.get(batch_id) {
            return *existing;
        }
        let nonce = self.next_nonce;
        self.next_nonce += 1;
        self.nonce_by_batch_id.insert(batch_id.to_owned(), nonce);
        nonce
    }

    fn upsert_idempotency(
        &mut self,
        batch_id: &str,
        idempotency_key: &str,
    ) -> Result<(), DataLayerM1Error> {
        if let Some(existing) = self.idempotency_by_batch_id.get(batch_id) {
            if existing != idempotency_key {
                return Err(DataLayerM1Error::ConflictingAnchoringIdempotencyKey {
                    batch_id: batch_id.to_owned(),
                    existing_key: existing.clone(),
                    provided_key: idempotency_key.to_owned(),
                });
            }
            return Ok(());
        }
        self.idempotency_by_batch_id
            .insert(batch_id.to_owned(), idempotency_key.to_owned());
        Ok(())
    }

    fn build_request(
        &self,
        batch: &DataLayerM1MerkleBatch,
        nonce: u64,
    ) -> Result<KolmeRuntimeCommitRequest, DataLayerM1Error> {
        let operation_id = format!("data-layer-m1-anchor-{}", batch.batch_id);
        let state_root = format!("{}:{}", self.state_root_prefix, batch.merkle_root);
        let payload_hash = tagged_digest(anchor_payload(batch).as_str());
        Ok(KolmeRuntimeCommitRequest::deterministic(
            operation_id.as_str(),
            state_root.as_str(),
            self.actor_did.as_str(),
            nonce,
            payload_hash.as_str(),
        )?)
    }
}

fn retry_class_from_receipt(
    receipt: &crate::kolme_runtime_commit::KolmeRuntimeCommitReceipt,
) -> DataLayerM1AnchorRetryClass {
    if receipt.finality == KolmeCommitReceiptFinality::Pending {
        DataLayerM1AnchorRetryClass::RetryableInFlight
    } else {
        DataLayerM1AnchorRetryClass::FinalizedNoRetry
    }
}

fn anchor_payload(batch: &DataLayerM1MerkleBatch) -> String {
    format!(
        "anchor-payload|batch:{}|root:{}|count:{}|first:{}|last:{}",
        batch.batch_id,
        batch.merkle_root,
        batch.message_count,
        batch.first_message_id,
        batch.last_message_id
    )
}

fn build_outcome(
    receipt: DataLayerM1AnchorReceipt,
    retry_class: DataLayerM1AnchorRetryClass,
) -> DataLayerM1AnchorOutcome {
    if retry_class == DataLayerM1AnchorRetryClass::NewSubmission {
        DataLayerM1AnchorOutcome::Submitted(receipt)
    } else {
        DataLayerM1AnchorOutcome::Duplicate(receipt)
    }
}
