use super::{
    GroupChannelCryptoError, GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE,
    KEY_AGREEMENT_MASTER_SEED_ENV,
};
use crate::AgentDid;
use std::collections::BTreeSet;
use std::env;
use zeroize::Zeroize;

pub(super) fn validate_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, GroupChannelCryptoError> {
    AgentDid::parse(value).map_err(|error| GroupChannelCryptoError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

pub(super) fn validate_sender_key_ref(value: &str) -> Result<(), GroupChannelCryptoError> {
    if value.contains("#sender-key-") {
        return Ok(());
    }
    Err(GroupChannelCryptoError::InvalidSenderKeyRef)
}

pub(super) fn load_key_agreement_master_seed() -> Result<[u8; 32], GroupChannelCryptoError> {
    let mut seed_hex = env::var(KEY_AGREEMENT_MASTER_SEED_ENV)
        .map_err(|_| GroupChannelCryptoError::MissingKeyAgreementMasterSeed)?;
    let seed = parse_fixed_hex_32(seed_hex.trim());
    seed_hex.zeroize();
    seed
}

fn parse_fixed_hex_32(value: &str) -> Result<[u8; 32], GroupChannelCryptoError> {
    if value.len() != 64 {
        return Err(GroupChannelCryptoError::InvalidKeyAgreementMasterSeed);
    }

    let mut out = [0u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let encoded = std::str::from_utf8(chunk)
            .map_err(|_| GroupChannelCryptoError::InvalidKeyAgreementMasterSeed)?;
        out[index] = u8::from_str_radix(encoded, 16)
            .map_err(|_| GroupChannelCryptoError::InvalidKeyAgreementMasterSeed)?;
    }
    Ok(out)
}

pub(super) fn validate_recipients(
    recipients: Vec<String>,
) -> Result<BTreeSet<String>, GroupChannelCryptoError> {
    if recipients.is_empty() {
        return Err(GroupChannelCryptoError::EmptyRecipients);
    }

    let mut allowlist = BTreeSet::new();
    for recipient in recipients {
        validate_did(
            &recipient,
            "recipients[]",
            GROUP_CHANNEL_CRYPTO_INVALID_RECIPIENT_DID_REASON_CODE,
        )?;
        allowlist.insert(recipient);
    }
    Ok(allowlist)
}
