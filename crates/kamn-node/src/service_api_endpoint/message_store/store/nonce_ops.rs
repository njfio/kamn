use super::super::*;

impl ServiceApiMessageStore {
    pub(crate) fn auth_nonce_high_watermarks(&self) -> BTreeMap<String, u64> {
        self.snapshot.auth_nonce_high_watermarks.clone()
    }

    pub(crate) fn record_auth_nonce_high_watermark(
        &mut self,
        sender_did: &str,
        nonce: u64,
    ) -> Result<(), String> {
        self.refresh_from_disk()?;
        let normalized_sender = validate_nonce_sender(sender_did)?;
        if !should_update_nonce(
            self.snapshot
                .auth_nonce_high_watermarks
                .get(normalized_sender)
                .copied(),
            nonce,
        ) {
            return Ok(());
        }
        self.snapshot
            .auth_nonce_high_watermarks
            .insert(normalized_sender.to_owned(), nonce);
        self.persist()
    }
}

fn validate_nonce_sender(sender_did: &str) -> Result<&str, String> {
    let normalized_sender = sender_did.trim();
    if normalized_sender.is_empty() {
        return Err("service api auth nonce sender did must not be empty".to_owned());
    }
    Ok(normalized_sender)
}

fn should_update_nonce(current: Option<u64>, nonce: u64) -> bool {
    nonce > 0 && nonce > current.unwrap_or(0)
}
