pub(crate) fn matching_object<'a>(
    raw: &'a str,
    start: usize,
    context: &str,
) -> Result<&'a str, String> {
    let mut depth = 0_u64;
    for (offset, byte) in raw[start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' if depth == 0 => break,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(&raw[start..start + offset + 1]);
                }
            }
            _ => {}
        }
    }
    Err(format!("malformed {context} object"))
}
