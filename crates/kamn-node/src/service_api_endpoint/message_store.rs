use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedMessageRecord {
    message_id: String,
    status: String,
    channel_id: Option<String>,
    #[serde(default)]
    sender_did: Option<String>,
    #[serde(default)]
    recipient_did: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedTaskRecord {
    task_id: String,
    state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedEscrowRecord {
    escrow_id: String,
    state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedMessageStoreSnapshot {
    schema_version: String,
    messages: BTreeMap<String, ServiceApiPersistedMessageRecord>,
    channel_messages: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    tasks: BTreeMap<String, ServiceApiPersistedTaskRecord>,
    #[serde(default)]
    escrows: BTreeMap<String, ServiceApiPersistedEscrowRecord>,
}

impl Default for ServiceApiPersistedMessageStoreSnapshot {
    fn default() -> Self {
        Self {
            schema_version: "kamn.runtime.service-api-message-store.v2".to_owned(),
            messages: BTreeMap::new(),
            channel_messages: BTreeMap::new(),
            tasks: BTreeMap::new(),
            escrows: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServiceApiMessageStore {
    state_file: Option<String>,
    snapshot: ServiceApiPersistedMessageStoreSnapshot,
}

impl ServiceApiMessageStore {
    pub(super) fn from_optional_state_file(state_file: Option<String>) -> Result<Self, String> {
        let snapshot = if let Some(path) = state_file.as_deref() {
            match fs::read_to_string(path) {
                Ok(contents) => serde_json::from_str::<ServiceApiPersistedMessageStoreSnapshot>(
                    contents.as_str(),
                )
                .map_err(|error| format!("service api state file parse failed: {path}: {error}"))?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    ServiceApiPersistedMessageStoreSnapshot::default()
                }
                Err(error) => {
                    return Err(format!(
                        "service api state file read failed: {path}: {error}"
                    ));
                }
            }
        } else {
            ServiceApiPersistedMessageStoreSnapshot::default()
        };
        Ok(Self {
            state_file,
            snapshot,
        })
    }

    fn persist(&self) -> Result<(), String> {
        let Some(path) = self.state_file.as_deref() else {
            return Ok(());
        };
        let payload = serde_json::to_string_pretty(&self.snapshot)
            .map_err(|error| format!("service api state serialization failed: {error}"))?;
        fs::write(path, payload)
            .map_err(|error| format!("service api state file write failed: {path}: {error}"))
    }

    pub(super) fn create_message(
        &mut self,
        payload: &str,
        runtime_mode: &str,
        channel_id: Option<&str>,
        sender_did: Option<&str>,
        recipient_did: Option<&str>,
    ) -> Result<ServiceApiMessageCreateBody, String> {
        let base = format!(
            "msg-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut message_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.messages.contains_key(message_id.as_str()) {
            message_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }

        self.snapshot.messages.insert(
            message_id.clone(),
            ServiceApiPersistedMessageRecord {
                message_id: message_id.clone(),
                status: "created".to_owned(),
                channel_id: channel_id.map(str::to_owned),
                sender_did: sender_did.map(str::to_owned),
                recipient_did: recipient_did.map(str::to_owned),
                body: Some(payload.to_owned()),
            },
        );
        if let Some(channel_id) = channel_id {
            self.snapshot
                .channel_messages
                .entry(channel_id.to_owned())
                .or_default()
                .push(message_id.clone());
        }
        if let Some(recipient_did) = recipient_did {
            self.snapshot
                .channel_messages
                .entry(recipient_mailbox_channel_id(recipient_did))
                .or_default()
                .push(message_id.clone());
        }
        self.persist()?;
        Ok(ServiceApiMessageCreateBody {
            message_id,
            status: "created".to_owned(),
            runtime_mode: runtime_mode.to_owned(),
        })
    }

    pub(super) fn get_message_for_requester(
        &mut self,
        message_id: &str,
        requester_did: Option<&str>,
    ) -> Result<Option<ServiceApiMessageGetBody>, String> {
        let Some(record) = self.snapshot.messages.get_mut(message_id) else {
            return Ok(None);
        };
        let should_mark_delivered = record.status == "created"
            && requester_did.is_some()
            && record.recipient_did.as_deref() == requester_did;
        let payload = if should_mark_delivered {
            record.status = "delivered".to_owned();
            ServiceApiMessageGetBody {
                message_id: record.message_id.clone(),
                status: record.status.clone(),
                sender_did: record.sender_did.clone(),
                recipient_did: record.recipient_did.clone(),
                body: record.body.clone(),
            }
        } else {
            ServiceApiMessageGetBody {
                message_id: record.message_id.clone(),
                status: record.status.clone(),
                sender_did: record.sender_did.clone(),
                recipient_did: record.recipient_did.clone(),
                body: record.body.clone(),
            }
        };
        let _ = record;
        if should_mark_delivered {
            self.persist()?;
        }
        Ok(Some(payload))
    }

    pub(super) fn list_channel_messages(&self, channel_id: &str) -> ServiceApiChannelMessagesBody {
        ServiceApiChannelMessagesBody {
            channel_id: channel_id.to_owned(),
            messages: self
                .snapshot
                .channel_messages
                .get(channel_id)
                .cloned()
                .unwrap_or_default(),
        }
    }

    pub(super) fn create_task(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiTaskCreateBody, String> {
        let base = format!(
            "task-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut task_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.tasks.contains_key(task_id.as_str()) {
            task_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.tasks.insert(
            task_id.clone(),
            ServiceApiPersistedTaskRecord {
                task_id: task_id.clone(),
                state: "submitted".to_owned(),
            },
        );
        self.persist()?;
        Ok(ServiceApiTaskCreateBody {
            task_id,
            state: "submitted".to_owned(),
        })
    }

    pub(super) fn get_task(&self, task_id: &str) -> Option<ServiceApiTaskGetBody> {
        let record = self.snapshot.tasks.get(task_id)?;
        Some(ServiceApiTaskGetBody {
            task_id: record.task_id.clone(),
            state: record.state.clone(),
        })
    }

    pub(super) fn transition_task(
        &mut self,
        task_id: &str,
        state: &str,
    ) -> Result<ServiceApiTaskTransitionBody, String> {
        let record = self.snapshot.tasks.entry(task_id.to_owned()).or_insert(
            ServiceApiPersistedTaskRecord {
                task_id: task_id.to_owned(),
                state: "submitted".to_owned(),
            },
        );
        record.state = state.to_owned();
        self.persist()?;
        Ok(ServiceApiTaskTransitionBody {
            task_id: task_id.to_owned(),
            state: state.to_owned(),
        })
    }

    pub(super) fn fund_escrow(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiEscrowStatusBody, String> {
        let base = format!(
            "escrow-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut escrow_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.escrows.contains_key(escrow_id.as_str()) {
            escrow_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.escrows.insert(
            escrow_id.clone(),
            ServiceApiPersistedEscrowRecord {
                escrow_id: escrow_id.clone(),
                state: "funded".to_owned(),
            },
        );
        self.persist()?;
        Ok(ServiceApiEscrowStatusBody {
            escrow_id,
            state: "funded".to_owned(),
        })
    }

    pub(super) fn release_escrow(
        &mut self,
        escrow_id: &str,
    ) -> Result<ServiceApiEscrowStatusBody, String> {
        let record = self.snapshot.escrows.entry(escrow_id.to_owned()).or_insert(
            ServiceApiPersistedEscrowRecord {
                escrow_id: escrow_id.to_owned(),
                state: "funded".to_owned(),
            },
        );
        record.state = "released".to_owned();
        self.persist()?;
        Ok(ServiceApiEscrowStatusBody {
            escrow_id: escrow_id.to_owned(),
            state: "released".to_owned(),
        })
    }
}

fn recipient_mailbox_channel_id(recipient_did: &str) -> String {
    format!("recipient:{recipient_did}")
}
