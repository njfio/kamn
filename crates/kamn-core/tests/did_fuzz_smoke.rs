use kamn_core::{
    canonical_did_document, AgentDid, AgentDidError, AgentDidMetadata, DidDocumentError,
};
use std::panic::catch_unwind;

fn mutation_slot(seed: u64, slots: usize) -> usize {
    let mixed = seed
        .wrapping_mul(2862933555777941757)
        .wrapping_add(3037000493);
    (mixed % slots as u64) as usize
}

fn mutate_did_case(seed: u64) -> String {
    let slot = mutation_slot(seed, 12);
    let suffix = (seed % 10_000).to_string();
    match slot {
        0 => format!("kamn:did:agent:agent-{suffix}"),
        1 => format!("did:example:agent-{suffix}"),
        2 => format!("KAMN:DID:AGENT:agent-{suffix}"),
        3 => "kamn:did:agent:".to_owned(),
        4 => format!("kamn:did:agent:agent {suffix}"),
        5 => format!("kamn:did:agent:agent+{suffix}"),
        6 => format!("kamn:did:agent:agent_{suffix}"),
        7 => format!("kamn:did:agent:agent-{suffix}-beta"),
        8 => format!("kamn:did:agent:agent.{suffix}"),
        9 => format!("kamn:did:agent:agent-{suffix}\n"),
        10 => format!("kamn:did:agent:{}", "a".repeat((seed % 64 + 1) as usize)),
        11 => format!("kamn:did:agent:agent-{suffix}-{}", "x".repeat(16)),
        _ => unreachable!("mutation slot is bounded"),
    }
}

fn metadata_from_seed(seed: u64) -> AgentDidMetadata {
    let slot = mutation_slot(seed.rotate_left(11), 8);
    match slot {
        0 => AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        },
        1 => AgentDidMetadata {
            agent_type: String::new(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        },
        2 => AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: String::new(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        },
        3 => AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec![],
            operator: None,
        },
        4 => AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["".to_owned()],
            operator: None,
        },
        5 => AgentDidMetadata {
            agent_type: "agentic".to_owned(),
            model_family: "gpt-5".to_owned(),
            capabilities: vec!["coordination".to_owned(), "planning".to_owned()],
            operator: Some("kamn:did:human:operator-1".to_owned()),
        },
        6 => AgentDidMetadata {
            agent_type: " ".to_owned(),
            model_family: "gpt-5".to_owned(),
            capabilities: vec!["planning".to_owned()],
            operator: None,
        },
        7 => AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: " ".to_owned(),
            capabilities: vec!["planning".to_owned()],
            operator: None,
        },
        _ => unreachable!("metadata slot is bounded"),
    }
}

#[test]
fn fuzz_smoke_did_parse_mutations_are_panic_free_and_deterministic() {
    for seed in 0_u64..1024 {
        let did = mutate_did_case(seed);
        let first = catch_unwind(|| AgentDid::parse(&did));
        assert!(first.is_ok(), "did parse panicked for seed {seed}: {did:?}");

        let first = first.expect("panic-free result should unwrap");
        let second = AgentDid::parse(&did);
        assert_eq!(
            first, second,
            "did parse result should be deterministic for seed {seed}: {did:?}"
        );

        if let Ok(parsed) = first {
            assert_eq!(parsed.as_str(), did);
            assert!(parsed
                .method_specific_id()
                .chars()
                .all(|ch| ch.is_ascii_lowercase()
                    || ch.is_ascii_digit()
                    || ch == '-'
                    || ch == '_'));
        }
    }
}

#[test]
fn fuzz_smoke_did_error_corpus_covers_expected_rejection_classes() {
    assert!(matches!(
        AgentDid::parse("did:example:agent-1"),
        Err(AgentDidError::InvalidPrefix(_))
    ));
    assert_eq!(
        AgentDid::parse("kamn:did:agent:"),
        Err(AgentDidError::MissingMethodSpecificId)
    );
    assert!(matches!(
        AgentDid::parse("kamn:did:agent:agent 1"),
        Err(AgentDidError::InvalidCharacter(_))
    ));
    assert!(AgentDid::parse("kamn:did:agent:agent_1").is_ok());
}

#[test]
fn fuzz_smoke_did_document_generation_lane_is_panic_free_and_deterministic() {
    for seed in 0_u64..256 {
        let did = AgentDid::parse(&format!("kamn:did:agent:doc-{seed:03}"))
            .expect("generated did should always parse");
        let metadata = metadata_from_seed(seed);
        let public_key = if mutation_slot(seed, 4) == 0 {
            ""
        } else {
            "z6Mkey"
        };

        let first = catch_unwind(|| canonical_did_document(&did, public_key, metadata.clone()));
        assert!(
            first.is_ok(),
            "did document generation panicked for seed {seed}"
        );

        let first = first.expect("panic-free result should unwrap");
        let second = canonical_did_document(&did, public_key, metadata);
        assert_eq!(first, second, "did document result drift for seed {seed}");

        match first {
            Ok(document) => {
                assert_eq!(document.id, did.as_str());
                assert_eq!(document.controller, did.as_str());
                assert!(document.service[0]
                    .service_endpoint
                    .starts_with("kamn://messaging/"));
            }
            Err(
                DidDocumentError::EmptyPublicKey
                | DidDocumentError::EmptyAgentType
                | DidDocumentError::EmptyModelFamily
                | DidDocumentError::MissingCapabilities
                | DidDocumentError::InvalidCapability,
            ) => {}
        }
    }
}
