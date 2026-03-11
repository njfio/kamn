use super::super::*;

impl ServiceApiMessageStore {
    pub(crate) fn register_content(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiContentRegisterBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "content-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut content_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.contents.contains_key(content_id.as_str()) {
            content_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.contents.insert(
            content_id.clone(),
            ServiceApiPersistedContentRecord {
                content_id: content_id.clone(),
                retention_class: "standard".to_owned(),
                lifecycle_state: "retained".to_owned(),
                redaction_status: "none".to_owned(),
            },
        );
        self.persist()?;
        Ok(ServiceApiContentRegisterBody {
            content_id,
            retention_class: "standard".to_owned(),
            lifecycle_state: "retained".to_owned(),
            redaction_status: "none".to_owned(),
        })
    }

    pub(crate) fn get_content(
        &mut self,
        content_id: &str,
    ) -> Result<Option<ServiceApiContentLifecycleBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.contents.get(content_id) else {
            return Ok(None);
        };
        Ok(Some(ServiceApiContentLifecycleBody {
            content_id: record.content_id.clone(),
            lifecycle_state: record.lifecycle_state.clone(),
            redaction_status: record.redaction_status.clone(),
        }))
    }

    pub(crate) fn expire_content(
        &mut self,
        content_id: &str,
    ) -> Result<Option<ServiceApiContentLifecycleBody>, String> {
        self.refresh_from_disk()?;
        let payload = {
            let Some(record) = self.snapshot.contents.get_mut(content_id) else {
                return Ok(None);
            };
            record.lifecycle_state = "expired".to_owned();
            record.redaction_status = "none".to_owned();
            ServiceApiContentLifecycleBody {
                content_id: record.content_id.clone(),
                lifecycle_state: record.lifecycle_state.clone(),
                redaction_status: record.redaction_status.clone(),
            }
        };
        self.persist()?;
        Ok(Some(payload))
    }

    pub(crate) fn tombstone_content(
        &mut self,
        content_id: &str,
    ) -> Result<Option<ServiceApiContentLifecycleBody>, String> {
        self.refresh_from_disk()?;
        let payload = {
            let Some(record) = self.snapshot.contents.get_mut(content_id) else {
                return Ok(None);
            };
            record.lifecycle_state = "tombstoned".to_owned();
            record.redaction_status = "redacted".to_owned();
            ServiceApiContentLifecycleBody {
                content_id: record.content_id.clone(),
                lifecycle_state: record.lifecycle_state.clone(),
                redaction_status: record.redaction_status.clone(),
            }
        };
        self.persist()?;
        Ok(Some(payload))
    }

    pub(crate) fn submit_bridge(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiBridgeSubmitBody, String> {
        self.refresh_from_disk()?;
        let bridge_tag = deterministic_body_tag(payload.as_bytes());
        let base = format!("bridge-local-{bridge_tag:016x}");
        let mut bridge_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.bridges.contains_key(bridge_id.as_str()) {
            bridge_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        let source_message_id =
            bridge_source_message_id_from_payload(payload, bridge_tag, bridge_id.as_str());
        let target_message_id = format!("msg-bridge-target-{bridge_id}");
        self.snapshot.bridges.insert(
            bridge_id.clone(),
            ServiceApiPersistedBridgeRecord {
                bridge_id: bridge_id.clone(),
                source_message_id: source_message_id.clone(),
                bridge_status: "submitted".to_owned(),
                target_message_id,
                forward_tx_hash: String::new(),
            },
        );
        self.persist()?;
        Ok(ServiceApiBridgeSubmitBody {
            bridge_id,
            source_message_id,
            bridge_status: "submitted".to_owned(),
        })
    }

    pub(crate) fn forward_bridge(
        &mut self,
        bridge_id: &str,
    ) -> Result<Option<ServiceApiBridgeStatusBody>, String> {
        self.refresh_from_disk()?;
        let payload = {
            let Some(record) = self.snapshot.bridges.get_mut(bridge_id) else {
                return Ok(None);
            };
            record.bridge_status = "forwarded".to_owned();
            if record.target_message_id.is_empty() {
                record.target_message_id = format!("msg-bridge-target-{}", record.bridge_id);
            }
            record.forward_tx_hash = format!("sha256:bridge-forwarded-{}", record.bridge_id);
            ServiceApiBridgeStatusBody {
                bridge_id: record.bridge_id.clone(),
                bridge_status: record.bridge_status.clone(),
                target_message_id: record.target_message_id.clone(),
                forward_tx_hash: record.forward_tx_hash.clone(),
            }
        };
        self.persist()?;
        Ok(Some(payload))
    }

    pub(crate) fn get_bridge(
        &mut self,
        bridge_id: &str,
    ) -> Result<Option<ServiceApiBridgeStatusBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.bridges.get(bridge_id) else {
            return Ok(None);
        };
        Ok(Some(ServiceApiBridgeStatusBody {
            bridge_id: record.bridge_id.clone(),
            bridge_status: record.bridge_status.clone(),
            target_message_id: record.target_message_id.clone(),
            forward_tx_hash: record.forward_tx_hash.clone(),
        }))
    }

}

pub(super) fn bridge_source_message_id_from_payload(
    payload: &str,
    bridge_tag: u64,
    bridge_id: &str,
) -> String {
    let default_value = format!("msg-bridge-source-{bridge_tag:016x}");
    let parsed = match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(value) => value,
        Err(_) => return default_value,
    };
    let Some(source_message_id) = parsed
        .get("source_message_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return default_value;
    };
    if source_message_id == bridge_id {
        return default_value;
    }
    source_message_id.to_owned()
}
