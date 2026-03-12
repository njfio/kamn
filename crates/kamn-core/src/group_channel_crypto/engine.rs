mod lifecycle;
mod sealing;

use super::{
    zeroize_sender_key_history, zeroize_u64_keyed_sender_history,
    SenderKeyDistributionRecord,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use zeroize::Zeroize;

/// In-memory engine for sender-key distribution, rotation, and message sealing.
#[derive(PartialEq, Eq)]
pub struct GroupChannelCryptoEngine {
    pub(super) channel_id: String,
    pub(super) sender_key_history: BTreeMap<String, BTreeMap<u64, SenderKeyDistributionRecord>>,
    pub(super) active_generation_by_sender: BTreeMap<String, u64>,
    pub(super) used_nonces: BTreeSet<(String, u64, u64)>,
    pub(super) cached_master_seed: RefCell<Option<[u8; 32]>>,
}

impl fmt::Debug for GroupChannelCryptoEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupChannelCryptoEngine")
            .field("channel_id", &self.channel_id)
            .field("sender_count", &self.sender_key_history.len())
            .field("active_sender_count", &self.active_generation_by_sender.len())
            .field("used_nonce_count", &self.used_nonces.len())
            .finish()
    }
}

impl Drop for GroupChannelCryptoEngine {
    fn drop(&mut self) {
        self.channel_id.zeroize();
        zeroize_sender_key_history(&mut self.sender_key_history);
        zeroize_u64_keyed_sender_history(&mut self.active_generation_by_sender);
        self.used_nonces.clear();
        if let Some(seed) = self.cached_master_seed.get_mut().as_mut() {
            seed.zeroize();
        }
    }
}
