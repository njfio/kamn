use super::super::*;

mod support;

use support::{
    bridge_source_message_id_from_payload, bridge_submit_body, build_bridge_record,
    build_content_record, content_register_body, next_bridge_id, next_content_id,
};

impl ServiceApiMessageStore {
    pub(crate) fn register_content(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiContentRegisterBody, String> {
        self.refresh_from_disk()?;
        let content_id = next_content_id(self, payload);
        self.snapshot.contents.insert(
            content_id.clone(),
            build_content_record(content_id.as_str()),
        );
        self.persist()?;
        Ok(content_register_body(content_id))
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
        let bridge_id = next_bridge_id(self, bridge_tag);
        let source_message_id =
            bridge_source_message_id_from_payload(payload, bridge_tag, bridge_id.as_str());
        self.snapshot.bridges.insert(
            bridge_id.clone(),
            build_bridge_record(bridge_id.as_str(), source_message_id.as_str()),
        );
        self.persist()?;
        Ok(bridge_submit_body(bridge_id, source_message_id))
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
