#!/usr/bin/env node
import { appendFileSync } from "node:fs";
import { createInterface } from "node:readline";
import { createProjections, digest } from "./fake-mcp-projections.mjs";

const mode = process.env.KAMN_MVP_FAKE_MCP_MODE ?? "success";
const resultMode = process.env.KAMN_MVP_FAKE_MCP_RESULT_MODE ?? "success";
const startFile = process.env.KAMN_MVP_FAKE_MCP_START_FILE;
const stopFile = process.env.KAMN_MVP_FAKE_MCP_STOP_FILE;
const agentName = argumentValue("--agent-name") ?? "agent-a";
let releaseAttempts = 0;
let taskQueryAttempts = 0;
const projections = createProjections(resultMode);
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
	if (mode === "error-on-second" && request.id === "2") return write(errorResponse(request));
	if (mode === "rate-limit-on-second" && request.id === "2") return write(rateLimitResponse(request));
	if (mode === "ambiguous-first-release" && request.tool === "release_escrow" && releaseAttempts++ === 0) {
		return write(ambiguousReleaseResponse(request));
	}
	if (mode === "ambiguous-three-releases" && request.tool === "release_escrow" && releaseAttempts++ < 3) {
		return write(ambiguousReleaseResponse(request));
	}
	write(successResponse(request));
});

function successResponse(request) {
	const serviceResult = { ...toolResult(request), pid: process.pid, request_id: request.id };
	const result = authorityResult(request, serviceResult);
	return {
		ok: true,
		id: mode === "mismatch" ? "wrong-id" : request.id,
		tool: request.tool,
		result,
	};
}

function authorityResult(request, serviceResult) {
	if (!isAuthorityTool(request.tool)) return serviceResult;
	if (resultMode === "missing-authority") return serviceResult;
	const envelope = request.tool === "register" ? registrationAuthority(serviceResult) : mutationAuthority(request.tool, serviceResult);
	if (resultMode === "malformed-authority" && request.tool === "register") envelope.profile_commitment = "invalid";
	if (resultMode === "malformed-authority" && request.tool !== "register") envelope.service_receipt_digest = "invalid";
	if (resultMode === "mixed-authority-version") envelope.schema_version = "kamn.mcp.authority-receipt.v2";
	if (resultMode === "cross-role-authority" && request.tool !== "register") envelope.actor_did = "kamn:did:agent-b";
	if (resultMode === "wrong-tool-authority" && request.tool !== "register") envelope.tool = "query_task";
	if (resultMode === "copied-resource-authority" && request.tool !== "register") envelope.resource_id = "task-copied";
	return envelope;
}

function registrationAuthority(serviceResult) {
	return {
		schema_version: "kamn.mcp.authority-receipt.v1", authority_kind: "service-profile-commitment", source: "kamn-service",
		actor_did: serviceResult.did, tool: "register", resource_id: serviceResult.did,
		profile_commitment: serviceResult.profile_commitment, service_result: serviceResult,
	};
}

function mutationAuthority(tool, serviceResult) {
	const resource = tool.includes("escrow") ? serviceResult.escrow_id : serviceResult.task_id;
	return {
		schema_version: "kamn.mcp.authority-receipt.v1", authority_kind: "service-receipt", source: "kamn-service",
		actor_did: serviceResult.actor_did, tool, resource_id: resource, resulting_state: serviceResult.state,
		service_receipt_id: serviceResult.receipt_id, service_receipt_digest: serviceResult.receipt_digest,
		...(serviceResult.settlement_receipt_id ? { settlement_service_receipt: {
			actor_did: serviceResult.actor_did, tool, action: serviceResult.settlement_receipt_action,
			resource_id: serviceResult.settlement_receipt_resource_id,
			resulting_state: serviceResult.settlement_receipt_state,
			service_receipt_id: serviceResult.settlement_receipt_id,
			service_receipt_digest: serviceResult.settlement_receipt_digest,
		} } : {}),
		service_result: serviceResult,
	};
}

