use std::fmt::Display;

pub(super) type JsonField = (&'static str, String);

pub(super) fn gossip_status(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

pub(super) fn text_opt_str(value: &Option<String>) -> String {
    value.as_deref().unwrap_or("none").to_owned()
}

pub(super) fn text_opt_num<T: Display>(value: Option<T>) -> String {
    value
        .map(|item| item.to_string())
        .unwrap_or_else(|| "none".to_owned())
}

pub(super) fn text_opt_list(value: &Option<Vec<String>>) -> String {
    value
        .as_ref()
        .map(|items| text_list(items))
        .unwrap_or_else(|| "none".to_owned())
}

pub(super) fn text_list(items: &[String]) -> String {
    items.join(", ")
}

pub(super) fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub(super) fn json_str(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

pub(super) fn json_opt_str(value: &Option<String>) -> String {
    value
        .as_deref()
        .map(json_str)
        .unwrap_or_else(|| "null".to_owned())
}

pub(super) fn json_opt_num<T: Display>(value: Option<T>) -> String {
    value
        .map(|item| item.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

pub(super) fn json_opt_list(value: &Option<Vec<String>>) -> String {
    value
        .as_ref()
        .map(|items| json_list(items))
        .unwrap_or_else(|| "null".to_owned())
}

pub(super) fn json_list(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|item| json_str(item))
            .collect::<Vec<_>>()
            .join(",")
    )
}
