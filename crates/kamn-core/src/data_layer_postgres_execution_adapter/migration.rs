use std::fs;
use std::path::PathBuf;

use super::*;

/// Collects migration SQL file names from the canonical migrations directory.
pub fn data_layer_pg_collect_migration_files(
) -> Result<Vec<String>, DataLayerPgExecutionAdapterError> {
    let migrations_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DATA_LAYER_PG_MIGRATIONS_DIR);
    let directory_iter = fs::read_dir(migrations_dir.as_path()).map_err(|error| {
        DataLayerPgExecutionAdapterError::MigrationIoFailed {
            reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
            detail: format!(
                "failed reading migrations directory {}: {error}",
                migrations_dir.display()
            ),
        }
    })?;

    let mut files = Vec::new();
    for entry in directory_iter {
        let entry = entry.map_err(
            |error| DataLayerPgExecutionAdapterError::MigrationIoFailed {
                reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                detail: format!("failed reading migration directory entry: {error}"),
            },
        )?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|extension| extension == "sql") {
            let file_name = path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .ok_or(DataLayerPgExecutionAdapterError::MigrationIoFailed {
                    reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                    detail: format!("invalid migration file path: {}", path.display()),
                })?;
            files.push(file_name);
        }
    }
    files.sort();
    Ok(files)
}

pub(super) fn data_layer_pg_split_migration_statements(source: &str) -> Vec<String> {
    let mut sanitized = String::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("--") {
            continue;
        }
        sanitized.push_str(line);
        sanitized.push('\n');
    }

    sanitized
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .filter(|statement| !statement.eq_ignore_ascii_case("BEGIN"))
        .filter(|statement| !statement.eq_ignore_ascii_case("COMMIT"))
        .map(|statement| format!("{statement};"))
        .collect()
}
