pub(crate) fn expected_escrow_id(payload: &str) -> String {
    format!(
        "escrow-local-{:016x}",
        deterministic_body_tag(payload.as_bytes())
    )
}

fn deterministic_body_tag(payload: &[u8]) -> u64 {
    payload.iter().fold(0xcbf29ce484222325_u64, |acc, byte| {
        acc.wrapping_mul(0x00000100000001B3) ^ u64::from(*byte)
    })
}
