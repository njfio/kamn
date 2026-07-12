use serde_json::Value;

const REQUIRED: [[&str; 4]; 3] = [
    ["register", "create_task", "fund_escrow", "release_escrow"],
    ["register", "accept_task", "complete_task", ""],
    ["register", "", "", ""],
];

pub(super) fn precheck_required_operations(paths: &[String; 3]) -> Result<(), String> {
    for (index, path) in paths.iter().enumerate() {
        let Some(receipts) = read_receipts(path)? else {
            continue;
        };
        for action in REQUIRED[index].iter().filter(|action| !action.is_empty()) {
            classify_action(receipts.as_slice(), action)?;
        }
    }
    Ok(())
}

fn read_receipts(path: &str) -> Result<Option<Vec<Value>>, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read Pi transaction actor {path}: {error}"))?;
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return Ok(None);
    };
    Ok(value["runtime_response_receipts"].as_array().cloned())
}

fn classify_action(receipts: &[Value], action: &str) -> Result<(), String> {
    let matching = receipts
        .iter()
        .filter(|receipt| receipt["tool"] == action)
        .collect::<Vec<_>>();
    let success_count = matching
        .iter()
        .filter(|receipt| receipt["outcome"] == "success")
        .count();
    if success_count > 1 {
        return Err("RUNTIME_RECEIPT_CHAIN_STEP_DUPLICATED".to_owned());
    }
    if success_count == 1 {
        return Ok(());
    }
    if matching.iter().any(|receipt| receipt["outcome"] == "error") {
        return Err("RUNTIME_RECEIPT_CHAIN_OUTCOME_INVALID".to_owned());
    }
    Err("RUNTIME_RECEIPT_CHAIN_STEP_MISSING".to_owned())
}
