use std::path::Path;

use super::AgentTransactionDemoConfig;

pub(super) struct AgentTransactionEvidencePaths {
    pub(super) handoff: String,
    pub(super) agent_a_receipt: String,
    pub(super) agent_b_receipt: String,
    pub(super) agent_c_observation: String,
    pub(super) actors: [String; 3],
    pub(super) run_id: String,
}

impl AgentTransactionEvidencePaths {
    pub(super) fn prepare(config: &AgentTransactionDemoConfig) -> Result<Self, String> {
        let root = Path::new(config.staging_root.as_str());
        reject_existing_artifacts(root)?;
        std::fs::create_dir_all(root)
            .map_err(|_| "AGENT_TRANSACTION_OUTPUT_CONFLICT: staging root failed".to_owned())?;
        Ok(Self {
            handoff: path(root, "handoff.json"),
            agent_a_receipt: path(root, "task-agent-a.json"),
            agent_b_receipt: path(root, "task-agent-b.json"),
            agent_c_observation: path(root, "task-agent-c.json"),
            actors: [
                path(root, "agent-a.json"),
                path(root, "agent-b.json"),
                path(root, "agent-c.json"),
            ],
            run_id: format!("agent-transaction-{}", std::process::id()),
        })
    }

    pub(super) fn environment(&self) -> Vec<(&'static str, String)> {
        vec![
            ("KAMN_MVP_LIVE_TASK_HANDOFF_FILE", self.handoff.clone()),
            (
                "KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE",
                self.agent_a_receipt.clone(),
            ),
            (
                "KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE",
                self.agent_b_receipt.clone(),
            ),
            (
                "KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE",
                self.agent_c_observation.clone(),
            ),
            (
                "KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE",
                self.actors[0].clone(),
            ),
            (
                "KAMN_MVP_PI_TRANSACTION_AGENT_B_FILE",
                self.actors[1].clone(),
            ),
            (
                "KAMN_MVP_PI_TRANSACTION_AGENT_C_FILE",
                self.actors[2].clone(),
            ),
            ("KAMN_MVP_PI_RUN_ID", self.run_id.clone()),
        ]
    }
}

fn reject_existing_artifacts(root: &Path) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    let occupied = std::fs::read_dir(root)
        .map_err(|_| "AGENT_TRANSACTION_OUTPUT_CONFLICT: staging root unreadable".to_owned())?
        .next()
        .is_some();
    if occupied {
        return Err("AGENT_TRANSACTION_OUTPUT_CONFLICT: staging root is not empty".to_owned());
    }
    Ok(())
}

fn path(root: &Path, name: &str) -> String {
    root.join(name).display().to_string()
}
