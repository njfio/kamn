use kamn_core::{
    data_layer_pg_collect_migration_files, DataLayerPgExecutionAdapter,
    DataLayerPgExecutionAdapterConfig, DataLayerPgExecutionAdapterError,
    DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE,
};

use crate::support::{live_postgres_url, runtime};

#[test]
fn spec_c02_migration_file_inventory_is_deterministic() {
    let files =
        data_layer_pg_collect_migration_files().expect("migration inventory should be readable");
    assert_eq!(
        files,
        vec!["202602190001_data_layer_phase1_bootstrap.sql".to_owned()],
    );
}

#[test]
fn spec_c04_invalid_database_url_fails_closed() {
    let runtime = runtime();
    let error = runtime
        .block_on(DataLayerPgExecutionAdapter::connect(
            DataLayerPgExecutionAdapterConfig::new("not-a-valid-url"),
        ))
        .expect_err("invalid URL must fail closed");
    match error {
        DataLayerPgExecutionAdapterError::InvalidDatabaseUrl {
            field, reason_code, ..
        } => {
            assert_eq!(field, "database_url");
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn spec_c02_live_adapter_applies_default_rls_statements_deterministically() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = crate::support::connect_live_adapter(database_url).await;
        let first_report = adapter
            .apply_default_rls_statements()
            .await
            .expect("default RLS policies should apply");
        let second_report = adapter
            .apply_default_rls_statements()
            .await
            .expect("default RLS policies should be idempotent");

        assert!(!first_report.statement_outcomes.is_empty());
        assert_eq!(
            first_report.statement_outcomes.len(),
            second_report.statement_outcomes.len(),
        );
    });
}
