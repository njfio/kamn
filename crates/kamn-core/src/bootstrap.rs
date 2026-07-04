//! Bootstrap planning for validated config, schema migrations, and runtime wiring.

mod entrypoints;
mod error_mapping;
mod layout;
mod models;
#[cfg(test)]
mod tests;
mod validation;

pub use entrypoints::{bootstrap, bootstrap_from_state_version, bootstrap_with_transport_profile};
pub use models::BootstrapPlan;
