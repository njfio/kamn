//! M5 vector-layer contracts for embedding storage, semantic query, recall drift, and anomaly scoring.
//! This module keeps the public surface stable while routing implementation through bounded modules.

mod analytics;
mod models;
mod query;
mod registry;
mod support;
#[cfg(test)]
mod tests;

pub use analytics::*;
pub use models::*;
pub use query::*;
pub use registry::*;
