mod decrypt;
mod encrypt;

use super::GroupChannelCryptoEngine;
use crate::group_channel_crypto::{load_key_agreement_master_seed, GroupChannelCryptoError};

impl GroupChannelCryptoEngine {
    pub(super) fn cached_master_seed(&self) -> Result<[u8; 32], GroupChannelCryptoError> {
        if let Some(seed) = self.cached_master_seed.borrow().as_ref().copied() {
            return Ok(seed);
        }
        let seed = load_key_agreement_master_seed()?;
        self.cached_master_seed.borrow_mut().replace(seed);
        Ok(seed)
    }
}
