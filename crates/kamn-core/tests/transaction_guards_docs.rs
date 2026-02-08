const DOC: &str = include_str!("../../../docs/foundation/transaction-guards.md");

#[test]
fn doc_contains_transaction_guard_scope_and_components() {
    assert!(DOC.contains("## Invariants Enforced"));
    assert!(DOC.contains("BaselineTransaction"));
    assert!(DOC.contains("TransactionGuards"));
    assert!(DOC.contains("TransactionGuardError"));
}

#[test]
fn doc_contains_canonical_signature_profile_contract() {
    assert!(DOC.contains("## Canonical Signature Profile"));
    assert!(DOC.contains("baseline_signature_for_fields(...)"));
    assert!(DOC.contains("signature_profile_compatibility_fixtures_for_fields(...)"));
    assert!(DOC.contains("legacy-unversioned"));
    assert!(DOC.contains("baseline-v0"));
    assert!(DOC.contains("secp256k1+baseline-v1"));
    assert!(DOC.contains("shared between `transaction` and `signer_backend` paths"));
    assert!(DOC.contains("baseline signature algorithm: `ed25519`."));
    assert!(DOC.contains("baseline signature profile id: `baseline-v1`"));
}

#[test]
fn doc_contains_signer_fallback_policy_integration_rules() {
    assert!(DOC.contains("## Signer Fallback Policy Integration"));
    assert!(DOC.contains("secure:aws-kms:role-<operator|admin|treasury|auditor>/<key-ref>"));
    assert!(DOC.contains("KeyRoleMismatch"));
    assert!(DOC.contains("FallbackDeniedByRolePolicy"));
}

#[test]
fn regression_requires_signature_profile_drift_guard_rule() {
    // Regression: #400
    assert!(DOC.contains(
        "signature-profile drift between transaction and signer paths is rejected (`Regression: #400`).",
    ));
    assert!(DOC.contains("non-versioned signature profile is rejected (`Regression: #404`)."));
    assert!(DOC.contains(
        "algorithm/profile metadata drift or downgrade is rejected (`Regression: #677`)."
    ));
    assert!(DOC.contains(
        "compatibility fixture matrix remains aligned between signer and transaction verification (`Regression: #677`)."
    ));
    assert!(
        DOC.contains(
            "privileged roles (`admin`, `treasury`, `auditor`) reject fallback via `FallbackDeniedByRolePolicy` (`Regression: #619`).",
        ),
    );
}
