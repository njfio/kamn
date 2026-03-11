use kamn_core::{AuditDomain, AuditEventRecord, AuditExportEngine, AuditExportFilter, AuditExportFormat, AuditExportRequest};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const SERVICE_API_AUDIT_EXPORT_FILE_ENV: &str = "KAMN_SERVICE_API_AUDIT_EXPORT_FILE";
const SERVICE_API_AUDIT_EXPORTER_DID: &str = "kamn:did:agent:service-api-audit-exporter";
const SERVICE_API_RUNTIME_ACTOR_DID: &str = "kamn:did:agent:service-api-runtime";

pub(super) fn resolve_service_api_audit_export_file(
    state_file: Option<&str>,
) -> Result<Option<String>, String> {
    match env::var(SERVICE_API_AUDIT_EXPORT_FILE_ENV) {
        Ok(path) if path.trim().is_empty() => Err(format!(
            "service api audit export file env is empty: {SERVICE_API_AUDIT_EXPORT_FILE_ENV}"
        )),
        Ok(path) => Ok(Some(path)),
        Err(env::VarError::NotPresent) => {
            Ok(state_file.map(default_service_api_audit_export_file_path_from_state_file))
        }
        Err(error) => Err(format!("service api audit export file env read failed: {error}")),
    }
}

pub(super) fn service_api_task_created_audit_event(task_id: &str) -> AuditEventRecord {
    build_event(AuditDomain::Tasks, task_id, SERVICE_API_RUNTIME_ACTOR_DID, "service_api_task_created")
}

pub(super) fn service_api_message_created_audit_event(
    message_id: &str,
    sender_did: Option<&str>,
) -> AuditEventRecord {
    build_event(
        AuditDomain::Messages,
        message_id,
        sender_did.unwrap_or(SERVICE_API_RUNTIME_ACTOR_DID),
        "service_api_message_created",
    )
}

pub(super) fn service_api_message_relayed_audit_event(
    message_id: &str,
    sender_did: Option<&str>,
) -> AuditEventRecord {
    build_event(
        AuditDomain::Messages,
        message_id,
        sender_did.unwrap_or(SERVICE_API_RUNTIME_ACTOR_DID),
        "service_api_message_relayed",
    )
}

pub(super) fn persist_service_api_audit_export_event(
    audit_export_file: Option<&str>,
    event: AuditEventRecord,
) -> Result<(), String> {
    let Some(path) = audit_export_file else {
        return Ok(());
    };
    let mut engine = AuditExportEngine::new(vec![SERVICE_API_AUDIT_EXPORTER_DID.to_owned()])
        .map_err(|error| format!("service api audit export engine init failed: {error}"))?;
    for record in load_existing_records(path)? {
        engine
            .ingest_event(record)
            .map_err(|error| format!("service api audit export load failed: {error}"))?;
    }
    engine
        .ingest_event(event)
        .map_err(|error| format!("service api audit export ingest failed: {error}"))?;
    let timestamp = current_unix_timestamp_string()?;
    let bundle = engine
        .export(&AuditExportRequest {
            request_id: "service-api-runtime-export".to_owned(),
            requested_by: SERVICE_API_AUDIT_EXPORTER_DID.to_owned(),
            requested_at: timestamp,
            format: AuditExportFormat::Json,
            filter: AuditExportFilter::default(),
        })
        .map_err(|error| format!("service api audit export render failed: {error}"))?;
    fs::write(path, render_bundle_json(&bundle))
        .map_err(|error| format!("service api audit export write failed: {path}: {error}"))?;
    Ok(())
}

fn build_event(domain: AuditDomain, event_id: &str, actor: &str, action: &str) -> AuditEventRecord {
    AuditEventRecord {
        domain,
        event_id: event_id.to_owned(),
        occurred_at: current_unix_timestamp_string().unwrap_or_else(|_| "0".to_owned()),
        actor: actor.to_owned(),
        action: action.to_owned(),
        payload_digest: event_id.to_owned(),
    }
}

fn default_service_api_audit_export_file_path_from_state_file(state_file: &str) -> String {
    format!("{state_file}.audit-export.json")
}

fn current_unix_timestamp_string() -> Result<String, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("service api audit export timestamp failed: {error}"))?
        .as_secs()
        .to_string())
}

fn load_existing_records(path: &str) -> Result<Vec<AuditEventRecord>, String> {
    let payload = match fs::read_to_string(path) {
        Ok(payload) => payload,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("service api audit export read failed: {path}: {error}")),
    };
    let parsed: Value = serde_json::from_str(payload.as_str())
        .map_err(|error| format!("service api audit export parse failed: {path}: {error}"))?;
    let Some(records) = parsed.get("records").and_then(Value::as_array) else {
        return Err(format!("service api audit export parse failed: {path}: missing records"));
    };
    records.iter().map(parse_record).collect()
}

fn parse_record(value: &Value) -> Result<AuditEventRecord, String> {
    Ok(AuditEventRecord {
        domain: parse_domain(project_field(value, "domain")?)?,
        event_id: project_field(value, "event_id")?.to_owned(),
        occurred_at: project_field(value, "occurred_at")?.to_owned(),
        actor: project_field(value, "actor")?.to_owned(),
        action: project_field(value, "action")?.to_owned(),
        payload_digest: project_field(value, "payload_digest")?.to_owned(),
    })
}

fn parse_domain(value: &str) -> Result<AuditDomain, String> {
    match value {
        "Messages" => Ok(AuditDomain::Messages),
        "Tasks" => Ok(AuditDomain::Tasks),
        "Escrows" => Ok(AuditDomain::Escrows),
        "Reputation" => Ok(AuditDomain::Reputation),
        other => Err(format!("unknown audit domain: {other}")),
    }
}

fn project_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing audit export field: {field}"))
}

fn render_bundle_json(bundle: &kamn_core::AuditExportBundle) -> String {
    json!({
        "manifest": {
            "request_id": bundle.manifest.request_id,
            "requested_by": bundle.manifest.requested_by,
            "exported_at": bundle.manifest.exported_at,
            "format": "Json",
            "record_count": bundle.manifest.record_count,
            "integrity_hash": bundle.manifest.integrity_hash,
        },
        "records": bundle.records.iter().map(|record| json!({
            "domain": render_domain(&record.domain),
            "event_id": record.event_id,
            "occurred_at": record.occurred_at,
            "actor": record.actor,
            "action": record.action,
            "payload_digest": record.payload_digest,
        })).collect::<Vec<_>>(),
    })
    .to_string()
}

fn render_domain(domain: &AuditDomain) -> &'static str {
    match domain {
        AuditDomain::Messages => "Messages",
        AuditDomain::Tasks => "Tasks",
        AuditDomain::Escrows => "Escrows",
        AuditDomain::Reputation => "Reputation",
    }
}
