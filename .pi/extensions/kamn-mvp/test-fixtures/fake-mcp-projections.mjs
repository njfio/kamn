export function createProjections(resultMode) {
	let participantAttempts = 0;
	return {
		participant(taskId, name) {
			if (resultMode === "pending-three-projections" && participantAttempts++ < 3) return pending(taskId, name);
			if (resultMode === "pending-first-projection" && participantAttempts++ === 0) return pending(taskId, name);
			if (resultMode === "missing-first-escrow" && participantAttempts++ === 0) return unbound(taskId, name);
			return participant(taskId, name, resultMode);
		},
		verifier(taskId) { return { ...shared(taskId), view_scope: "restricted-public" }; },
	};
}

export function digest(character) { return `sha256:${character.repeat(64)}`; }

function pending(taskId, name) {
	const projection = participant(taskId, name);
	delete projection.settlement_tx_signature;
	delete projection.settlement_commitment;
	return projection;
}
function unbound(taskId, name) {
	const projection = participant(taskId, name);
	delete projection.escrow_id;
	return projection;
}
function participant(taskId, name, resultMode) {
	const suffix = name.endsWith("b") ? "b" : "a";
	const receipts = roleReceipts(suffix);
	if (resultMode === "projection-authority-mismatch") receipts[0].receipt_digest = digest("9");
	return {
		...shared(taskId), view_scope: "participant-private",
		participant_role: suffix === "a" ? "creator" : "provider",
		task_receipt_ids: receipts.filter((receipt) => receipt.action.startsWith("task:")).map((receipt) => receipt.receipt_id),
		receipt_chain_receipts: receipts,
	};
}
function roleReceipts(suffix) {
	const entries = suffix === "a"
		? [["1", "task:create", "task-live-1", "submitted"], ["2", "escrow:fund", "escrow-live-1", "funded"], ["3", "escrow:release-authorize", "escrow-live-1", "release-authorized"], ["6", "settlement:confirmed", "escrow-live-1", "confirmed"]]
		: [["4", "task:accept", "task-live-1", "accepted"], ["5", "task:complete", "task-live-1", "completed"]];
	return entries.map(([id, action, resource_id, resulting_state]) => ({
		receipt_id: id === "6" ? "settlement-intent-escrow-live-1" : `task-transition-receipt-${id}`,
		receipt_digest: digest(id), action, resource_id, resulting_state,
	}));
}
function shared(taskId) {
	return {
		task_id: taskId, transaction_id: "transaction-live-1", escrow_id: "escrow-live-1",
		amount_lamports: 1000000, network: "solana-devnet", settlement_tx_signature: "devnet-signature-1",
		settlement_commitment: "finalized", receipt_chain_commitment: digest("c"), public_commitment: digest("d"),
	};
}
