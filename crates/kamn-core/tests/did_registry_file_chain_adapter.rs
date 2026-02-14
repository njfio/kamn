use kamn_core::{
    canonical_did_document, AgentDid, AgentDidMetadata, DidChainSubmissionOutcome, DidDocument,
    DidRegistry, DidRegistryError, FileDidRegistrationChainAdapter,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn metadata(model_family: &str) -> AgentDidMetadata {
    AgentDidMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: model_family.to_owned(),
        capabilities: vec!["text".to_owned()],
        operator: None,
    }
}

fn document_for(did: &AgentDid, model_family: &str) -> DidDocument {
    canonical_did_document(did, "z6Mpub", metadata(model_family))
        .expect("did document should build")
}

fn unique_temp_file(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should advance")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-did-chain-adapter-{tag}-{}-{nonce}.log",
        std::process::id()
    ))
}

#[test]
fn did_registry_file_chain_adapter_persists_duplicate_detection_across_restart() {
    let path = unique_temp_file("duplicate");

    let mut first_registry = DidRegistry::new();
    let mut first_adapter =
        FileDidRegistrationChainAdapter::new(path.clone(), "ledger-file").expect("adapter");
    let did = AgentDid::parse("kamn:did:agent:file-adapter-1").expect("did should parse");
    let document = document_for(&did, "gpt-5");

    let first = first_registry
        .submit_registration_via_chain_adapter(&mut first_adapter, did.clone(), document.clone())
        .expect("first submit should succeed");
    assert!(matches!(
        first.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));

    let mut second_registry = DidRegistry::new();
    let mut second_adapter =
        FileDidRegistrationChainAdapter::new(path.clone(), "ledger-file").expect("adapter");
    let second = second_registry
        .submit_registration_via_chain_adapter(&mut second_adapter, did, document)
        .expect("retry submit should succeed");
    assert!(matches!(
        second.outcome,
        DidChainSubmissionOutcome::Duplicate(_)
    ));

    fs::remove_file(path).expect("cleanup should succeed");
}

#[test]
fn did_registry_file_chain_adapter_regression_rejects_corrupt_payload_line() {
    // Regression: #2902
    let path = unique_temp_file("corrupt");
    fs::write(&path, "schema|kamn.did.chain-adapter.v1\nreceipt|broken\n")
        .expect("fixture write should succeed");

    let result = FileDidRegistrationChainAdapter::new(path.clone(), "ledger-file");
    assert!(matches!(
        result,
        Err(DidRegistryError::PersistenceInvalidPayload(value)) if value.contains("receipt|broken")
    ));

    fs::remove_file(path).expect("cleanup should succeed");
}
