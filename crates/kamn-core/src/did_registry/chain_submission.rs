use crate::kolme_runtime_commit::{
    KolmeRuntimeCommitClient, KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome,
    KolmeRuntimeCommitRequest,
};
use std::collections::HashMap;
use std::path::PathBuf;

use super::models::{
    DidChainSubmissionOutcome, DidChainSubmissionReceipt, DidChainSubmissionRequest,
    DidLifecycleChainAdapter, DidLifecycleChainSubmissionRequest, DidRegistrationChainAdapter,
    PersistedDidChainAdapterState, SubmissionReceiptIndex, SubmissionRejectIndex,
};
use super::validation::DidRegistryError;

mod file_adapter;
mod in_memory_adapter;
mod kolme_adapter;
mod persistence;

pub use file_adapter::FileDidRegistrationChainAdapter;
pub use in_memory_adapter::InMemoryDidRegistrationChainAdapter;
pub use kolme_adapter::KolmeDidLifecycleChainAdapter;
pub(crate) use persistence::{persist_did_chain_adapter_file, read_did_chain_adapter_file};
