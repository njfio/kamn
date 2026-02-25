const DISPATCH_SOURCE: &str = include_str!("../src/dispatch.rs");
const PROTOCOL_SOURCE: &str = include_str!("../src/protocol.rs");

#[test]
fn spec_c03_mcp_json_escape_helper_is_not_duplicated_across_modules() {
    // Regression: #5935
    for (module, source) in [("dispatch", DISPATCH_SOURCE), ("protocol", PROTOCOL_SOURCE)] {
        assert!(
            !source.contains("fn escape_json("),
            "{module} must use canonical escape_json helper"
        );
    }
}

#[test]
fn spec_c03_mcp_json_field_lookup_helper_is_not_duplicated_across_modules() {
    // Regression: #5935
    for (module, source) in [("dispatch", DISPATCH_SOURCE), ("protocol", PROTOCOL_SOURCE)] {
        assert!(
            !source.contains("fn json_field_value"),
            "{module} must use canonical json_field_value helper"
        );
    }
}
