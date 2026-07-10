export type VerifiedActorEvidence = {
	task_id: string;
	state: "accepted";
	agent_a_pi_process_id: number;
	agent_b_pi_process_id: number;
	source_handoff_digest: string;
	source_agent_a_receipt_digest: string;
	source_agent_b_receipt_digest: string;
};
