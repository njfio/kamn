use super::report::DemoReportInput;

pub(crate) use super::three_agent_receipt_verify::validate_three_agent_receipt_files;
pub(crate) use super::three_agent_receipt_write::write_three_agent_receipts;

pub(super) const AGENT_A_RECEIPT_FILE: &str = "agent-a-observation-receipt.json";
pub(super) const AGENT_B_RECEIPT_FILE: &str = "agent-b-observation-receipt.json";
pub(super) const AGENT_C_RECEIPT_FILE: &str = "agent-c-verifier-observation-receipt.json";

pub(crate) fn agent_a_observation_receipt_path(input: &DemoReportInput<'_>) -> String {
    receipt_path(input, AGENT_A_RECEIPT_FILE)
}

pub(crate) fn agent_b_observation_receipt_path(input: &DemoReportInput<'_>) -> String {
    receipt_path(input, AGENT_B_RECEIPT_FILE)
}

pub(crate) fn agent_c_verifier_observation_receipt_path(input: &DemoReportInput<'_>) -> String {
    receipt_path(input, AGENT_C_RECEIPT_FILE)
}

fn receipt_path(input: &DemoReportInput<'_>, file_name: &str) -> String {
    input
        .output_root
        .join(format!("{}/proof/{file_name}", input.run_id))
        .display()
        .to_string()
}
