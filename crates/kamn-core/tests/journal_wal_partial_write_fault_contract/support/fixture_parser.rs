const FIXTURE: &str =
    include_str!("../../../../../fixtures/runtime/journal_wal_partial_write_fault_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FixtureMetadata {
    pub(crate) schema_version: String,
    pub(crate) reason_taxonomy_version: String,
    pub(crate) reason_codes_csv: String,
    pub(crate) columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FixtureCase {
    pub(crate) case_id: String,
    pub(crate) store: String,
    pub(crate) fault_mode: String,
    pub(crate) expected_outcome: String,
    pub(crate) expected_marker: String,
}

pub(crate) fn parse_fixture() -> Result<(FixtureMetadata, Vec<FixtureCase>), String> {
    let mut metadata = MetadataFields::default();
    let mut cases = Vec::new();
    for raw_line in FIXTURE.lines() {
        parse_fixture_line(raw_line, &mut metadata, &mut cases)?;
    }
    let metadata = metadata.build()?;
    validate_cases(&cases)?;
    Ok((metadata, cases))
}

pub(crate) fn parse_case_line(line: &str) -> Result<FixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 5 {
        return Err(format!("expected 5 columns, found {} in '{line}'", parts.len()));
    }
    Ok(FixtureCase {
        case_id: parts[0].to_owned(),
        store: parts[1].to_owned(),
        fault_mode: parts[2].to_owned(),
        expected_outcome: parts[3].to_owned(),
        expected_marker: parts[4].to_owned(),
    })
}

#[derive(Default)]
struct MetadataFields {
    schema_version: Option<String>,
    reason_taxonomy_version: Option<String>,
    reason_codes_csv: Option<String>,
    columns: Option<String>,
}

impl MetadataFields {
    fn build(self) -> Result<FixtureMetadata, String> {
        Ok(FixtureMetadata {
            schema_version: self
                .schema_version
                .ok_or("missing schema version metadata".to_owned())?,
            reason_taxonomy_version: self
                .reason_taxonomy_version
                .ok_or("missing reason taxonomy metadata".to_owned())?,
            reason_codes_csv: self
                .reason_codes_csv
                .ok_or("missing reason codes csv metadata".to_owned())?,
            columns: self.columns.ok_or("missing columns metadata".to_owned())?,
        })
    }
}

fn parse_fixture_line(
    raw_line: &str,
    metadata: &mut MetadataFields,
    cases: &mut Vec<FixtureCase>,
) -> Result<(), String> {
    let line = raw_line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(());
    }
    if line.contains('=') {
        return assign_metadata(line, metadata);
    }
    cases.push(parse_case_line(line)?);
    Ok(())
}

fn assign_metadata(line: &str, metadata: &mut MetadataFields) -> Result<(), String> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| format!("invalid metadata line: {line}"))?;
    let value = value.trim().to_owned();
    match key.trim() {
        "journal_wal_partial_write_fixture_schema_version" => metadata.schema_version = Some(value),
        "journal_wal_partial_write_reason_taxonomy_version" => {
            metadata.reason_taxonomy_version = Some(value)
        }
        "journal_wal_partial_write_reason_codes_csv" => metadata.reason_codes_csv = Some(value),
        "columns" => metadata.columns = Some(value),
        unknown => return Err(format!("unknown metadata key: {unknown}")),
    }
    Ok(())
}

fn validate_cases(cases: &[FixtureCase]) -> Result<(), String> {
    if cases.is_empty() {
        return Err("fixture matrix must contain at least one case".to_owned());
    }
    Ok(())
}
