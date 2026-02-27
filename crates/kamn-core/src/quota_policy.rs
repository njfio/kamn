//! Compatibility re-exports for quota-policy contracts extracted from `kamn-core`.

pub use kamn_runtime_guards::quota_policy::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_c09_core_quota_policy_reexport_contracts() {
        let allow = QuotaPolicyInput {
            scope: "peer_sync".to_owned(),
            window_seconds: 10,
            limit: 4,
            observed_count: 4,
        };
        assert_eq!(evaluate_quota_policy(&allow), QuotaPolicyDecision::Allow);

        let reject = QuotaPolicyInput {
            scope: "peer_sync".to_owned(),
            window_seconds: 10,
            limit: 4,
            observed_count: 5,
        };
        assert_eq!(
            evaluate_quota_policy(&reject),
            QuotaPolicyDecision::Reject {
                reason: QuotaPolicyViolationReason::QuotaLimitExceeded,
            }
        );
        assert_eq!(
            QuotaPolicyViolationReason::QuotaLimitExceeded.as_str(),
            "quota_limit_exceeded"
        );
        assert_eq!(
            quota_policy_reason_taxonomy_version(),
            "kamn.runtime.quota-policy-reason-taxonomy.v1"
        );
    }
}
