use std::path::Path;

use kamn_agent_lib::KamnAgentHandle;

use super::super::artifact_digest::sha256_hex;
use super::super::live_task_binding::LiveTaskBinding;
use super::super::report::escape_json;

pub(super) struct SettlementAgreement {
    transaction_id: String,
    terms_digest: String,
    completion_digest: String,
    creator_did: String,
    provider_did: String,
    amount_lamports: u64,
}

impl SettlementAgreement {
    pub(super) fn new(
        run_dir: &Path,
        binding: Option<&LiveTaskBinding>,
        amount_lamports: u64,
        creator: &KamnAgentHandle,
        provider: &KamnAgentHandle,
    ) -> Result<Self, String> {
        let run_id = run_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "failed to derive MVP demo run id".to_owned())?;
        let creator_did = creator.identity().did().to_string();
        let provider_did = provider.identity().did().to_string();
        let binding_digest = binding.map_or("unbound", |value| value.digest.as_str());
        let seed =
            format!("{run_id}:{creator_did}:{provider_did}:{amount_lamports}:{binding_digest}");
        let digest = sha256_hex(seed.as_str());
        Ok(Self {
            transaction_id: format!("mvp-devnet-{}", &digest[..16]),
            terms_digest: digest,
            completion_digest: sha256_hex(format!("completed:{seed}").as_str()),
            creator_did,
            provider_did,
            amount_lamports,
        })
    }

    pub(super) fn task_payload(&self) -> String {
        format!(
            "{{\"provider_did\":\"{}\",\"transaction_id\":\"{}\",\"terms_digest\":\"{}\",\"idempotency_key\":\"{}-create\"}}",
            escape_json(self.provider_did.as_str()), self.transaction_id, self.terms_digest,
            self.transaction_id,
        )
    }

    pub(super) fn accept_payload(&self) -> String {
        format!("{{\"idempotency_key\":\"{}-accept\"}}", self.transaction_id)
    }

    pub(super) fn complete_payload(&self) -> String {
        format!(
            "{{\"idempotency_key\":\"{}-complete\",\"completion_evidence_digest\":\"{}\"}}",
            self.transaction_id, self.completion_digest,
        )
    }

    pub(super) fn release_payload(&self) -> String {
        format!(
            "{{\"idempotency_key\":\"{}-release\"}}",
            self.transaction_id
        )
    }

    pub(super) fn fund_payload(&self, task_id: &str) -> String {
        format!(
            "{{\"task_id\":\"{}\",\"transaction_id\":\"{}\",\"beneficiary_did\":\"{}\",\"amount_lamports\":{},\"network\":\"solana-devnet\",\"terms_digest\":\"{}\",\"release_authority_did\":\"{}\",\"release_policy\":\"task-completed\",\"idempotency_key\":\"{}-fund\"}}",
            escape_json(task_id), self.transaction_id, escape_json(self.provider_did.as_str()),
            self.amount_lamports, self.terms_digest, escape_json(self.creator_did.as_str()),
            self.transaction_id,
        )
    }
}
