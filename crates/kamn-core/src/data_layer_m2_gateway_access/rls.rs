use super::DATA_LAYER_M2_REQUESTER_DID_SETTING;

/// RLS policy template projection for one table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2RlsPolicy {
    pub table_name: String,
    pub policy_name: String,
    pub using_clause: String,
    pub with_check_clause: Option<String>,
}

/// Returns default M2 RLS policy templates for gateway-scoped tables.
pub fn data_layer_m2_default_rls_policies() -> Vec<DataLayerM2RlsPolicy> {
    let requester = format!("current_setting('{DATA_LAYER_M2_REQUESTER_DID_SETTING}', true)");
    let requester_guard = format!("{requester} <> ''");
    vec![
        messages_policy(&requester, &requester_guard),
        access_log_policy(&requester, &requester_guard),
    ]
}

fn messages_policy(requester: &str, requester_guard: &str) -> DataLayerM2RlsPolicy {
    DataLayerM2RlsPolicy {
        table_name: "messages".to_owned(),
        policy_name: "m2_messages_requester_scope".to_owned(),
        using_clause: format!(
            "{requester_guard} AND (sender_did = {requester} OR recipient_did = {requester} OR owner_sender_did = {requester} OR owner_recipient_did = {requester})"
        ),
        with_check_clause: None,
    }
}

fn access_log_policy(requester: &str, requester_guard: &str) -> DataLayerM2RlsPolicy {
    DataLayerM2RlsPolicy {
        table_name: "access_log".to_owned(),
        policy_name: "m2_access_log_requester_scope".to_owned(),
        using_clause: format!("{requester_guard} AND requester_did = {requester}"),
        with_check_clause: None,
    }
}
