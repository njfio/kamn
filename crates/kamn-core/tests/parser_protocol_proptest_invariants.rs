use kamn_core::{
    baseline_signature_for_fields, parse_signature_profile_metadata,
    signature_matches_supported_profile_for_fields, KolmeApiNextNonceResponse,
    KolmeRuntimeCommitError, KolmeRuntimeCommitRequest, SignatureProfileMetadata,
    BASELINE_SIGNATURE_ALGORITHM, BASELINE_SIGNATURE_PROFILE_ID,
};
#[path = "property_invariant_helpers.rs"]
mod property_invariant_helpers;

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{RngAlgorithm, RngSeed};

const CASES: u32 = 128;
const MAX_TOKEN_LEN: usize = 32;
const AUTH_REPLAY_SEED: u64 = 0x5938_0000_0000_0001;
const PROTOCOL_NONCE_SEED: u64 = 0x5938_0000_0000_0002;
const MESSAGE_PROTOCOL_SEED: u64 = 0x5938_0000_0000_0003;
const PARSER_PROTOCOL_SEED_ENV_KEY: &str = "KAMN_PROPTEST_PARSER_PROTOCOL_SEED";
const PARSER_PROTOCOL_SEED_SALT: u64 = 0x0038_5938;
const PROPTEST_SOURCE_PATH: &str = file!();

const TOKEN_BYTES: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789-_";

fn base_seed() -> u64 {
    property_invariant_helpers::resolve_seed_from_env(
        PARSER_PROTOCOL_SEED_ENV_KEY,
        AUTH_REPLAY_SEED,
    )
}

fn deterministic_config(cases: u32, seed: u64) -> proptest::test_runner::Config {
    property_invariant_helpers::deterministic_proptest_config(cases, seed, PROPTEST_SOURCE_PATH)
}

fn token_strategy(max_len: usize) -> impl Strategy<Value = String> {
    vec(0usize..TOKEN_BYTES.len(), 1..(max_len + 1)).prop_map(|indexes| {
        indexes
            .into_iter()
            .map(|index| TOKEN_BYTES[index] as char)
            .collect()
    })
}

#[test]
fn unit_parser_protocol_proptest_config_is_deterministic_and_persistent() {
    let seed = base_seed();
    let config = deterministic_config(CASES, seed);
    assert_eq!(config.cases, CASES);
    assert_eq!(config.rng_algorithm, RngAlgorithm::ChaCha);
    assert_eq!(config.rng_seed, RngSeed::Fixed(seed));
    assert_eq!(config.source_file, Some(PROPTEST_SOURCE_PATH));
    assert!(config.failure_persistence.is_some());
}

#[test]
fn regression_parser_protocol_proptest_seed_corpus_is_tracked() {
    let corpus =
        include_str!("../proptest-regressions/tests/parser_protocol_proptest_invariants.txt");
    assert!(corpus.contains("Seeds for failure cases"));
}

#[test]
fn unit_parser_protocol_proptest_budget_envelope_is_bounded() {
    let cases = std::hint::black_box(CASES);
    let max_token_len = std::hint::black_box(MAX_TOKEN_LEN);
    assert!(
        cases <= 192,
        "parser/protocol property case budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        max_token_len <= 48,
        "parser/protocol token strategy budget must stay bounded for deterministic CI runtime"
    );
}

