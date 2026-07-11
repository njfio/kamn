#!/usr/bin/env node
import { appendFileSync } from "node:fs";
import { createInterface } from "node:readline";

const mode = process.env.KAMN_MVP_FAKE_MCP_MODE ?? "success";
const resultMode = process.env.KAMN_MVP_FAKE_MCP_RESULT_MODE ?? "success";
const startFile = process.env.KAMN_MVP_FAKE_MCP_START_FILE;
const stopFile = process.env.KAMN_MVP_FAKE_MCP_STOP_FILE;
const agentName = argumentValue("--agent-name") ?? "agent-a";
if (startFile) appendFileSync(startFile, `${process.pid}\n`);

process.on("SIGTERM", () => {
	if (stopFile) appendFileSync(stopFile, `${process.pid}\n`);
	process.exit(0);
});

createInterface({ input: process.stdin }).on("line", (line) => {
	if (mode === "exit") process.exit(7);
	if (mode === "hang") return;
	if (mode === "malformed") return process.stdout.write("not-json\n");
	const request = JSON.parse(line);
	if (mode === "error") return write(errorResponse(request));
	write(successResponse(request));
});

function successResponse(request) {
	const result = toolResult(request);
	return {
		ok: true,
		id: mode === "mismatch" ? "wrong-id" : request.id,
		tool: request.tool,
		result: { ...result, pid: process.pid, request_id: request.id },
	};
}

function toolResult(request) {
	if (request.tool === "register") {
		return { did: resultMode === "same-did" ? "kamn:did:shared" : `kamn:did:${agentName}` };
	}
	if (request.tool === "query_agent_profile") return { did: request.did };
	if (request.tool === "create_task") {
		const payload = JSON.parse(request.payload);
		if (resultMode === "missing-task-id") return { state: "submitted", ...payload };
		const state = resultMode === "wrong-create-state" ? "queued" : "submitted";
		return { task_id: "task-live-1", state, ...payload };
	}
	if (request.tool === "accept_task") {
		const taskId = resultMode === "wrong-accept-id" ? "task-other" : request.task_id;
		return { task_id: taskId, state: "accepted" };
	}
	if (request.tool === "query_task") {
		const state = resultMode === "wrong-query-state" ? "submitted" : "accepted";
		return { task_id: request.task_id, state };
	}
	if (request.tool === "complete_task") return { task_id: request.task_id, state: "completed" };
	if (request.tool === "fund_escrow") return { escrow_id: "escrow-live-1", state: "funded" };
	if (request.tool === "release_escrow") {
		return { escrow_id: request.escrow_id, state: "released", settlement_tx_signature: "devnet-signature-1" };
	}
	if (request.tool === "query_participant_task_projection") {
		return participantProjection(request.task_id, agentName);
	}
	if (request.tool === "query_verifier_task_projection") {
		return { ...sharedProjection(request.task_id), view_scope: "restricted-public" };
	}
	return {};
}

function participantProjection(taskId, name) {
	const suffix = name.endsWith("b") ? "b" : "a";
	return {
		...sharedProjection(taskId),
		view_scope: "participant-private",
		private_receipt_digest: `sha256:participant-${suffix}`,
	};
}

function sharedProjection(taskId) {
	return {
		task_id: taskId,
		transaction_id: "transaction-live-1",
		escrow_id: "escrow-live-1",
		amount_lamports: 1000000,
		network: "solana-devnet",
		settlement_tx_signature: "devnet-signature-1",
		settlement_commitment: "finalized",
		public_commitment: "sha256:shared",
	};
}

function errorResponse(request) {
	return {
		ok: false,
		id: request.id,
		tool: request.tool,
		error: { kind: "backend_error", message: "forced backend failure" },
	};
}

function write(response) {
	process.stdout.write(`${JSON.stringify(response)}\n`);
}

function argumentValue(flag) {
	const index = process.argv.indexOf(flag);
	return index >= 0 ? process.argv[index + 1] : undefined;
}
