fn section_between<'a>(workflow: &'a str, start: &str, end: Option<&str>) -> Option<&'a str> {
    let start_idx = workflow.find(start)?;
    let end_idx = end.and_then(|marker| workflow[start_idx..].find(marker).map(|idx| start_idx + idx));
    Some(&workflow[start_idx..end_idx.unwrap_or(workflow.len())])
}

pub(crate) fn sdk_direct_section(workflow: &str) -> Option<&str> {
    section_between(workflow, "  e2e-sdk-direct:", Some("  e2e-mcp-agent:"))
}

pub(crate) fn mcp_agent_section(workflow: &str) -> Option<&str> {
    section_between(workflow, "  e2e-mcp-agent:", Some("  e2e-cli-smoke:"))
}

pub(crate) fn cli_smoke_section(workflow: &str) -> Option<&str> {
    section_between(workflow, "  e2e-cli-smoke:", None)
}
