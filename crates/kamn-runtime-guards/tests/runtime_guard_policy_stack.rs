use kamn_runtime_guards::anti_spam::{
    AntiSpamConfig, AntiSpamEngine, AntiSpamError, AntiSpamRejection,
};
use kamn_runtime_guards::fairness_policy::FairnessPolicyInput;
use kamn_runtime_guards::policy_stack::{
    evaluate_runtime_guard_policy_stack, RuntimeGuardPolicyDecision, RuntimeGuardPolicyRejection,
};
use kamn_runtime_guards::quota_policy::QuotaPolicyInput;

fn anti_spam_engine() -> AntiSpamEngine {
    let mut engine =
        AntiSpamEngine::new(AntiSpamConfig::default()).expect("valid anti-spam config");
    engine
        .set_deposit("kamn:did:agent:alice", 100)
        .expect("valid sender deposit");
    engine
}

fn valid_quota() -> QuotaPolicyInput {
    QuotaPolicyInput {
        scope: "processor_ingress".to_owned(),
        window_seconds: 60,
        limit: 5,
        observed_count: 1,
    }
}

fn valid_fairness() -> FairnessPolicyInput {
    FairnessPolicyInput {
        scope: "control_plane".to_owned(),
        window_seconds: 60,
        active_weighted_share: 1,
        max_weighted_share_gap: 2,
    }
}

#[test]
fn integration_policy_stack_allows_when_all_policies_allow() {
    let mut anti_spam = anti_spam_engine();
    let decision = evaluate_runtime_guard_policy_stack(
        &mut anti_spam,
        "kamn:did:agent:alice",
        "msg-allow",
        1_700_000_000,
        &valid_quota(),
        &valid_fairness(),
    )
    .expect("evaluation should succeed");

    assert_eq!(decision, RuntimeGuardPolicyDecision::Allow);
}

#[test]
fn integration_policy_stack_rejects_anti_spam_before_quota_and_fairness() {
    let mut anti_spam = anti_spam_engine();
    anti_spam
        .set_deposit("kamn:did:agent:alice", 0)
        .expect("valid sender update");

    let quota = QuotaPolicyInput {
        scope: "unknown".to_owned(),
        ..valid_quota()
    };
    let fairness = FairnessPolicyInput {
        scope: "unknown".to_owned(),
        ..valid_fairness()
    };

    let decision = evaluate_runtime_guard_policy_stack(
        &mut anti_spam,
        "kamn:did:agent:alice",
        "msg-anti-spam-first",
        1_700_000_001,
        &quota,
        &fairness,
    )
    .expect("evaluation should succeed");

    assert_eq!(
        decision,
        RuntimeGuardPolicyDecision::Reject {
            reason: RuntimeGuardPolicyRejection::AntiSpam(AntiSpamRejection::InsufficientDeposit {
                required: 10,
                provided: 0,
            },),
        }
    );
}

#[test]
fn integration_policy_stack_rejects_quota_after_anti_spam_allow() {
    let mut anti_spam = anti_spam_engine();
    let quota = QuotaPolicyInput {
        observed_count: 99,
        ..valid_quota()
    };

    let decision = evaluate_runtime_guard_policy_stack(
        &mut anti_spam,
        "kamn:did:agent:alice",
        "msg-quota-reject",
        1_700_000_002,
        &quota,
        &valid_fairness(),
    )
    .expect("evaluation should succeed");

    assert!(matches!(
        decision,
        RuntimeGuardPolicyDecision::Reject {
            reason: RuntimeGuardPolicyRejection::Quota(_),
        }
    ));
}

#[test]
fn integration_policy_stack_rejects_fairness_after_anti_spam_and_quota_allow() {
    let mut anti_spam = anti_spam_engine();
    let fairness = FairnessPolicyInput {
        active_weighted_share: 999,
        ..valid_fairness()
    };

    let decision = evaluate_runtime_guard_policy_stack(
        &mut anti_spam,
        "kamn:did:agent:alice",
        "msg-fairness-reject",
        1_700_000_003,
        &valid_quota(),
        &fairness,
    )
    .expect("evaluation should succeed");

    assert!(matches!(
        decision,
        RuntimeGuardPolicyDecision::Reject {
            reason: RuntimeGuardPolicyRejection::Fairness(_),
        }
    ));
}

#[test]
fn integration_policy_stack_propagates_invalid_input_error_from_anti_spam_engine() {
    let mut anti_spam = anti_spam_engine();

    let error = evaluate_runtime_guard_policy_stack(
        &mut anti_spam,
        "did:example:alice",
        "msg-invalid-input",
        1_700_000_004,
        &valid_quota(),
        &valid_fairness(),
    )
    .expect_err("invalid anti-spam input must fail closed");

    assert_eq!(
        error,
        AntiSpamError::InvalidInput("sender_did must use kamn:did:agent:* format".to_owned())
    );
}
