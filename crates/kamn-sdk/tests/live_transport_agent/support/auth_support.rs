use super::*;

#[path = "auth_support/request_auth_support.rs"]
mod request_auth_support;
#[path = "auth_support/route_scope_support.rs"]
mod route_scope_support;

pub(crate) use request_auth_support::validate_auth;
