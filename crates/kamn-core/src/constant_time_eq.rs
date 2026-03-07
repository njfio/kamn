pub(crate) fn constant_time_eq_bytes(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}

pub(crate) fn constant_time_eq_str(left: &str, right: &str) -> bool {
    constant_time_eq_bytes(left.as_bytes(), right.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{constant_time_eq_bytes, constant_time_eq_str};

    #[test]
    fn equal_bytes_compare_true() {
        assert!(constant_time_eq_bytes(b"sig:alpha", b"sig:alpha"));
    }

    #[test]
    fn mismatched_bytes_compare_false() {
        assert!(!constant_time_eq_bytes(b"sig:alpha", b"sig:beta"));
    }

    #[test]
    fn mismatched_lengths_fail_closed() {
        assert!(!constant_time_eq_bytes(b"sig:alpha", b"sig:alpha:extra"));
        assert!(!constant_time_eq_str("sig:alpha", "sig:alpha:extra"));
    }
}
