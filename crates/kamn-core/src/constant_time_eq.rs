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
