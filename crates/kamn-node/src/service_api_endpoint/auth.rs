use super::*;

mod anti_spam;
mod request_auth;
mod scope_policy;
mod support;

#[cfg(test)]
mod tests;

pub(crate) use anti_spam::enforce_sender_anti_spam;
#[cfg(test)]
pub(super) use anti_spam::map_anti_spam_rejection_to_reasoned_error;
pub(crate) use request_auth::authorize_service_api_request;
pub(crate) use scope_policy::enforce_request_scope_policy;
pub(crate) use support::header_value;
