use kamn_core::{
    canonical_did_document, AgentDid, AgentDidError, AgentDidMetadata, DidDocumentError,
};
use std::panic::catch_unwind;
use std::time::Instant;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DidMutationClass {
    Normalization,
    Encoding,
    MethodMismatch,
}

#[derive(Debug, Clone)]
struct DidMutationCase {
    id: &'static str,
    class: DidMutationClass,
    did: &'static str,
    expected_error: AgentDidError,
}

fn deterministic_did_mutation_cases() -> Vec<DidMutationCase> {
    vec![
        DidMutationCase {
            id: "normalization-uppercase-prefix",
            class: DidMutationClass::Normalization,
            did: "KAMN:DID:AGENT:agent-1",
            expected_error: AgentDidError::InvalidPrefix("KAMN:DID:AGENT:agent-1".to_owned()),
        },
        DidMutationCase {
            id: "encoding-plus-character",
            class: DidMutationClass::Encoding,
            did: "kamn:did:agent:agent+1",
            expected_error: AgentDidError::InvalidCharacter("agent+1".to_owned()),
        },
        DidMutationCase {
            id: "method-mismatch-prefix",
            class: DidMutationClass::MethodMismatch,
            did: "did:example:agent-1",
            expected_error: AgentDidError::InvalidPrefix("did:example:agent-1".to_owned()),
        },
    ]
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

#[test]
fn functional_did_mutation_suite_covers_normalization_encoding_and_method_mismatch_classes() {
    let cases = deterministic_did_mutation_cases();
    let mut saw_normalization = false;
    let mut saw_encoding = false;
    let mut saw_method_mismatch = false;

    for case in &cases {
        let actual = AgentDid::parse(case.did).expect_err("mutation case must fail closed");
        assert_eq!(
            actual, case.expected_error,
            "unexpected fail-closed reason for did case {}",
            case.id
        );
        match case.class {
            DidMutationClass::Normalization => saw_normalization = true,
            DidMutationClass::Encoding => saw_encoding = true,
            DidMutationClass::MethodMismatch => saw_method_mismatch = true,
        }
    }

    assert!(
        saw_normalization,
        "normalization mutation class must be covered"
    );
    assert!(saw_encoding, "encoding mutation class must be covered");
    assert!(
        saw_method_mismatch,
        "method mismatch mutation class must be covered"
    );
}

#[test]
fn integration_did_mutation_fail_closed_reasons_are_explicit_and_deterministic() {
    for case in deterministic_did_mutation_cases() {
        let first = AgentDid::parse(case.did).expect_err("mutation case must fail closed");
        let second = AgentDid::parse(case.did).expect_err("mutation case must fail closed");
        assert_eq!(first, second, "did reason drifted for case {}", case.id);
        let reason = first.to_string();
        assert!(
            !reason.trim().is_empty(),
            "did fail-closed reason must be explicit for case {}",
            case.id
        );
    }
}

#[test]
fn regression_did_mutation_reason_signatures_remain_stable() {
    // Regression: #843
    let error =
        AgentDid::parse("did:example:agent-1").expect_err("non-kamn DID method should fail closed");
    assert_eq!(
        error,
        AgentDidError::InvalidPrefix("did:example:agent-1".to_owned())
    );
    assert_eq!(
        error.to_string(),
        "invalid agent did prefix: did:example:agent-1"
    );
}

#[test]
fn performance_did_mutation_contract_lane_stays_within_budget() {
    let started = Instant::now();
    let mut accepted = 0_u64;
    let mut rejected = 0_u64;

    for seed in 0_u64..2048 {
        let did = mutate_did_case(seed);
        if AgentDid::parse(&did).is_ok() {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }

    assert!(
        accepted > 0,
        "mutation lane should retain valid DID samples"
    );
    assert!(
        rejected > 0,
        "mutation lane should reject invalid DID samples"
    );

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 300,
        "did mutation contract lane exceeded budget: {elapsed_millis}ms"
    );
}
