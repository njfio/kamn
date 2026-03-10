use crate::support::constants::{REASON_CODES_CSV, REASON_CODES_ORDER, REASON_TAXONOMY_VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContractDecision {
    pub(crate) status: &'static str,
    pub(crate) final_decision: &'static str,
    pub(crate) reason_taxonomy_version: &'static str,
    pub(crate) reason_codes_csv: &'static str,
    pub(crate) reason_codes_value: String,
    pub(crate) contract_status: &'static str,
}

pub(crate) fn build_decision(raw_reasons: Vec<&'static str>) -> ContractDecision {
    let reasons = normalize_reasons(raw_reasons);
    let status = if reasons.is_empty() { "pass" } else { "fail" };
    let final_decision = if status == "pass" { "GO" } else { "NO-GO" };
    let contract_status = if status == "pass" { "verified" } else { "violation" };
    ContractDecision {
        status,
        final_decision,
        reason_taxonomy_version: REASON_TAXONOMY_VERSION,
        reason_codes_csv: REASON_CODES_CSV,
        reason_codes_value: reason_codes_value(&reasons),
        contract_status,
    }
}

pub(crate) fn add_reason(reasons: &mut Vec<&'static str>, reason: &'static str) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn normalize_reasons(observed: Vec<&'static str>) -> Vec<&'static str> {
    REASON_CODES_ORDER
        .iter()
        .copied()
        .filter(|candidate| observed.contains(candidate))
        .collect()
}

fn reason_codes_value(reasons: &[&str]) -> String {
    if reasons.is_empty() {
        return "none".to_owned();
    }
    reasons.join(",")
}
