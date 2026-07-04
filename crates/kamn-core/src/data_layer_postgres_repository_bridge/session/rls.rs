use crate::{data_layer_m2_default_rls_policies, DataLayerM2RlsPolicy};

use super::super::DataLayerPgRlsStatement;

/// Runs the data layer pg project default rls statements contract helper.
pub fn data_layer_pg_project_default_rls_statements() -> Vec<DataLayerPgRlsStatement> {
    sorted_policies()
        .into_iter()
        .flat_map(policy_to_statements)
        .collect()
}

fn sorted_policies() -> Vec<DataLayerM2RlsPolicy> {
    let mut policies = data_layer_m2_default_rls_policies();
    policies.sort_by(|left, right| {
        left.table_name
            .cmp(&right.table_name)
            .then(left.policy_name.cmp(&right.policy_name))
    });
    policies
}

fn policy_to_statements(policy: DataLayerM2RlsPolicy) -> Vec<DataLayerPgRlsStatement> {
    vec![
        enable_rls_statement(&policy),
        drop_policy_statement(&policy),
        create_policy_statement(policy),
    ]
}

fn enable_rls_statement(policy: &DataLayerM2RlsPolicy) -> DataLayerPgRlsStatement {
    DataLayerPgRlsStatement {
        table_name: policy.table_name.clone(),
        policy_name: policy.policy_name.clone(),
        sql: format!(
            "ALTER TABLE {} ENABLE ROW LEVEL SECURITY;",
            policy.table_name
        ),
    }
}

fn drop_policy_statement(policy: &DataLayerM2RlsPolicy) -> DataLayerPgRlsStatement {
    DataLayerPgRlsStatement {
        table_name: policy.table_name.clone(),
        policy_name: policy.policy_name.clone(),
        sql: format!(
            "DROP POLICY IF EXISTS {} ON {};",
            policy.policy_name, policy.table_name
        ),
    }
}

fn create_policy_statement(policy: DataLayerM2RlsPolicy) -> DataLayerPgRlsStatement {
    DataLayerPgRlsStatement {
        table_name: policy.table_name.clone(),
        policy_name: policy.policy_name.clone(),
        sql: create_policy_sql(&policy),
    }
}

fn create_policy_sql(policy: &DataLayerM2RlsPolicy) -> String {
    let mut create_sql = format!(
        "CREATE POLICY {} ON {} USING ({})",
        policy.policy_name, policy.table_name, policy.using_clause
    );
    if let Some(with_check_clause) = policy.with_check_clause.as_ref() {
        create_sql.push_str(format!(" WITH CHECK ({with_check_clause})").as_str());
    }
    create_sql.push(';');
    create_sql
}
