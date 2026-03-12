use super::GroupChannelCryptoEngine;
use crate::group_channel_crypto::{
    validate_did, validate_recipients, validate_sender_key_ref, GroupChannelCryptoError,
    SenderKeyDistributionRecord, GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

impl GroupChannelCryptoEngine {
    /// Constructs an engine for a specific channel identifier.
    pub fn new(channel_id: &str) -> Result<Self, GroupChannelCryptoError> {
        if channel_id.trim().is_empty() {
            return Err(GroupChannelCryptoError::EmptyChannelId);
        }

        Ok(Self {
            channel_id: channel_id.to_owned(),
            sender_key_history: BTreeMap::new(),
            active_generation_by_sender: BTreeMap::new(),
            used_nonces: BTreeSet::new(),
            cached_master_seed: RefCell::new(None),
        })
    }

    /// Distributes a new sender-key generation and marks the previous one inactive.
    pub fn distribute_sender_key(
        &mut self,
        sender_did: &str,
        sender_key_ref: &str,
        recipients: Vec<String>,
    ) -> Result<SenderKeyDistributionRecord, GroupChannelCryptoError> {
        let recipient_allowlist =
            validate_distribution_inputs(sender_did, sender_key_ref, recipients)?;
        let next_generation = next_generation(self, sender_did);
        deactivate_active_generation(self, sender_did);
        let record = new_distribution_record(
            self,
            sender_did,
            sender_key_ref,
            recipient_allowlist,
            next_generation,
        );
        store_new_generation(self, sender_did, next_generation, record.clone());
        Ok(record)
    }

    /// Rotates sender-key material by issuing a new distribution generation.
    pub fn rotate_sender_key(
        &mut self,
        sender_did: &str,
        sender_key_ref: &str,
        recipients: Vec<String>,
    ) -> Result<SenderKeyDistributionRecord, GroupChannelCryptoError> {
        self.distribute_sender_key(sender_did, sender_key_ref, recipients)
    }

    /// Returns the active sender-key generation for a sender DID.
    pub fn active_sender_key_generation(
        &self,
        sender_did: &str,
    ) -> Result<u64, GroupChannelCryptoError> {
        self.active_generation_by_sender
            .get(sender_did)
            .copied()
            .ok_or_else(|| GroupChannelCryptoError::SenderKeyNotFound(sender_did.to_owned()))
    }

    /// Returns a sender-key distribution record for a specific generation.
    pub fn sender_key_record(
        &self,
        sender_did: &str,
        key_generation: u64,
    ) -> Result<&SenderKeyDistributionRecord, GroupChannelCryptoError> {
        self.sender_key_history
            .get(sender_did)
            .and_then(|history| history.get(&key_generation))
            .ok_or_else(|| GroupChannelCryptoError::UnknownSenderKeyGeneration {
                sender_did: sender_did.to_owned(),
                key_generation,
            })
    }
}

fn validate_distribution_inputs(
    sender_did: &str,
    sender_key_ref: &str,
    recipients: Vec<String>,
) -> Result<BTreeSet<String>, GroupChannelCryptoError> {
    validate_did(
        sender_did,
        "sender_did",
        GROUP_CHANNEL_CRYPTO_INVALID_SENDER_DID_REASON_CODE,
    )?;
    validate_sender_key_ref(sender_key_ref)?;
    validate_recipients(recipients)
}

fn next_generation(engine: &GroupChannelCryptoEngine, sender_did: &str) -> u64 {
    engine
        .active_generation_by_sender
        .get(sender_did)
        .copied()
        .unwrap_or(0)
        + 1
}

fn new_distribution_record(
    engine: &GroupChannelCryptoEngine,
    sender_did: &str,
    sender_key_ref: &str,
    recipient_allowlist: BTreeSet<String>,
    key_generation: u64,
) -> SenderKeyDistributionRecord {
    SenderKeyDistributionRecord {
        channel_id: engine.channel_id.clone(),
        sender_did: sender_did.to_owned(),
        sender_key_ref: sender_key_ref.to_owned(),
        key_generation,
        recipient_allowlist,
        active: true,
    }
}

fn deactivate_active_generation(engine: &mut GroupChannelCryptoEngine, sender_did: &str) {
    if let Some(history) = engine.sender_key_history.get_mut(sender_did) {
        if let Some(active_generation) = engine.active_generation_by_sender.get(sender_did) {
            if let Some(active_record) = history.get_mut(active_generation) {
                active_record.active = false;
            }
        }
    }
}

fn store_new_generation(
    engine: &mut GroupChannelCryptoEngine,
    sender_did: &str,
    generation: u64,
    record: SenderKeyDistributionRecord,
) {
    engine
        .sender_key_history
        .entry(sender_did.to_owned())
        .or_default()
        .insert(generation, record);
    engine
        .active_generation_by_sender
        .insert(sender_did.to_owned(), generation);
}
