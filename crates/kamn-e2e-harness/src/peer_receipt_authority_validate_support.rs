use crate::PeerReceiptAuthorityError;

pub(crate) fn digest(
    stage: &'static str,
    field: &'static str,
    claimed: &str,
    computed: String,
) -> Result<(), PeerReceiptAuthorityError> {
    if !valid_digest(claimed) {
        return Err(error("PEER_AUTHORITY_DIGEST_INVALID", stage, field));
    }
    (claimed == computed)
        .then_some(())
        .ok_or_else(|| error("PEER_AUTHORITY_DIGEST_MISMATCH", stage, field))
}

pub(crate) fn time(stage: &'static str, field: &'static str) -> PeerReceiptAuthorityError {
    error("PEER_AUTHORITY_TIME_INVALID", stage, field)
}

pub(crate) fn error(
    code: &'static str,
    stage: &'static str,
    field: &'static str,
) -> PeerReceiptAuthorityError {
    PeerReceiptAuthorityError {
        code,
        message: format!("{stage} authority failed at {field}"),
        stage,
        field,
        context: format!("stage={stage},field={field}"),
        cause: None,
    }
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
