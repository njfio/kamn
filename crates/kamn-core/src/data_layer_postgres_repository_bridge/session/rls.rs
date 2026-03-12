use crate::data_layer_m2_default_rls_policies;

use super::super::DataLayerPgRlsStatement;

pub fn data_layer_pg_project_default_rls_statements() -> Vec<DataLayerPgRlsStatement> {
    let mut policies = data_layer_m2_default_rls_policies();
    policies.sort_by(|left, right| {
        left.table_name
            .cmp(&right.table_name)
            .then(left.policy_name.cmp(&right.policy_name))
    });

    let mut statements = Vec::with_capacity(policies.len() * 3);
    for policy in policies {
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name.clone(),
            policy_name: policy.policy_name.clone(),
            sql: format!(
                "ALTER TABLE {} ENABLE ROW LEVEL SECURITY;",
                policy.table_name
            ),
        });
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name.clone(),
            policy_name: policy.policy_name.clone(),
            sql: format!(
                "DROP POLICY IF EXISTS {} ON {};",
                policy.policy_name, policy.table_name
            ),
        });
        let mut create_sql = format!(
            "CREATE POLICY {} ON {} USING ({}",
            policy.policy_name, policy.table_name, policy.using_clause
        );
        create_sql.push(')');
        if let Some(with_check_clause) = policy.with_check_clause {
            create_sql.push_str(format!(" WITH CHECK ({with_check_clause})").as_str());
        }
        create_sql.push(';');
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name,
            policy_name: policy.policy_name,
            sql: create_sql,
        });
    }
    statements
}
