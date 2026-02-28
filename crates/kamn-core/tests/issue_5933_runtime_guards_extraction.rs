use kamn_core::{
    fairness_policy_reason_taxonomy_version as core_fairness_reason_taxonomy_version,
    quota_policy_reason_taxonomy_version as core_quota_reason_taxonomy_version,
    AntiSpamConfig as CoreAntiSpamConfig,
};
use kamn_runtime_guards::anti_spam::AntiSpamConfig as GuardsAntiSpamConfig;
use kamn_runtime_guards::fairness_policy::FAIRNESS_POLICY_REASON_TAXONOMY_VERSION as GUARDS_FAIRNESS_VERSION;
use kamn_runtime_guards::quota_policy::QUOTA_POLICY_REASON_TAXONOMY_VERSION as GUARDS_QUOTA_VERSION;

#[test]
fn spec_c01_phase1_runtime_guards_extraction_keeps_policy_parity() {
    assert_eq!(
        CoreAntiSpamConfig::default(),
        GuardsAntiSpamConfig::default()
    );
    assert_eq!(core_quota_reason_taxonomy_version(), GUARDS_QUOTA_VERSION);
    assert_eq!(
        core_fairness_reason_taxonomy_version(),
        GUARDS_FAIRNESS_VERSION
    );
}