proptest! {
    #![proptest_config(deterministic_config(CASES, base_seed()))]

    #[test]
    fn functional_auth_replay_signature_profile_proptest_invariants_hold(
        sender in token_strategy(MAX_TOKEN_LEN),
        state_token in token_strategy(MAX_TOKEN_LEN),
        payload in token_strategy(64),
        nonce in 1_u64..1_000_000_u64
    ) {
        let state_hash = format!("state:{state_token}");
        let signature = baseline_signature_for_fields(
            sender.as_str(),
            nonce,
            state_hash.as_str(),
            payload.as_str(),
        );
        let metadata = parse_signature_profile_metadata(signature.as_str());

        prop_assert_eq!(
            metadata,
            Some(SignatureProfileMetadata {
                algorithm: BASELINE_SIGNATURE_ALGORITHM.to_owned(),
                profile_id: BASELINE_SIGNATURE_PROFILE_ID.to_owned(),
            })
        );
        prop_assert!(signature_matches_supported_profile_for_fields(
            signature.as_str(),
            sender.as_str(),
            nonce,
            state_hash.as_str(),
            payload.as_str(),
        ));
        prop_assert!(!signature_matches_supported_profile_for_fields(
            signature.as_str(),
            sender.as_str(),
            nonce + 1,
            state_hash.as_str(),
            payload.as_str(),
        ));
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        CASES,
        property_invariant_helpers::derive_seed(base_seed(), PARSER_PROTOCOL_SEED_SALT ^ PROTOCOL_NONCE_SEED),
    ))]

    #[test]
    fn functional_protocol_nonce_response_proptest_preserves_positive_nonce_and_optional_account(
        next_nonce in 1_u64..1_000_000_u64,
        quote_numeric_nonce in any::<bool>(),
        account_id in proptest::option::of(token_strategy(24)),
    ) {
        let nonce_field = if quote_numeric_nonce {
            format!("\"{next_nonce}\"")
        } else {
            next_nonce.to_string()
        };
        let account_field = match account_id.as_ref() {
            Some(value) => format!("\"{value}\""),
            None => "null".to_owned(),
        };

        let response = format!(
            "{{\"next_nonce\":{nonce_field},\"account_id\":{account_field}}}"
        );

        match KolmeApiNextNonceResponse::parse_json(response.as_str()) {
            Ok(parsed) => {
                prop_assert_eq!(parsed.next_nonce, next_nonce);
                prop_assert_eq!(parsed.account_id, account_id);
            }
            Err(error) => {
                prop_assert!(false, "expected valid nonce response parse, got {error:?}");
            }
        }
    }
}

proptest! {
    #![proptest_config(deterministic_config(
        CASES,
        property_invariant_helpers::derive_seed(base_seed(), PARSER_PROTOCOL_SEED_SALT ^ MESSAGE_PROTOCOL_SEED),
    ))]

    #[test]
    fn integration_message_protocol_signed_envelope_translation_proptest_is_stable(
        operation_id in token_strategy(16),
        state_token in token_strategy(16),
        actor_token in token_strategy(16),
        payload_token in token_strategy(24),
        signer_token in token_strategy(16),
        signature_token in token_strategy(32),
        nonce in 1_u64..10_000_u64,
        recovery_id in 0_u8..=3_u8,
    ) {
        let request = match KolmeRuntimeCommitRequest::deterministic(
            format!("op-{operation_id}").as_str(),
            format!("state:{state_token}").as_str(),
            format!("kamn:did:agent:{actor_token}").as_str(),
            nonce,
            format!("payload:{payload_token}").as_str(),
        ) {
            Ok(request) => request,
            Err(error) => {
                prop_assert!(false, "request construction should remain valid, got {error:?}");
                return Ok(());
            }
        };

        let canonical_message = request.to_wire_payload();
        let signer_key_id = format!("kamn:key:signer:{signer_token}");
        let signature = format!("sig:{signature_token}");

        let envelope = match request.translate_to_signed_broadcast_envelope(
            signer_key_id.as_str(),
            canonical_message.as_str(),
            signature.as_str(),
            recovery_id,
        ) {
            Ok(envelope) => envelope,
            Err(error) => {
                prop_assert!(false, "expected canonical translation to pass, got {error:?}");
                return Ok(());
            }
        };

        let broadcast_request = match envelope.to_broadcast_request() {
            Ok(request) => request,
            Err(error) => {
                prop_assert!(false, "expected broadcast request translation to pass, got {error:?}");
                return Ok(());
            }
        };

        prop_assert_eq!(broadcast_request.signature.as_str(), signature.as_str());
        prop_assert_eq!(broadcast_request.recovery_id, recovery_id);

        let tampered_message = format!("{canonical_message}tamper");
        prop_assert_eq!(
            request.translate_to_signed_broadcast_envelope(
                signer_key_id.as_str(),
                tampered_message.as_str(),
                signature.as_str(),
                recovery_id,
            ),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signed_message",
                reason: "must match canonical runtime commit wire payload",
            })
        );
    }
}
