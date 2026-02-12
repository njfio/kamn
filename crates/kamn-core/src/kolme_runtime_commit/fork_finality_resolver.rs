//! Fork-profile finality resolver orchestration for notifications and block fallback.

use super::{
    require_kolme_commit_id_matches_expected_txhash_contract, resolve_kolme_lookup_upper_bound,
    txhash_from_kolme_commit_id, KolmeRuntimeCommitBlockFallbackReconciler,
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitNotificationEvent,
    KolmeRuntimeCommitNotificationsConnector, KolmeRuntimeCommitNotificationsConsumer,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderReceipt,
};

/// Fork-profile finality resolver that composes notifications and block fallback lookups.
pub struct KolmeRuntimeCommitForkFinalityResolver<C, T>
where
    C: KolmeRuntimeCommitNotificationsConnector,
    T: KolmeRuntimeCommitBlockFallbackTransport,
{
    notifications_consumer: KolmeRuntimeCommitNotificationsConsumer<C>,
    block_fallback_reconciler: KolmeRuntimeCommitBlockFallbackReconciler<T>,
}

impl<C, T> KolmeRuntimeCommitForkFinalityResolver<C, T>
where
    C: KolmeRuntimeCommitNotificationsConnector,
    T: KolmeRuntimeCommitBlockFallbackTransport,
{
    /// Builds a fork finality resolver from notifications and block fallback components.
    pub fn new(
        notifications_consumer: KolmeRuntimeCommitNotificationsConsumer<C>,
        block_fallback_reconciler: KolmeRuntimeCommitBlockFallbackReconciler<T>,
    ) -> Self {
        Self {
            notifications_consumer,
            block_fallback_reconciler,
        }
    }

    /// Resolves finality for one commit id using notifications first, then bounded block fallback.
    pub fn resolve_commit_finality(
        &mut self,
        commit_id: &str,
        from_height: u64,
        latest_height: u64,
    ) -> Result<KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderError> {
        let expected_txhash = txhash_from_kolme_commit_id(commit_id).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            }
        })?;

        match self.notifications_consumer.next_notification_event() {
            Ok(event) => match event {
                KolmeRuntimeCommitNotificationEvent::LatestBlock { height } => {
                    let upper_bound =
                        resolve_kolme_lookup_upper_bound(from_height, latest_height, height);
                    self.block_fallback_reconciler.reconcile_txhash(
                        expected_txhash.as_str(),
                        from_height,
                        upper_bound,
                    )
                }
                _ => {
                    let receipt = event
                        .to_provider_receipt(self.notifications_consumer.provider())
                        .ok_or_else(|| KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: "notification event did not carry receipt data".to_owned(),
                        })?;
                    require_kolme_commit_id_matches_expected_txhash_contract(
                        receipt.commit_id.as_str(),
                        expected_txhash.as_str(),
                    )
                    .map_err(|error| {
                        KolmeRuntimeCommitProviderError::MalformedResponse {
                            reason: error.to_string(),
                        }
                    })?;
                    Ok(receipt)
                }
            },
            Err(KolmeRuntimeCommitProviderError::Unavailable { .. })
            | Err(KolmeRuntimeCommitProviderError::Timeout) => self
                .block_fallback_reconciler
                .reconcile_txhash(expected_txhash.as_str(), from_height, latest_height),
            Err(error @ KolmeRuntimeCommitProviderError::MalformedResponse { .. }) => Err(error),
        }
    }
}
