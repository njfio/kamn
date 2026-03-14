use crate::verify::support::{extract_json_string_marker, require_marker, strip_json_whitespace};

fn extract_blocks_payload(chain_dump_json: &str) -> Result<String, String> {
    let normalized = strip_json_whitespace(chain_dump_json);
    let blocks_marker = "\"blocks\":[";
    let blocks_start = normalized
        .find(blocks_marker)
        .ok_or_else(|| "chain dump missing blocks marker".to_owned())?
        + blocks_marker.len();
    let blocks_relative_end = normalized[blocks_start..]
        .find(']')
        .ok_or_else(|| "chain dump blocks payload malformed".to_owned())?;
    let blocks_payload = &normalized[blocks_start..blocks_start + blocks_relative_end];
    if blocks_payload.is_empty() {
        return Err("chain dump blocks array is empty".to_owned());
    }
    Ok(blocks_payload.to_owned())
}

fn push_block_pair(
    pairs: &mut Vec<(String, String)>,
    block_start: Option<usize>,
    index: usize,
    blocks_payload: &str,
) -> Result<(), String> {
    let start = block_start.ok_or_else(|| "chain dump blocks payload malformed".to_owned())?;
    pairs.push(read_block_pair(&blocks_payload[start..=index])?);
    Ok(())
}

fn start_block(block_start: &mut Option<usize>, depth: &mut usize, index: usize) {
    if *depth == 0 {
        *block_start = Some(index);
    }
    *depth += 1;
}

fn finish_block(
    pairs: &mut Vec<(String, String)>,
    block_start: &mut Option<usize>,
    depth: &mut usize,
    index: usize,
    blocks_payload: &str,
) -> Result<(), String> {
    if *depth == 0 {
        return Err("chain dump blocks payload malformed".to_owned());
    }
    *depth -= 1;
    if *depth == 0 {
        push_block_pair(pairs, *block_start, index, blocks_payload)?;
        *block_start = None;
    }
    Ok(())
}

fn parse_block_pairs(blocks_payload: &str) -> Result<Vec<(String, String)>, String> {
    let mut pairs = Vec::new();
    let mut depth = 0usize;
    let mut block_start = None;
    for (index, character) in blocks_payload.char_indices() {
        match character {
            '{' => start_block(&mut block_start, &mut depth, index),
            '}' => finish_block(
                &mut pairs,
                &mut block_start,
                &mut depth,
                index,
                blocks_payload,
            )?,
            _ => {}
        }
    }
    if depth != 0 || block_start.is_some() || pairs.is_empty() {
        return Err("chain dump blocks payload malformed".to_owned());
    }
    Ok(pairs)
}

fn extract_chain_block_hash_pairs(chain_dump_json: &str) -> Result<Vec<(String, String)>, String> {
    let blocks_payload = extract_blocks_payload(chain_dump_json)?;
    parse_block_pairs(&blocks_payload)
}

fn read_block_pair(block_fragment: &str) -> Result<(String, String), String> {
    let block_hash = extract_json_string_marker(block_fragment, "\"block_hash\":\"")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "chain dump block missing block_hash marker".to_owned())?;
    let previous_block_hash =
        extract_json_string_marker(block_fragment, "\"previous_block_hash\":\"")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "chain dump block missing previous_block_hash marker".to_owned())?;
    Ok((block_hash, previous_block_hash))
}

fn verify_chain_continuity(block_hash_pairs: &[(String, String)]) -> Result<(), String> {
    if block_hash_pairs[0].1.as_str() != "GENESIS" {
        return Err("chain dump genesis anchor mismatch at block index 0".to_owned());
    }
    for (index, pair) in block_hash_pairs.windows(2).enumerate() {
        if pair[1].1.as_str() != pair[0].0.as_str() {
            return Err(format!(
                "chain dump hash continuity mismatch at block index {}",
                index + 1
            ));
        }
    }
    Ok(())
}

/// Verifies required chain-dump markers used by deterministic `chain_check`.
pub fn verify_chain_dump(chain_dump_json: &str) -> Result<(), String> {
    require_marker(
        chain_dump_json,
        "\"chain_name\":",
        "chain dump missing chain_name marker",
    )?;
    require_marker(
        chain_dump_json,
        "\"chain_version\":",
        "chain dump missing chain_version marker",
    )?;
    require_marker(
        chain_dump_json,
        "\"blocks\":",
        "chain dump missing blocks marker",
    )?;
    let block_hash_pairs = extract_chain_block_hash_pairs(chain_dump_json)?;
    verify_chain_continuity(&block_hash_pairs)
}
