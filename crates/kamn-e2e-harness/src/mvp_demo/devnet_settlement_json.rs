use super::devnet_settlement::DevnetSettlementEvidence;
use super::report::escape_json;

pub(crate) fn parse_devnet_settlement_evidence(
    json: &str,
) -> Result<DevnetSettlementEvidence, String> {
    let evidence = DevnetSettlementEvidence {
        network: json_string_value(json, "network")?,
        execution_surface: "command-override".to_owned(),
        rpc_url: json_string_value(json, "rpc_url")?,
        payer_pubkey: json_string_value(json, "payer_pubkey")?,
        recipient_pubkey: json_string_value(json, "recipient_pubkey")?,
        lamports: json_u64_value(json, "lamports")?,
        escrow_id: json_string_value(json, "escrow_id")?,
        task_id: None,
        task_binding_digest: None,
        settlement_tx_signature: json_string_value(json, "settlement_tx_signature")?,
        settlement_commitment: json_string_value(json, "settlement_commitment")?,
        payer_balance_before: json_u64_value(json, "payer_balance_before")?,
        payer_balance_after: json_u64_value(json, "payer_balance_after")?,
        recipient_balance_before: json_u64_value(json, "recipient_balance_before")?,
        recipient_balance_after: json_u64_value(json, "recipient_balance_after")?,
        persisted_settlement_tx_signature: json_string_value(
            json,
            "persisted_settlement_tx_signature",
        )?,
    };
    validate_devnet_settlement_evidence(&evidence)?;
    Ok(evidence)
}

pub(crate) fn devnet_settlement_claim_json(evidence: &DevnetSettlementEvidence) -> String {
    let binding = binding_fields(evidence);
    format!(
        "{{\"id\":\"devnet_settlement_asset_movement\",\"label\":\"devnet-backed\",\"required\":true,\"status\":\"PASS\",\"summary\":\"Solana devnet escrow settlement transfer observed\",\"network\":\"{}\",\"execution_surface\":\"{}\",\"rpc_url\":\"{}\",\"payer_pubkey\":\"{}\",\"recipient_pubkey\":\"{}\",\"lamports\":{},\"escrow_id\":\"{}\",\"settlement_tx_signature\":\"{}\",\"settlement_commitment\":\"{}\",\"payer_balance_before\":{},\"payer_balance_after\":{},\"recipient_balance_before\":{},\"recipient_balance_after\":{},\"persisted_settlement_tx_signature\":\"{}\"{}}}",
        escape_json(evidence.network.as_str()),
        escape_json(evidence.execution_surface.as_str()),
        escape_json(evidence.rpc_url.as_str()),
        escape_json(evidence.payer_pubkey.as_str()),
        escape_json(evidence.recipient_pubkey.as_str()),
        evidence.lamports,
        escape_json(evidence.escrow_id.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        evidence.payer_balance_before,
        evidence.payer_balance_after,
        evidence.recipient_balance_before,
        evidence.recipient_balance_after,
        escape_json(evidence.persisted_settlement_tx_signature.as_str()),
        binding,
    )
}

fn binding_fields(evidence: &DevnetSettlementEvidence) -> String {
    match (&evidence.task_id, &evidence.task_binding_digest) {
        (Some(task_id), Some(digest)) => format!(
            ",\"task_id\":\"{}\",\"task_binding_digest\":\"{}\"",
            escape_json(task_id),
            escape_json(digest)
        ),
        _ => String::new(),
    }
}

pub(super) fn json_string_value(json: &str, key: &str) -> Result<String, String> {
    let value = json_value_after_key(json, key)?;
    let value = value.trim_start();
    let value = value
        .strip_prefix('"')
        .ok_or_else(|| format!("JSON field is not a string: {key}"))?;
    let end = value
        .find('"')
        .ok_or_else(|| format!("unterminated JSON string field: {key}"))?;
    Ok(value[..end].to_owned())
}

fn json_u64_value(json: &str, key: &str) -> Result<u64, String> {
    let digits = json_value_after_key(json, key)?
        .trim_start()
        .chars()
        .take_while(|item| item.is_ascii_digit())
        .collect::<String>();
    digits
        .parse::<u64>()
        .map_err(|error| format!("invalid JSON u64 field {key}: {error}"))
}

fn json_value_after_key<'a>(json: &'a str, key: &str) -> Result<&'a str, String> {
    let marker = format!("\"{key}\"");
    let key_start = json
        .find(marker.as_str())
        .ok_or_else(|| format!("missing JSON string field: {key}"))?
        + marker.len();
    let tail = json[key_start..].trim_start();
    tail.strip_prefix(':')
        .ok_or_else(|| format!("missing JSON field separator: {key}"))
}

fn validate_devnet_settlement_evidence(evidence: &DevnetSettlementEvidence) -> Result<(), String> {
    if evidence.network != "solana:devnet" {
        return Err("devnet settlement evidence network must be solana:devnet".to_owned());
    }
    if evidence.settlement_tx_signature != evidence.persisted_settlement_tx_signature {
        return Err(
            "devnet settlement persisted signature must match submitted signature".to_owned(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_json_string_value_accepts_pretty_json_spacing() {
        let value = super::json_string_value(
            r#"{"settlement_network": "solana:devnet"}"#,
            "settlement_network",
        )
        .expect("pretty JSON string field should parse");
        assert_eq!(value, "solana:devnet");
    }

    #[test]
    fn unit_json_u64_value_accepts_pretty_json_spacing() {
        let value = super::json_u64_value(
            r#"{"payer_balance_before": 2494975000}"#,
            "payer_balance_before",
        )
        .expect("pretty JSON u64 field should parse");
        assert_eq!(value, 2_494_975_000);
    }
}
