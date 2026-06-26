use super::{
    DataLayerM1AnchorOutcome, DataLayerM1AnchorReceipt, DataLayerM1AnchorRetryClass,
    DataLayerM1MerkleBatch,
};
use crate::kolme_runtime_commit::{KolmeCommitReceiptFinality, KolmeRuntimeCommitReceipt};

pub(crate) fn retry_class_from_receipt(
    receipt: &KolmeRuntimeCommitReceipt,
) -> DataLayerM1AnchorRetryClass {
    if receipt.finality == KolmeCommitReceiptFinality::Pending {
        DataLayerM1AnchorRetryClass::RetryableInFlight
    } else {
        DataLayerM1AnchorRetryClass::FinalizedNoRetry
    }
}

pub(crate) fn anchor_payload(batch: &DataLayerM1MerkleBatch) -> String {
    format!(
        "anchor-payload|batch:{}|root:{}|count:{}|first:{}|last:{}",
        batch.batch_id,
        batch.merkle_root,
        batch.message_count,
        batch.first_message_id,
        batch.last_message_id
    )
}

pub(crate) fn build_outcome(
    receipt: DataLayerM1AnchorReceipt,
    retry_class: DataLayerM1AnchorRetryClass,
) -> DataLayerM1AnchorOutcome {
    if retry_class == DataLayerM1AnchorRetryClass::NewSubmission {
        DataLayerM1AnchorOutcome::Submitted(receipt)
    } else {
        DataLayerM1AnchorOutcome::Duplicate(receipt)
    }
}
