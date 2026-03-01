use crate::anti_spam::{
    AntiSpamDecision, AntiSpamEngine, AntiSpamError, AntiSpamRejection,
};
use crate::fairness_policy::{
    evaluate_fairness_policy, FairnessPolicyDecision, FairnessPolicyInput,
    FairnessPolicyViolationReason,
};
use crate::quota_policy::{
    evaluate_quota_policy, QuotaPolicyDecision, QuotaPolicyInput, QuotaPolicyViolationReason,
};

/// Rejection reason emitted by runtime guard policy-stack evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeGuardPolicyRejection {
    /// Anti-spam rejected the message.
    AntiSpam(AntiSpamRejection),
    /// Quota policy rejected the request.
    Quota(QuotaPolicyViolationReason),
    /// Fairness policy rejected the request.
    Fairness(FairnessPolicyViolationReason),
}

/// Deterministic decision emitted by runtime guard policy-stack evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeGuardPolicyDecision {
    /// All policies allowed the request.
    Allow,
    /// One policy rejected the request.
    Reject {
        /// Typed policy rejection.
        reason: RuntimeGuardPolicyRejection,
    },
}

/// Evaluates anti-spam, quota, and fairness in deterministic order.
///
/// Order is fixed and fail-closed:
/// 1. anti-spam
/// 2. quota
/// 3. fairness
pub fn evaluate_runtime_guard_policy_stack(
    anti_spam: &mut AntiSpamEngine,
    sender_did: &str,
    message_id: &str,
    now_unix: u64,
    quota_input: &QuotaPolicyInput,
    fairness_input: &FairnessPolicyInput,
) -> Result<RuntimeGuardPolicyDecision, AntiSpamError> {
    match anti_spam.evaluate(sender_did, message_id, now_unix)? {
        AntiSpamDecision::Accepted => {}
        AntiSpamDecision::Rejected(reason) => {
            return Ok(RuntimeGuardPolicyDecision::Reject {
                reason: RuntimeGuardPolicyRejection::AntiSpam(reason),
            });
        }
    }

    match evaluate_quota_policy(quota_input) {
        QuotaPolicyDecision::Allow => {}
        QuotaPolicyDecision::Reject { reason } => {
            return Ok(RuntimeGuardPolicyDecision::Reject {
                reason: RuntimeGuardPolicyRejection::Quota(reason),
            });
        }
    }

    match evaluate_fairness_policy(fairness_input) {
        FairnessPolicyDecision::Allow => {}
        FairnessPolicyDecision::Reject { reason } => {
            return Ok(RuntimeGuardPolicyDecision::Reject {
                reason: RuntimeGuardPolicyRejection::Fairness(reason),
            });
        }
    }

    Ok(RuntimeGuardPolicyDecision::Allow)
}
