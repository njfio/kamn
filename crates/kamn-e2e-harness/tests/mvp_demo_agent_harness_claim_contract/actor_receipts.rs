use std::path::Path;

pub(crate) fn valid_actor_receipts(root: &Path) -> String {
    format!(
        r#","three_agent_actor_tool_receipts":[{},{},{},{},{}]"#,
        receipt(root, 1, "kamn_agent_a_register", "agent_a", "register"),
        receipt(
            root,
            2,
            "kamn_agent_a_invoke_transaction",
            "agent_a",
            "invoke_transaction"
        ),
        receipt(root, 3, "kamn_agent_b_register", "agent_b", "register"),
        receipt(
            root,
            4,
            "kamn_agent_b_accept_task",
            "agent_b",
            "accept_task"
        ),
        receipt(
            root,
            5,
            "kamn_agent_c_verify_three_agent_proof",
            "agent_c_verifier",
            "verify_proof"
        )
    )
}

fn receipt(root: &Path, sequence: u64, tool: &str, agent: &str, action: &str) -> String {
    let view = view_for(root, agent);
    format!(
        r#"{{"sequence":{},"tool":"{}","agent":"{}","action":"{}","outcome":"PASS","report_path":"{}","view_scope":"{}","view_artifact":"{}","view_digest":"{}"}}"#,
        sequence,
        tool,
        agent,
        action,
        root.join("proof/report.json").display(),
        view.scope,
        view.artifact.display(),
        view.digest
    )
}

fn view_for(root: &Path, agent: &str) -> View {
    match agent {
        "agent_a" => View::participant(root, "agent-a-view.json", "agent-a-view-digest-7045"),
        "agent_b" => View::participant(root, "agent-b-view.json", "agent-b-view-digest-7045"),
        _ => View::verifier(root),
    }
}

struct View {
    scope: &'static str,
    artifact: std::path::PathBuf,
    digest: &'static str,
}

impl View {
    fn participant(root: &Path, file: &str, digest: &'static str) -> Self {
        Self {
            scope: "participant-private",
            artifact: root.join("proof").join(file),
            digest,
        }
    }

    fn verifier(root: &Path) -> Self {
        Self {
            scope: "restricted-public",
            artifact: root.join("proof/agent-c-verifier-view.json"),
            digest: "agent-c-view-digest-7045",
        }
    }
}
