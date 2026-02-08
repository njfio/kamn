use crate::{
    AgentDid, ChannelAction, ChannelPermissionEngine, ChannelPolicyError, ContentStorageAdapter,
    ContentStorageError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRetrievalConfig {
    pub cache_ttl_secs: u64,
}

impl ContentRetrievalConfig {
    pub fn new(cache_ttl_secs: u64) -> Result<Self, ContentRetrievalError> {
        if cache_ttl_secs == 0 {
            return Err(ContentRetrievalError::InvalidConfig("cache_ttl_secs"));
        }
        Ok(Self { cache_ttl_secs })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentRetrievalScope {
    Channel(String),
    Task(String),
}

impl ContentRetrievalScope {
    fn key(&self) -> String {
        match self {
            Self::Channel(value) => format!("channel:{value}"),
            Self::Task(value) => format!("task:{value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRetrievalRequest {
    pub cid: String,
    pub requester: String,
    pub scope: ContentRetrievalScope,
    pub requested_at_unix: u64,
}

impl ContentRetrievalRequest {
    pub fn new(
        cid: &str,
        requester: &str,
        scope: ContentRetrievalScope,
        requested_at_unix: u64,
    ) -> Result<Self, ContentRetrievalError> {
        if cid.trim().is_empty() {
            return Err(ContentRetrievalError::EmptyField("cid"));
        }
        validate_did(requester)?;
        validate_scope(&scope)?;
        if requested_at_unix == 0 {
            return Err(ContentRetrievalError::EmptyField("requested_at_unix"));
        }

        Ok(Self {
            cid: cid.to_owned(),
            requester: requester.to_owned(),
            scope,
            requested_at_unix,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentRetrievalOutcome {
    Allowed,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRetrievalAuditEvent {
    pub cid: String,
    pub requester: String,
    pub scope: ContentRetrievalScope,
    pub requested_at_unix: u64,
    pub outcome: ContentRetrievalOutcome,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRetrievalResult {
    pub cid: String,
    pub media_type: String,
    pub payload: Vec<u8>,
    pub from_cache: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRetrievalEngine {
    config: ContentRetrievalConfig,
    task_read_allowlist: BTreeMap<String, BTreeSet<String>>,
    cache: BTreeMap<String, CachedContent>,
    audit_events: Vec<ContentRetrievalAuditEvent>,
}

impl ContentRetrievalEngine {
    pub fn new(config: ContentRetrievalConfig) -> Self {
        Self {
            config,
            task_read_allowlist: BTreeMap::new(),
            cache: BTreeMap::new(),
            audit_events: Vec::new(),
        }
    }

    pub fn grant_task_read(
        &mut self,
        task_id: &str,
        requester: &str,
    ) -> Result<(), ContentRetrievalError> {
        if task_id.trim().is_empty() {
            return Err(ContentRetrievalError::EmptyField("task_id"));
        }
        validate_did(requester)?;
        self.task_read_allowlist
            .entry(task_id.to_owned())
            .or_default()
            .insert(requester.to_owned());
        Ok(())
    }

    pub fn invalidate_cache_for_cid(&mut self, cid: &str) {
        self.cache.retain(|_, entry| entry.cid != cid);
    }

    pub fn audit_events(&self) -> Vec<ContentRetrievalAuditEvent> {
        self.audit_events.clone()
    }

    pub fn retrieve<A: ContentStorageAdapter>(
        &mut self,
        adapter: &A,
        request: &ContentRetrievalRequest,
        channel_permissions: Option<&ChannelPermissionEngine>,
    ) -> Result<ContentRetrievalResult, ContentRetrievalError> {
        if let Err(error) = self.authorize(request, channel_permissions) {
            self.audit_events.push(ContentRetrievalAuditEvent {
                cid: request.cid.clone(),
                requester: request.requester.clone(),
                scope: request.scope.clone(),
                requested_at_unix: request.requested_at_unix,
                outcome: ContentRetrievalOutcome::Denied,
                cache_hit: false,
            });
            return Err(error);
        }

        let cache_key = cache_key(request);
        if let Some(cached) = self.cache.get(&cache_key) {
            if request.requested_at_unix <= cached.expires_at_unix {
                let result = ContentRetrievalResult {
                    cid: cached.cid.clone(),
                    media_type: cached.media_type.clone(),
                    payload: cached.payload.clone(),
                    from_cache: true,
                };
                self.audit_events.push(ContentRetrievalAuditEvent {
                    cid: request.cid.clone(),
                    requester: request.requester.clone(),
                    scope: request.scope.clone(),
                    requested_at_unix: request.requested_at_unix,
                    outcome: ContentRetrievalOutcome::Allowed,
                    cache_hit: true,
                });
                return Ok(result);
            }
        }

        adapter
            .verify(&request.cid)
            .map_err(ContentRetrievalError::Storage)?;
        let object = adapter
            .get(&request.cid)
            .map_err(ContentRetrievalError::Storage)?;
        let expires_at_unix = request
            .requested_at_unix
            .saturating_add(self.config.cache_ttl_secs);

        self.cache.insert(
            cache_key,
            CachedContent {
                cid: object.cid.clone(),
                media_type: object.media_type.clone(),
                payload: object.payload.clone(),
                expires_at_unix,
            },
        );

        let result = ContentRetrievalResult {
            cid: object.cid,
            media_type: object.media_type,
            payload: object.payload,
            from_cache: false,
        };
        self.audit_events.push(ContentRetrievalAuditEvent {
            cid: request.cid.clone(),
            requester: request.requester.clone(),
            scope: request.scope.clone(),
            requested_at_unix: request.requested_at_unix,
            outcome: ContentRetrievalOutcome::Allowed,
            cache_hit: false,
        });
        Ok(result)
    }

    fn authorize(
        &self,
        request: &ContentRetrievalRequest,
        channel_permissions: Option<&ChannelPermissionEngine>,
    ) -> Result<(), ContentRetrievalError> {
        match &request.scope {
            ContentRetrievalScope::Task(task_id) => {
                let allowed = self
                    .task_read_allowlist
                    .get(task_id)
                    .map(|allowlist| allowlist.contains(&request.requester))
                    .unwrap_or(false);
                if !allowed {
                    return Err(ContentRetrievalError::Unauthorized {
                        requester: request.requester.clone(),
                        scope: request.scope.clone(),
                    });
                }
                Ok(())
            }
            ContentRetrievalScope::Channel(channel_id) => {
                let Some(engine) = channel_permissions else {
                    return Err(ContentRetrievalError::MissingChannelPermissions(
                        channel_id.clone(),
                    ));
                };
                engine
                    .authorize(channel_id, &request.requester, ChannelAction::Read)
                    .map_err(ContentRetrievalError::ChannelPolicy)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentRetrievalError {
    InvalidConfig(&'static str),
    EmptyField(&'static str),
    InvalidDid(String),
    Unauthorized {
        requester: String,
        scope: ContentRetrievalScope,
    },
    MissingChannelPermissions(String),
    ChannelPolicy(ChannelPolicyError),
    Storage(ContentStorageError),
}

impl fmt::Display for ContentRetrievalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(field) => write!(f, "invalid retrieval config field: {field}"),
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::Unauthorized { requester, scope } => {
                write!(f, "unauthorized requester {requester} for scope {scope:?}")
            }
            Self::MissingChannelPermissions(channel_id) => {
                write!(
                    f,
                    "missing channel permission engine for channel scope {channel_id}"
                )
            }
            Self::ChannelPolicy(error) => write!(f, "channel policy error: {error}"),
            Self::Storage(error) => write!(f, "storage retrieval error: {error}"),
        }
    }
}

impl std::error::Error for ContentRetrievalError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CachedContent {
    cid: String,
    media_type: String,
    payload: Vec<u8>,
    expires_at_unix: u64,
}

fn cache_key(request: &ContentRetrievalRequest) -> String {
    format!(
        "{}|{}|{}",
        request.requester,
        request.scope.key(),
        request.cid
    )
}

fn validate_did(value: &str) -> Result<(), ContentRetrievalError> {
    AgentDid::parse(value).map_err(|error| ContentRetrievalError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn validate_scope(scope: &ContentRetrievalScope) -> Result<(), ContentRetrievalError> {
    match scope {
        ContentRetrievalScope::Channel(value) => {
            if value.trim().is_empty() {
                return Err(ContentRetrievalError::EmptyField("channel_id"));
            }
        }
        ContentRetrievalScope::Task(value) => {
            if value.trim().is_empty() {
                return Err(ContentRetrievalError::EmptyField("task_id"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        cache_key, ContentRetrievalConfig, ContentRetrievalError, ContentRetrievalRequest,
        ContentRetrievalScope,
    };

    #[test]
    fn config_rejects_zero_ttl() {
        assert_eq!(
            ContentRetrievalConfig::new(0),
            Err(ContentRetrievalError::InvalidConfig("cache_ttl_secs"))
        );
    }

    #[test]
    fn request_rejects_invalid_did() {
        assert_eq!(
            ContentRetrievalRequest::new(
                "kamn:cid:v1:aaaaaaaaaaaaaaaa",
                "invalid-did",
                ContentRetrievalScope::Task("task-1".to_owned()),
                1,
            ),
            Err(ContentRetrievalError::InvalidDid(
                "invalid agent did prefix: invalid-did".to_owned()
            ))
        );
    }

    #[test]
    fn cache_key_is_scope_and_requester_specific() {
        let request = ContentRetrievalRequest::new(
            "kamn:cid:v1:aaaaaaaaaaaaaaaa",
            "kamn:did:agent:reader-1",
            ContentRetrievalScope::Task("task-1".to_owned()),
            1,
        )
        .expect("request should be valid");
        assert_eq!(
            cache_key(&request),
            "kamn:did:agent:reader-1|task:task-1|kamn:cid:v1:aaaaaaaaaaaaaaaa".to_owned()
        );
    }
}
