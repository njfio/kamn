pub(crate) fn strip_suffix_id<'a>(path: &'a str, prefix: &str, suffix: &str) -> &'a str {
    path.trim_start_matches(prefix)
        .trim_end_matches(suffix)
        .trim_end_matches('/')
}
