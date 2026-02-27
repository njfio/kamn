//! Compatibility re-exports for fairness-policy contracts extracted from `kamn-core`.

pub use kamn_runtime_guards::fairness_policy::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_c10_core_fairness_policy_reexport_contracts() {
        let allow = FairnessPolicyInput {
            scope: "tenant_interactive".to_owned(),
            window_seconds: 30,
            active_weighted_share: 7,
            max_weighted_share_gap: 7,
        };
        assert_eq!(
            evaluate_fairness_policy(&allow),
            FairnessPolicyDecision::Allow
        );

        let reject = FairnessPolicyInput {
            scope: "tenant_interactive".to_owned(),
            window_seconds: 30,
            active_weighted_share: 8,
            max_weighted_share_gap: 7,
        };
        assert_eq!(
            evaluate_fairness_policy(&reject),
            FairnessPolicyDecision::Reject {
                reason: FairnessPolicyViolationReason::WeightedShareExceedsGap,
            }
        );
        assert_eq!(
            FairnessPolicyViolationReason::WeightedShareExceedsGap.as_str(),
            "fairness_weighted_share_exceeds_gap"
        );
        assert_eq!(
            fairness_policy_reason_taxonomy_version(),
            "kamn.runtime.fairness-policy-reason-taxonomy.v1"
        );
    }
}
