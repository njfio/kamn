use kamn_core::anti_spam::AntiSpamConfig as CoreAntiSpamConfig;
use kamn_core::fairness_policy::FAIRNESS_POLICY_REASON_TAXONOMY_VERSION as CORE_FAIRNESS_VERSION;
use kamn_core::quota_policy::QUOTA_POLICY_REASON_TAXONOMY_VERSION as CORE_QUOTA_VERSION;
use kamn_runtime_guards::anti_spam::AntiSpamConfig as GuardsAntiSpamConfig;
use kamn_runtime_guards::fairness_policy::FAIRNESS_POLICY_REASON_TAXONOMY_VERSION as GUARDS_FAIRNESS_VERSION;
use kamn_runtime_guards::quota_policy::QUOTA_POLICY_REASON_TAXONOMY_VERSION as GUARDS_QUOTA_VERSION;

#[test]
fn spec_c01_phase1_runtime_guards_extraction_keeps_policy_parity() {
    assert_eq!(
        CoreAntiSpamConfig::default(),
        GuardsAntiSpamConfig::default()
    );
    assert_eq!(CORE_QUOTA_VERSION, GUARDS_QUOTA_VERSION);
    assert_eq!(CORE_FAIRNESS_VERSION, GUARDS_FAIRNESS_VERSION);
}