function toolResult(request) {
	if (request.tool === "register") {
		return { did: resultMode === "same-did" ? "kamn:did:shared" : actorDid(), profile_commitment: digest("f") };
	}
	if (request.tool === "query_agent_profile") return { did: request.did };
	if (request.tool === "create_task") {
		const payload = JSON.parse(request.payload);
		if (resultMode === "missing-task-id") return { state: "submitted", ...payload };
		const state = resultMode === "wrong-create-state" ? "queued" : "submitted";
		return mutationResult("create_task", "task-live-1", state, payload, "1");
	}
	if (request.tool === "accept_task") {
		const taskId = resultMode === "wrong-accept-id" ? "task-other" : request.task_id;
		return mutationResult("accept_task", taskId, "accepted", JSON.parse(request.payload), "4");
	}
	if (request.tool === "query_task") {
		if (resultMode === "completed-second-query") {
			return { task_id: request.task_id, state: taskQueryAttempts++ === 0 ? "accepted" : "completed" };
		}
		const state = resultMode === "wrong-query-state" ? "submitted" : "accepted";
		return { task_id: request.task_id, state };
	}
	if (request.tool === "complete_task") return mutationResult("complete_task", request.task_id, "completed", JSON.parse(request.payload), "5");
	if (request.tool === "fund_escrow") return mutationResult("fund_escrow", "escrow-live-1", "funded", JSON.parse(request.payload), "2");
	if (request.tool === "release_escrow") {
		return mutationResult("release_escrow", request.escrow_id, "release-authorized", {
			settlement_tx_signature: "devnet-signature-1", ...JSON.parse(request.payload),
			settlement_receipt_id: "settlement-intent-escrow-live-1",
			settlement_receipt_digest: digest("6"), settlement_receipt_action: "settlement:confirmed",
			settlement_receipt_resource_id: request.escrow_id, settlement_receipt_state: "confirmed",
		}, "3");
	}
	if (request.tool === "query_participant_task_projection") {
		return projections.participant(request.task_id, agentName);
	}
	if (request.tool === "query_verifier_task_projection") {
		return projections.verifier(request.task_id);
	}
	return {};
}

function mutationResult(tool, resource, state, fields, suffix) {
	const action = {
		create_task: "task:create", accept_task: "task:accept", complete_task: "task:complete",
		fund_escrow: "escrow:fund", release_escrow: "escrow:release-authorize",
	}[tool];
	return {
		...(tool.includes("escrow") ? { escrow_id: resource } : { task_id: resource }), ...fields,
		actor_did: actorDid(), action, state, receipt_id: `task-transition-receipt-${suffix}`, receipt_digest: digest(suffix),
	};
}

function isAuthorityTool(tool) {
	return ["register", "create_task", "accept_task", "complete_task", "fund_escrow", "release_escrow"].includes(tool);
}
function actorDid() { return `kamn:did:${agentName}`; }
function errorResponse(request) {
	return {
		ok: false,
		id: request.id,
		tool: request.tool,
		error: { kind: "backend_error", message: "forced backend failure" },
	};
}

function ambiguousReleaseResponse(request) {
	return {
		ok: false,
		id: request.id,
		tool: request.tool,
		error: { kind: "backend_error", message: "SETTLEMENT_OUTCOME_AMBIGUOUS: reconciliation required" },
	};
}

function rateLimitResponse(request) {
	return {
		ok: false,
		id: request.id,
		tool: request.tool,
		error: { kind: "backend_error", message: "sender anti-spam rate limit exceeded: observed=3, limit=3, window_seconds=1" },
	};
}

function write(response) {
	process.stdout.write(`${JSON.stringify(response)}\n`);
}

function argumentValue(flag) {
	const index = process.argv.indexOf(flag);
	return index >= 0 ? process.argv[index + 1] : undefined;
}
