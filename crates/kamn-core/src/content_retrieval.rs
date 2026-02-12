//! Content retrieval authorization, caching, and audit-event contracts.

use crate::{
    AgentDid, ChannelAction, ChannelPermissionEngine, ChannelPolicyError, ContentStorageAdapter,
    ContentStorageError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retrieval engine cache configuration.
pub struct ContentRetrievalConfig {
    /// Maximum seconds cached retrieval entries remain valid.
    pub cache_ttl_secs: u64,
}

impl ContentRetrievalConfig {
    /// Builds a validated retrieval config.
    pub fn new(cache_ttl_secs: u64) -> Result<Self, ContentRetrievalError> {
        if cache_ttl_secs == 0 {
            return Err(ContentRetrievalError::InvalidConfig("cache_ttl_secs"));
        }
        Ok(Self { cache_ttl_secs })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Resource scope requested for content retrieval.
pub enum ContentRetrievalScope {
    /// Channel-scoped retrieval keyed by channel identifier.
    Channel(String),
    /// Task-scoped retrieval keyed by task identifier.
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
/// Authenticated retrieval request envelope.
pub struct ContentRetrievalRequest {
    /// Content identifier requested by caller.
    pub cid: String,
    /// Requester DID initiating the read.
    pub requester: String,
    /// Retrieval scope controlling authorization path.
    pub scope: ContentRetrievalScope,
    /// Unix timestamp when the request was created.
    pub requested_at_unix: u64,
}

impl ContentRetrievalRequest {
    /// Constructs a validated retrieval request.
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
/// Authorization outcome recorded for retrieval attempts.
pub enum ContentRetrievalOutcome {
    /// Request was authorized and retrieval completed.
    Allowed,
    /// Request failed authorization.
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Audit event emitted for each retrieval decision.
pub struct ContentRetrievalAuditEvent {
    /// Content identifier targeted by the request.
    pub cid: String,
    /// Requester DID that initiated retrieval.
    pub requester: String,
    /// Scope used during authorization.
    pub scope: ContentRetrievalScope,
    /// Unix timestamp when retrieval was requested.
    pub requested_at_unix: u64,
    /// Authorization decision outcome.
    pub outcome: ContentRetrievalOutcome,
    /// Whether payload came from cache.
    pub cache_hit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retrieval result payload returned to authorized callers.
pub struct ContentRetrievalResult {
    /// Content identifier returned by storage.
    pub cid: String,
    /// MIME media type associated with payload.
    pub media_type: String,
    /// Raw content payload bytes.
    pub payload: Vec<u8>,
    /// True when result came from cache.
    pub from_cache: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retrieval engine implementing scope-based authorization and cache behavior.
pub struct ContentRetrievalEngine {
    config: ContentRetrievalConfig,
    task_read_allowlist: BTreeMap<String, BTreeSet<String>>,
    cache: BTreeMap<String, CachedContent>,
    audit_events: Vec<ContentRetrievalAuditEvent>,
}

impl ContentRetrievalEngine {
    /// Creates a retrieval engine from validated config.
    pub fn new(config: ContentRetrievalConfig) -> Self {
        Self {
            config,
            task_read_allowlist: BTreeMap::new(),
            cache: BTreeMap::new(),
            audit_events: Vec::new(),
        }
    }

    /// Grants task-scoped read permission to a requester DID.
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

    /// Invalidates cached retrieval entries for a CID.
    pub fn invalidate_cache_for_cid(&mut self, cid: &str) {
        self.cache.retain(|_, entry| entry.cid != cid);
    }

    /// Returns retrieval audit events accumulated by this engine.
    pub fn audit_events(&self) -> Vec<ContentRetrievalAuditEvent> {
        self.audit_events.clone()
    }

    /// Retrieves content for an authorized request, applying cache policy.
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
/// Retrieval engine error taxonomy.
pub enum ContentRetrievalError {
    /// Retrieval config contained invalid values.
    InvalidConfig(&'static str),
    /// A required field was empty or zero.
    EmptyField(&'static str),
    /// Requester DID failed parsing/validation.
    InvalidDid(String),
    /// Requester is not authorized for target scope.
    Unauthorized {
        /// Requester DID rejected by authorization.
        requester: String,
        /// Scope where authorization failed.
        scope: ContentRetrievalScope,
    },
    /// Channel permission engine was required but not provided.
    MissingChannelPermissions(String),
    /// Channel authorization call returned a policy error.
    ChannelPolicy(ChannelPolicyError),
    /// Storage adapter returned retrieval/validation failure.
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
